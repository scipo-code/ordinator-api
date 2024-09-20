use crate::sap_mapper_and_types::{DATS, TIMS};
use calamine::{Data, Error, Reader, Xlsx};
use core::fmt;
use regex::Regex;
use shared_types::scheduling_environment::time_environment::day::Day;
use shared_types::Asset;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, event, info, warn};

use shared_types::scheduling_environment::time_environment::period::Period;
use shared_types::scheduling_environment::time_environment::TimeEnvironment;
use shared_types::scheduling_environment::work_order::system_condition::SystemCondition;

use chrono::{
    naive, DateTime, Datelike, Days, Duration, NaiveDate, NaiveTime, TimeZone, Timelike, Utc,
};
use shared_types::scheduling_environment::work_order::functional_location::FunctionalLocation;
use shared_types::scheduling_environment::work_order::priority::Priority;
use shared_types::scheduling_environment::work_order::revision::Revision;
use shared_types::scheduling_environment::work_order::status_codes::{MaterialStatus, StatusCodes};
use shared_types::scheduling_environment::work_order::work_order_dates::WorkOrderDates;
use shared_types::scheduling_environment::work_order::work_order_text::WorkOrderText;
use shared_types::scheduling_environment::work_order::work_order_type::{
    WDFPriority, WGNPriority, WPMPriority,
};
use shared_types::scheduling_environment::work_order::work_order_type::{
    WROPriority, WorkOrderType,
};

use shared_types::scheduling_environment::work_order::unloading_point::UnloadingPoint;
use shared_types::scheduling_environment::work_order::{
    ActivityRelation, WorkOrder, WorkOrderAnalytic, WorkOrderInfo, WorkOrderInfoDetail,
    WorkOrderNumber,
};
use shared_types::scheduling_environment::worker_environment::resources::{
    MainResources, Resources,
};

use shared_types::scheduling_environment::worker_environment::WorkerEnvironment;
use shared_types::scheduling_environment::{SchedulingEnvironment, WorkOrders};

use regex;
use shared_types::scheduling_environment::work_order::operation::operation_analytic::OperationAnalytic;
use shared_types::scheduling_environment::work_order::operation::operation_info::OperationInfo;
use shared_types::scheduling_environment::work_order::operation::{
    ActivityNumber, Operation, OperationDates, Work,
};

use super::{SchedulingEnvironmentFactory, SchedulingEnvironmentFactoryError};

#[derive(Clone)]
pub struct TotalExcel<'a> {
    file_path: &'a Path,
    number_of_strategic_periods: u64,
    number_of_tactical_periods: u64,
    number_of_days: u64,
}

impl<'a> TotalExcel<'a> {
    pub fn new(
        file_path: &'a Path,
        number_of_strategic_periods: u64,
        number_of_tactical_periods: u64,
        number_of_days: u64,
    ) -> Self {
        Self {
            file_path,
            number_of_strategic_periods,
            number_of_tactical_periods,
            number_of_days,
        }
    }
}

impl<'a> SchedulingEnvironmentFactory<TotalExcel<'a>> for SchedulingEnvironment {
    fn create_scheduling_environment(
        data_source: TotalExcel<'a>,
    ) -> Result<SchedulingEnvironment, SchedulingEnvironmentFactoryError> {
        let file_path_str = data_source.file_path.to_str().unwrap();
        let mut workbook: Xlsx<_> =
            calamine::open_workbook(data_source.file_path).expect(&format!("{}", file_path_str));

        info!(
            "Excel file from path {:?} successfully loaded",
            data_source.file_path
        );
        let sheet: &calamine::Range<calamine::Data> = &workbook
            .worksheet_range_at(0)
            .ok_or(calamine::Error::Msg("Cannot find work order sheet"))?
            .expect("Could not load work order sheet.");

        let mut work_orders: WorkOrders = WorkOrders::default();
        let worker_environment: WorkerEnvironment = WorkerEnvironment::new();

        let strategic_periods: Vec<Period> =
            create_periods(data_source.number_of_strategic_periods).unwrap_or_else(|_| {
                panic!(
                    "Could not create periods in {} at line {}",
                    file!(),
                    line!()
                )
            });

        let tactical_periods: &Vec<Period> =
            &strategic_periods.clone()[0..data_source.number_of_tactical_periods as usize].to_vec();

        let first_period = strategic_periods.first().unwrap().clone();

        let tactical_days = |number_of_days: u64| -> Vec<Day> {
            let mut days: Vec<Day> = Vec::new();
            let mut date = first_period.start_date().to_owned();
            for day_index in 0..number_of_days {
                days.push(Day::new(day_index as usize, date.to_owned()));
                date = date.checked_add_days(Days::new(1)).unwrap();
            }
            days
        };

        populate_work_orders(&mut work_orders, &strategic_periods, sheet)
            .expect("could not populate the work orders");

        let time_environment = TimeEnvironment::new(
            strategic_periods,
            tactical_periods.to_vec(),
            tactical_days(data_source.number_of_days),
        );

        let scheduling_environment =
            SchedulingEnvironment::new(work_orders, worker_environment, time_environment);
        Ok(scheduling_environment)
    }
}

fn populate_work_orders<'a>(
    work_orders: &'a mut WorkOrders,
    periods: &[Period],
    sheet: &'a calamine::Range<calamine::Data>,
) -> Result<&'a mut WorkOrders, calamine::Error> {
    let headers: Vec<String> = sheet
        .rows()
        .next()
        .ok_or(calamine::Error::Msg("Sheet is empty"))?
        .iter()
        .filter_map(|cell| {
            if let calamine::Data::String(s) = cell {
                Some(s.clone())
            } else {
                None
            }
        })
        .collect();

    let header_to_index: HashMap<String, usize> = headers
        .iter()
        .enumerate()
        .map(|(index, header)| (header.clone(), index))
        .collect();

    let mut counter = 0;
    for row in sheet.rows().skip(1) {
        let work_order_column = 5;
        if row[work_order_column] == Data::Empty || row[work_order_column] == "" {
            counter += 1;
            continue;
        }
        info!("processed {} WorkOrder", counter);
        let work_order_number: WorkOrderNumber = match header_to_index.get("WO_Number") {
            Some(column_index) => {
                let work_order_string = &row[*column_index];
                match &work_order_string {
                    calamine::Data::Empty => continue,
                    calamine::Data::String(s) => match s.parse::<u64>() {
                        Ok(n) => WorkOrderNumber(n),
                        Err(e) => {
                            dbg!(
                                "Could not parse work order number as string: {}, ",
                                e,
                                work_order_string
                            );
                            panic!("WorkOrderNumber could not be extracted from the inputted excel file");
                        }
                    },
                    calamine::Data::Int(s) => WorkOrderNumber(*s as u64),
                    calamine::Data::Float(s) => WorkOrderNumber(*s as u64),
                    _ => {
                        todo!("Handle other cases of calamine::Data");
                    }
                }
            }
            None => panic!("Input excel data have rows without a WorkOrderNumber"),
        };

        if work_orders.new_work_order(work_order_number) {
            work_orders.insert(
                create_new_work_order(row, &header_to_index, periods)
                    .expect("Could not insert new work order"),
            );
        }

        let operation: Operation =
            create_new_operation(row, &header_to_index).expect("Could not create a new operation");

        work_orders
            .inner
            .get_mut(&work_order_number)
            .expect("Work order not yet created")
            .insert_operation(operation);
    }
    Ok(work_orders)
}

fn create_new_work_order(
    row: &[calamine::Data],
    header_to_index: &HashMap<String, usize>,
    periods: &[Period],
) -> Result<WorkOrder, Error> {
    let work_order_type_possible_headers = ["Order Type", "Order_Type", "WO_Order_Type"];
    let main_work_center_possible_headers = [
        "Main Work Center",
        "Main_Work_Center",
        "Main WorkCtr",
        "WBS_Name_right",
    ];

    let work_order_type_data =
        get_data_from_headers(row, header_to_index, &work_order_type_possible_headers);

    let main_work_center_data =
        get_data_from_headers(row, header_to_index, &main_work_center_possible_headers);

    let priority = match row
        .get(
            *header_to_index
                .get("WO_Priority")
                .ok_or("Priority header not found")?,
        )
        .cloned()
    {
        Some(calamine::Data::Int(n)) => Priority::IntValue(n as u64),
        Some(calamine::Data::String(s)) => {
            match s.parse::<u64>() {
                Ok(num) => Priority::IntValue(num), // If successful, use the integer value
                Err(_) => Priority::StringValue(s), // If not, fall back to using the string
            }
        }
        Some(calamine::Data::Float(n)) => Priority::IntValue(n as u64),
        _ => Priority::StringValue(String::new()),
    };

    let main_work_center = match main_work_center_data {
        Some(calamine::Data::String(s)) => MainResources::new_from_string(s.clone()),
        _ => return Err(Error::Msg("Could not parse Main Work Center as string")),
    };

    let work_order_number = match row
        .get(
            *header_to_index
                .get("WO_Number")
                .ok_or("Order header not found")?,
        )
        .cloned()
    {
        Some(calamine::Data::Int(n)) => WorkOrderNumber(n as u64),
        Some(calamine::Data::Float(n)) => WorkOrderNumber(n as u64),
        Some(calamine::Data::String(s)) => WorkOrderNumber(s.parse::<u64>().unwrap_or(0)),
        _ => panic!("Work order number could not be parsed"),
    };

    let work_order_analytic = WorkOrderAnalytic::new(
        0,
        Work::from(0.0),
        HashMap::new(),
        false,
        false,
        extract_status_codes(row, header_to_index).expect("Failed to extract StatusCodes"),
    );

    let work_order_info = WorkOrderInfo::new(
        priority.clone(),
        extract_order_type_and_priority(work_order_type_data, priority),
        extract_functional_location(row, header_to_index)
            .expect("Failed to extract FunctionalLocation"),
        extract_order_text(row, header_to_index).expect("Failed to extract OrderText"),
        extract_unloading_point(row, header_to_index, periods)
            .expect("Failed to extract UnloadingPoint"),
        extract_revision(row, header_to_index).expect("Failed to extract Revision"),
        SystemCondition::default(),
        WorkOrderInfoDetail::default(),
    );

    Ok(WorkOrder::new(
        work_order_number,
        main_work_center,
        HashMap::<ActivityNumber, Operation>::new(),
        Vec::<ActivityRelation>::new(),
        work_order_analytic,
        extract_order_dates(row, header_to_index, periods).expect("Failed to extract OrderDates"),
        work_order_info,
    ))
}

fn create_new_operation(
    row: &[calamine::Data],
    header_to_index: &HashMap<String, usize>,
) -> Result<Operation, Error> {
    let _default_future_date = Utc.with_ymd_and_hms(2026, 1, 1, 7, 0, 0).unwrap();

    let work_possible_headers = [
        "OPR_Planned_Work",
        "Remaining Work",
        "Work_Remaining",
        "Work_Planned",
        "Work",
    ];
    let earliest_start_date_headers = [
        "Earliest_Start_Date",
        "Earliest start date",
        "Earliest StrDate",
        "OPR_Start_Date",
    ];
    let earliest_start_time_headers = ["Earliest start time", "Earliest_Start_Time"];
    let earliest_finish_date_headers = [
        "Earliest_Finish_Date",
        "Earliest_End_Date",
        "Earliest finish date",
        "Earliest end date",
        "OPR_End_Date",
    ];
    let earliest_finish_time_headers = [
        "Earliest_Finish_Time",
        "Latest_Finish_Time",
        "Earliest finish time",
        "OPR_End_Time",
    ];
    let work_center_headers = [
        "Work_Center",
        "Work Center",
        "Work center",
        "Oper.WorkCenter",
        "WBS_Name",
    ];
    let actual_work_headers = [
        "Work_Actual",
        "Work Actual",
        "Actual work",
        "Work Actual (Hrs)",
        "OPR_Actual_Work",
    ];

    let earliest_start_date_data =
        get_data_from_headers(row, header_to_index, &earliest_start_date_headers);
    let earliest_start_time_data =
        get_data_from_headers(row, header_to_index, &earliest_start_time_headers);
    let earliest_finish_date_data =
        get_data_from_headers(row, header_to_index, &earliest_finish_date_headers);
    let earliest_finish_time_data =
        get_data_from_headers(row, header_to_index, &earliest_finish_time_headers);
    let work_center_data = get_data_from_headers(row, header_to_index, &work_center_headers);
    let work_remaining_data = get_data_from_headers(row, header_to_index, &work_possible_headers);
    let actual_work_data = get_data_from_headers(row, header_to_index, &actual_work_headers);

    let activity = match row
        .get(
            *header_to_index
                .get("OPR_Activity_Number")
                .ok_or("Activity header not found")?,
        )
        .cloned()
    {
        Some(calamine::Data::Int(n)) => ActivityNumber(n as u64),
        Some(calamine::Data::Float(n)) => ActivityNumber(n as u64),
        Some(calamine::Data::String(s)) => ActivityNumber(s.parse::<u64>().unwrap_or(0)),
        _ => {
            panic!("Activity number is not present or could not be parsed. That should not happen")
        }
    };

    let operating_time = Work::from(6.0);

    let operation_info = OperationInfo::new(
        match row
            .get(
                *header_to_index
                    .get("OPR_Workers_Numbers")
                    .unwrap_or(&1_usize),
            )
            .cloned()
        {
            Some(calamine::Data::Int(n)) => n as u64,
            Some(calamine::Data::Float(n)) => n as u64,
            Some(calamine::Data::String(s)) => s.parse::<u64>().unwrap_or(1),
            _ => 1,
        },
        match work_remaining_data.cloned() {
            Some(calamine::Data::Int(planned_work)) => Work::from(planned_work as f64),
            Some(calamine::Data::Float(planned_work)) => Work::from(planned_work),
            Some(calamine::Data::String(planned_work)) => {
                planned_work.parse::<Work>().unwrap_or(Work::from(0.0))
            }
            _ => Work::from(100000.0),
        },
        match actual_work_data.cloned() {
            Some(calamine::Data::Int(actual_work)) => Work::from(actual_work as f64),
            Some(calamine::Data::Float(actual_work)) => Work::from(actual_work),
            Some(calamine::Data::String(s)) => s.parse::<Work>().unwrap_or(Work::from(0.0)),
            _ => Work::from(0.0),
        },
        Work::from(0.0),
        operating_time,
    );

    let operation_analytic = OperationAnalytic::new(
        Work::from(0.0),
        match header_to_index.get("OPR_Duration") {
            Some(index) => match row.get(*index).cloned() {
                Some(calamine::Data::Int(duration)) => Work::from(duration as f64),
                Some(calamine::Data::Float(duration)) => Work::from(duration),
                Some(calamine::Data::String(duration)) => duration
                    .parse::<Work>()
                    .expect("Duration is not a valid number"),
                _ => Work::from(0.0),
            },
            None => {
                if operation_info.number() != 0 {
                    operation_info
                        .work_remaining()
                        .cal_duration(operation_info.number())
                } else {
                    Work::from(0.0)
                }
            }
        },
    );

    let earliest_start_datetime = {
        let date = match earliest_start_date_data.cloned() {
            Some(calamine::Data::String(s)) => parse_date(&s),
            Some(calamine::Data::DateTime(s)) => {
                let start = NaiveDate::from_ymd_opt(1900, 1, 1).expect("DATE");
                let date = start.checked_add_signed(Duration::days(s.as_f64() as i64 - 2));
                date.unwrap()
            }
            Some(calamine::Data::Float(s)) => {
                let var_name = s.to_string();
                DATS(var_name).into()
            }
            _ => return Err(Error::Msg("Could not parse Earliest_Start_Date as string")),
        };

        let time = match earliest_start_time_data.cloned() {
            Some(calamine::Data::String(s)) => match NaiveTime::parse_from_str(&s, "%H:%M:%s") {
                Ok(naive_date) => naive_date,
                Err(_) => {
                    println!(
                        "Could not parse earliest_start_time_data from string: {}",
                        s
                    );
                    NaiveTime::from_hms_opt(7, 0, 0).unwrap()
                }
            },
            Some(calamine::Data::DateTime(s)) => excel_time_to_hh_mm_ss(s.as_f64()),
            Some(calamine::Data::Float(s)) => {
                let var_name = s.to_string();
                TIMS(var_name).into()
            }
            _ => {
                event!(
                    tracing::Level::DEBUG,
                    "Could not parse earliest_start_time is not present"
                );
                NaiveTime::from_hms_opt(7, 0, 0).unwrap()
            }
        };

        Utc.from_utc_datetime(&naive::NaiveDateTime::new(date, time))
    };

    let earliest_finish_datetime = {
        let date = match earliest_finish_date_data.cloned() {
            Some(calamine::Data::String(s)) => parse_date(&s),
            Some(calamine::Data::DateTime(s)) => {
                let start = NaiveDate::from_ymd_opt(1900, 1, 1).expect("DATE");
                let date = start.checked_add_signed(Duration::days(s.as_f64() as i64 - 2));
                date.unwrap()
            }
            Some(calamine::Data::Float(s)) => {
                let var_name = s.to_string();
                DATS(var_name).into()
            }

            _ => NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        };

        let time = match earliest_finish_time_data.cloned() {
            Some(calamine::Data::String(s)) => match NaiveTime::parse_from_str(&s, "%H:%M:%s") {
                Ok(naive_date) => naive_date,
                Err(_) => {
                    println!(
                        "Could not parse earliest_finish_time_data from string: {}",
                        s
                    );
                    NaiveTime::from_hms_opt(7, 0, 0).unwrap()
                }
            },
            Some(calamine::Data::DateTime(s)) => excel_time_to_hh_mm_ss(s.as_f64()),
            Some(calamine::Data::Float(s)) => {
                let var_name = s.to_string();
                TIMS(var_name).into()
            }
            _ => NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
        };
        Utc.from_utc_datetime(&naive::NaiveDateTime::new(date, time))
    };

    let operation_dates = OperationDates::new(
        Day::new(0, Utc::now()),
        Day::new(0, Utc::now()),
        earliest_start_datetime,
        earliest_finish_datetime,
    );

    Ok(Operation::new(
        activity,
        match work_center_data.cloned() {
            Some(calamine::Data::String(s)) => Resources::new_from_string(s),
            _ => return Err(Error::Msg("Could not parse work center as string")),
        },
        operation_info,
        operation_analytic,
        operation_dates,
    ))
}

/// This function will extract the status codes from the row and return them as a StatusCodes struct.
fn extract_status_codes(
    row: &[calamine::Data],
    header_to_index: &HashMap<String, usize>,
) -> Result<StatusCodes, Error> {
    let system_status_possible_headers = [
        "System_Status",
        "System Status",
        "Order System Status",
        "System status",
        "WO_I_Status_Code",
    ];
    let user_status_possible_headers = [
        "User_Status",
        "User Status",
        "Order User Status",
        "User status",
        "WO_E_Status_Code",
    ];
    let op_status_possible_headers = [
        "OPR_E_Status_Code",
        "Opr_User_Status",
        "Op User Status",
        "Oper.UserStatus",
    ];

    let system_status_data =
        get_data_from_headers(row, header_to_index, &system_status_possible_headers);
    let user_status_data =
        get_data_from_headers(row, header_to_index, &user_status_possible_headers);
    let op_status_data = get_data_from_headers(row, header_to_index, &op_status_possible_headers);

    let system_status = match system_status_data.cloned() {
        Some(calamine::Data::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse system status as string")),
    };

    let user_status = match user_status_data.cloned() {
        Some(calamine::Data::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse user status as string")),
    };

    let opr_user_status = match op_status_data.cloned() {
        Some(calamine::Data::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse opr user status as string")),
    };

    let opr_system_status = match header_to_index.get("Opr_System_Status") {
        Some(index) => match row.get(*index).cloned() {
            Some(calamine::Data::String(s)) => s,
            None => "Not present".to_string(),
            _ => return Err(Error::Msg("Opr_System_Status value is not a string")),
        },
        None => "Column not present".to_string(), // Handle the case where the column is absent
    };

    // concatenate the status codes into a single string
    let status_codes_string = format!(
        "{} {} {} {}",
        system_status, user_status, opr_user_status, opr_system_status
    );

    let pcnf_pattern = regex::Regex::new(r"PCNF").unwrap();
    let awsc_pattern = regex::Regex::new(r"AWSC").unwrap();
    let well_pattern = regex::Regex::new(r"WELL").unwrap();
    let sch_pattern = regex::Regex::new(r"SCH").unwrap();
    let sece_pattern = regex::Regex::new(r"SECE").unwrap();

    let material_status: MaterialStatus =
        MaterialStatus::from_status_code_string(&status_codes_string);

    Ok(StatusCodes {
        material_status,
        pcnf: pcnf_pattern.is_match(&status_codes_string),
        awsc: awsc_pattern.is_match(&status_codes_string),
        well: well_pattern.is_match(&status_codes_string),
        sch: sch_pattern.is_match(&status_codes_string),
        sece: sece_pattern.is_match(&status_codes_string),
        unloading_point: false, // Assuming default value; modify as needed
    })
}

fn extract_order_dates(
    row: &[calamine::Data],
    header_to_index: &HashMap<String, usize>,
    periods: &[Period],
) -> Result<WorkOrderDates, Error> {
    let earliest_allowed_start_date_possible_headers = [
        "WO_Earliest_Allowed_Start_Date",
        "Earliest Allowed Start Date",
        "Earliest_Start_Date",
        "Earliest start date",
        "Earl.start date",
    ];

    let latest_allowed_finish_date_possible_headers = [
        "WO_Latest_Allowed_Finish_Date",
        "Latst Allowd.FinDate",
        "Latest Allowed Finish Date",
    ];

    let basic_start_possible_headers = [
        "WO_Basic_Start_Date",
        "Earliest start date",
        "Basic_Start_Date",
        "Basic Start Date",
        "Bas. start date",
    ];
    let basic_finish_possible_headers = [
        "WO_Basic_End_Date",
        "Basic_Finish_Date",
        "Basic Finish Date",
        "Basic fin. date",
    ];

    // let earliest_start_time_possible_headers = ["Earliest_Start_Time", "Earliest start time"];

    let earliest_allowed_start_date_data = get_data_from_headers(
        row,
        header_to_index,
        &earliest_allowed_start_date_possible_headers,
    );
    let latest_allowed_finish_date_data = get_data_from_headers(
        row,
        header_to_index,
        &latest_allowed_finish_date_possible_headers,
    );
    let basic_start_data =
        get_data_from_headers(row, header_to_index, &basic_start_possible_headers);
    let basic_finish_data =
        get_data_from_headers(row, header_to_index, &basic_finish_possible_headers);

    let earliest_allowed_start_date = match earliest_allowed_start_date_data.cloned() {
        Some(calamine::Data::DateTimeIso(s)) => {
            match s.parse::<DateTime<Utc>>() {
                Ok(date_time) => {
                    // Now that we have a `DateTime<Utc>`, we can get a `NaiveDate`
                    Ok(date_time.naive_utc().date())
                }
                Err(_e) => {
                    // Handle the error, maybe return it or log it
                    event!(
                        tracing::Level::ERROR,
                        "Could not parse earliest_start_date_data as date"
                    );

                    let error_message = "Could not parse earliest_start_date_data as date";
                    Err(Error::Msg(error_message))
                }
            }
            .unwrap()
        }
        Some(calamine::Data::DateTime(s)) => {
            let start = NaiveDate::from_ymd_opt(1900, 1, 1).expect("DATE");
            let date = start.checked_add_signed(Duration::days(s.as_f64() as i64 - 2));
            date.unwrap()
        }
        Some(calamine::Data::String(s)) => parse_date(&s),
        Some(calamine::Data::Float(s)) => parse_date(&s.to_string()),
        Some(calamine::Data::Int(s)) => parse_date(&s.to_string()),
        Some(calamine::Data::Empty) => {
            panic!("Earliest start date is empty");
        }
        _ => {
            event!(tracing::Level::ERROR, "Could not parse earliest_start_date");
            let error_message = "Could not parse earliest_start_date_data as anything";
            return Err(Error::Msg(error_message));
        }
    };
    let latest_allowed_finish_date = match latest_allowed_finish_date_data.cloned() {
        Some(calamine::Data::String(s)) => parse_date(&s),
        Some(calamine::Data::DateTime(s)) => {
            let start = NaiveDate::from_ymd_opt(1900, 1, 1).expect("DATE");
            let date = start.checked_add_signed(Duration::days(s.as_f64() as i64 - 2));
            date.unwrap()
        }
        Some(calamine::Data::Empty) => NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),

        Some(calamine::Data::Float(s)) => {
            let var_name = s.to_string();
            DATS(var_name).into()
        }
        _ => {
            return Err(Error::Msg(
                "Could not parse latest_allowed_finish_date_data as string",
            ))
        }
    };

    let basic_start_date_naive = match basic_start_data.cloned() {
        Some(calamine::Data::String(s)) => parse_date(&s),
        Some(calamine::Data::DateTime(datetime)) => {
            let start = NaiveDate::from_ymd_opt(1900, 1, 1).expect("DATE");
            start
                .checked_add_signed(Duration::days(datetime.as_f64() as i64))
                .unwrap()
        }
        Some(calamine::Data::Float(s)) => {
            let var_name = s.to_string();
            DATS(var_name).into()
        }
        Some(_) => panic!("Could not parse basic_start_data as string"),
        None => panic!("Basic start date is None"),
    };
    let basic_start_date = basic_start_date_naive
        .and_hms_opt(7, 0, 0)
        .unwrap()
        .and_utc();

    let basic_finish_date = match basic_finish_data.cloned() {
        Some(calamine::Data::String(s)) => parse_date(&s),
        Some(calamine::Data::DateTime(datetime)) => {
            let start = NaiveDate::from_ymd_opt(1900, 1, 1).expect("DATE");
            start
                .checked_add_signed(Duration::days(datetime.as_f64() as i64))
                .unwrap()
        }
        Some(calamine::Data::Float(s)) => {
            let var_name = s.to_string();
            DATS(var_name).into()
        }
        Some(_) => panic!("Could not parse basic finish as string"),
        None => {
            warn!("basic finish date could not be parsed and is not part of the scheduling system. Setting it to Basic Start.");
            basic_start_date_naive
        }
    }
    .and_hms_opt(7, 0, 0)
    .unwrap()
    .and_utc();

    let duration = basic_finish_date - basic_start_date;

    Ok(WorkOrderDates {
        earliest_allowed_start_date: DateTime::<Utc>::from_naive_utc_and_offset(
            earliest_allowed_start_date
                .clone()
                .and_hms_opt(7, 0, 0)
                .unwrap(),
            Utc,
        ),
        latest_allowed_finish_date: DateTime::<Utc>::from_naive_utc_and_offset(
            latest_allowed_finish_date
                .clone()
                .and_hms_opt(7, 0, 0)
                .unwrap(),
            Utc,
        ),
        earliest_allowed_start_period: date_to_period(
            periods,
            &DateTime::<Utc>::from_naive_utc_and_offset(
                earliest_allowed_start_date.and_hms_opt(7, 0, 0).unwrap(),
                Utc,
            ),
        ),
        latest_allowed_finish_period: date_to_period(
            periods,
            &DateTime::<Utc>::from_naive_utc_and_offset(
                latest_allowed_finish_date.and_hms_opt(7, 0, 0).unwrap(),
                Utc,
            ),
        ),
        basic_start_date,
        basic_finish_date,
        duration,
        basic_start_scheduled: None,
        basic_finish_scheduled: None,
        material_expected_date: None,
    })
}

fn extract_revision(
    row: &[calamine::Data],
    header_to_index: &HashMap<String, usize>,
) -> Result<Revision, Error> {
    let string = match row
        .get(
            *header_to_index
                .get("WO_Revision")
                .ok_or("Revision header not found")?,
        )
        .cloned()
    {
        Some(calamine::Data::String(s)) => s,
        Some(calamine::Data::Empty) => String::from("Empty"),
        _ => {
            dbg!(row);
            return Err(Error::Msg("Could not parse revision as string"));
        }
    };

    let shutdown_pattern = r"NOSD|NE";

    let shutdown = Regex::new(shutdown_pattern).unwrap();
    let shutdown = !shutdown.is_match(&string);

    Ok(Revision { string, shutdown })
}

fn extract_unloading_point(
    row: &[calamine::Data],
    header_to_index: &HashMap<String, usize>,
    periods: &[Period],
) -> Result<UnloadingPoint, Error> {
    let unloading_point_possible_headers =
        ["OPR_Scheduled_Work", "Unloading_Point", "Unloading Point"];

    let unloading_point_data =
        get_data_from_headers(row, header_to_index, &unloading_point_possible_headers);

    let unloading_point_string = match unloading_point_data.cloned() {
        Some(calamine::Data::String(s)) => s,
        Some(calamine::Data::Int(n)) => n.to_string(),
        Some(calamine::Data::Float(n)) => n.to_string(),
        Some(calamine::Data::Bool(b)) => b.to_string(),
        Some(calamine::Data::Error(e)) => e.to_string(),
        Some(calamine::Data::Empty) => String::from("Empty"),
        None => String::from("None"),
        _ => return Err(Error::Msg("Could not parse unloading point as string")),
    };

    let start_year_and_weeks = extract_year_and_weeks(&unloading_point_string);

    Ok(UnloadingPoint {
        string: unloading_point_string.clone(),
        period: periods
            .iter()
            .find(|&period| {
                if start_year_and_weeks.0.is_some() {
                    period.year == start_year_and_weeks.0.unwrap() + 2000
                        && (period.start_week == start_year_and_weeks.1.unwrap_or(0)
                            || period.end_week == start_year_and_weeks.1.unwrap_or(0))
                } else {
                    period.start_week == start_year_and_weeks.1.unwrap_or(0)
                        || period.end_week == start_year_and_weeks.1.unwrap_or(0)
                }
            })
            .cloned(),
    })
}

fn extract_year_and_weeks(input_string: &str) -> (Option<i32>, Option<u32>, Option<u32>) {
    let re = regex::Regex::new(r"(\d{2})?-?[W|w](\d+)-?[W|w]?(\d+)").unwrap();
    let captures = re.captures(input_string);

    match captures {
        Some(cap) => (
            cap.get(1).map_or("", |m| m.as_str()).parse().ok(),
            cap.get(2).map_or("", |m| m.as_str()).parse().ok(),
            cap.get(3).map_or("", |m| m.as_str()).parse().ok(),
        ),
        None => (None, None, None),
    }
}

fn extract_functional_location(
    row: &[calamine::Data],
    header_to_index: &HashMap<String, usize>,
) -> Result<FunctionalLocation, Error> {
    let functional_location_possible_headers = [
        "FLOC_Name",
        "Functional Loc.",
        "functional_location",
        "Functional Location",
    ];

    let functional_location_data =
        get_data_from_headers(row, header_to_index, &functional_location_possible_headers);

    let string = functional_location_data.cloned();

    match string {
        Some(s) => match s {
            calamine::Data::String(s) => {
                let asset = Asset::new_from_string(s[0..2].to_string());
                Ok(FunctionalLocation { string: s, asset })
            }
            calamine::Data::Empty => Ok(FunctionalLocation {
                string: "None".to_string(),
                asset: Asset::Unknown,
            }),
            _ => Err(Error::Msg("Could not parse functional location as string")),
        },
        None => Ok(FunctionalLocation {
            string: "None".to_string(),
            asset: Asset::Unknown,
        }),
    }
}

fn extract_order_text(
    row: &[calamine::Data],
    header_to_index: &HashMap<String, usize>,
) -> Result<WorkOrderText, Error> {
    let notes_1_possible_headers = ["Notes_1", "notes_1", "Notes 1"];
    let notes_2_possible_headers = ["Notes_2", "Notes 2", "Notes_2"];
    let description_1_possible_headers = [
        "WO_Header_Description",
        "Object Description",
        "Description_1",
        "Description 1",
        "Description_1",
        "Description",
    ];
    let description_2_possible_headers = [
        "WO_Header_Description",
        "Order Description",
        "Description_2",
        "Description 2",
        "Description",
    ];
    let operation_description_possible_headers = [
        "OPR_Description",
        "Short_Text",
        "Operation Description",
        "Opr. short text",
        "Operation Description",
    ];
    let system_status_possible_headers = [
        "WO_I_Status_Code",
        "System_Status",
        "System Status",
        "Order System Status",
        "Op.SystemStatus",
    ];
    let user_status_possible_headers = [
        "WO_E_Status_Code",
        "User_Status",
        "User Status",
        "Order User Status",
        "User status",
    ];

    let notes_1_data = get_data_from_headers(row, header_to_index, &notes_1_possible_headers);
    let notes_2_data = get_data_from_headers(row, header_to_index, &notes_2_possible_headers);
    let description_1_data =
        get_data_from_headers(row, header_to_index, &description_1_possible_headers);
    let description_2_data =
        get_data_from_headers(row, header_to_index, &description_2_possible_headers);
    let operation_description_data = get_data_from_headers(
        row,
        header_to_index,
        &operation_description_possible_headers,
    );
    let system_status_data =
        get_data_from_headers(row, header_to_index, &system_status_possible_headers);
    let user_status_data =
        get_data_from_headers(row, header_to_index, &user_status_possible_headers);

    let notes_1 = match notes_1_data.cloned() {
        Some(calamine::Data::String(s)) => s.to_string(),
        None => "Notes 1 is not part of the inputed data".to_string(),
        _ => return Err(Error::Msg("Could not parse notes_1 as string")),
    };

    let notes_2 = match notes_2_data {
        Some(calamine::Data::String(s)) => s.parse::<u64>().unwrap_or(8),
        Some(calamine::Data::Int(n)) => *n as u64,
        None => 5,
        _ => {
            debug!("Could not parse notes_2 as an integer {}", line!());
            return Err(Error::Msg("Could not parse notes_2 as an integer {}"));
        }
    };

    let object_description = match description_2_data.cloned() {
        Some(calamine::Data::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse object_description as string")),
    };

    let order_description = match description_1_data.cloned() {
        Some(calamine::Data::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse order_description as string")),
    };

    let operation_description = match operation_description_data.cloned() {
        Some(calamine::Data::String(s)) => s,
        _ => {
            event!(
                tracing::Level::INFO,
                "operation_description is not a string"
            );
            "operation_description_not_present".to_string()
        }
    };

    let order_system_status = match system_status_data.cloned() {
        Some(calamine::Data::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse order_system_status as string")),
    };

    let order_user_status = match user_status_data.cloned() {
        Some(calamine::Data::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse order_user_status as string")),
    };

    Ok(WorkOrderText {
        order_system_status,
        order_user_status,
        order_description,
        operation_description,
        object_description,
        notes_1,
        notes_2,
    })
}

fn date_to_period(periods: &[Period], date: &DateTime<Utc>) -> Period {
    let period: Option<Period> = periods
        .iter()
        .find(|period| period.start_date() <= date && period.end_date() >= date)
        .cloned();

    match period {
        Some(period) => period,
        None => {
            let mut first_period = periods.first().unwrap().clone();
            let mut counter = 0;
            loop {
                counter += 1;
                first_period = first_period - Duration::weeks(2);
                if first_period.start_date() <= date && first_period.end_date() >= date {
                    break;
                }
                if counter >= 1000 {
                    break;
                };
            }
            first_period.clone()
        }
    }
}

fn parse_date(s: &str) -> NaiveDate {
    let formats = ["%Y%m%d", "%Y-%m-%d", "%d/%m/%Y", "%d-%m-%Y", "%d.%m.%Y"];

    for format in &formats {
        match NaiveDate::parse_from_str(s, format) {
            Ok(naive_date) => return naive_date,
            Err(_) => continue,
        }
    }

    println!("Could not parse date from string: {}", s);
    NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
}

fn get_data_from_headers<'a>(
    row: &'a [calamine::Data],
    header_to_index: &HashMap<String, usize>,
    headers: &[&str],
) -> Option<&'a calamine::Data> {
    for &header in headers {
        if let Some(&index) = header_to_index.get(header) {
            if let Some(data) = row.get(index) {
                return Some(data);
            }
        }
    }
    None
}

fn create_periods(number_of_periods: u64) -> Result<Vec<Period>, Error> {
    let mut periods: Vec<Period> = Vec::<Period>::new();
    let mut start_date = Utc::now();

    // Get the ISO week number
    let week_number = start_date.iso_week().week();
    // Determine target week number: If current is even, target is the previous odd
    let target_week = if week_number % 2 == 0 {
        week_number - 1
    } else {
        week_number
    };

    // Compute the offset in days to reach Monday of the target week
    let days_to_offset = (start_date.weekday().num_days_from_monday() as i64)
        + (7 * (week_number - target_week) as i64);

    start_date -= Duration::days(days_to_offset);

    start_date = start_date
        .with_hour(0)
        .and_then(|d| d.with_minute(0))
        .and_then(|d| d.with_second(0))
        .and_then(|d| d.with_nanosecond(0))
        .unwrap();

    let mut end_date = start_date + Duration::weeks(2);

    end_date -= Duration::days(1);

    end_date = end_date
        .with_hour(23)
        .and_then(|d| d.with_minute(59))
        .and_then(|d| d.with_second(59))
        .and_then(|d| d.with_nanosecond(0))
        .unwrap();

    let mut period = Period::new(0, start_date, end_date);
    periods.push(period.clone());
    for _ in 1..number_of_periods {
        period = period + Duration::weeks(2);
        periods.push(period.clone());
    }
    Ok(periods)
}

fn excel_time_to_hh_mm_ss(serial_time: f64) -> NaiveTime {
    let total_seconds: u32 = (serial_time * 24.0 * 3600.0).round() as u32;
    let hours: u32 = total_seconds / 3600;
    let minutes: u32 = (total_seconds % 3600) / 60;
    let seconds: u32 = total_seconds % 60;

    NaiveTime::from_hms_opt(hours, minutes, seconds)
        .expect("Could not convert excel time to NaiveTime")
}

fn extract_order_type_and_priority(
    work_order_type_data: Option<&Data>,
    priority: Priority,
) -> WorkOrderType {
    match work_order_type_data.cloned() {
        Some(calamine::Data::String(work_order_type)) => match work_order_type.as_str() {
            "WDF" => match &priority {
                Priority::IntValue(value) => match value {
                    1 => Ok(WorkOrderType::Wdf(WDFPriority::One)),
                    2 => Ok(WorkOrderType::Wdf(WDFPriority::Two)),
                    3 => Ok(WorkOrderType::Wdf(WDFPriority::Three)),
                    4 => Ok(WorkOrderType::Wdf(WDFPriority::Four)),
                    _ => Ok(WorkOrderType::Other),
                },
                _ => Err(ExcelLoadError("Could not parse WDF priority as int".into())),
            },
            "WGN" => match &priority {
                Priority::IntValue(value) => match value {
                    1 => Ok(WorkOrderType::Wgn(WGNPriority::One)),
                    2 => Ok(WorkOrderType::Wgn(WGNPriority::Two)),
                    3 => Ok(WorkOrderType::Wgn(WGNPriority::Three)),
                    4 => Ok(WorkOrderType::Wgn(WGNPriority::Four)),
                    _ => Ok(WorkOrderType::Other),
                },
                _ => Err(ExcelLoadError("Could not parse WGN priority as int".into())),
            },
            "WPM" => match &priority {
                Priority::StringValue(value) => match value.as_str() {
                    "A" => Ok(WorkOrderType::Wpm(WPMPriority::A)),
                    "B" => Ok(WorkOrderType::Wpm(WPMPriority::B)),
                    "C" => Ok(WorkOrderType::Wpm(WPMPriority::C)),
                    "D" => Ok(WorkOrderType::Wpm(WPMPriority::D)),
                    _ => Ok(WorkOrderType::Other),
                },
                _ => Err(ExcelLoadError("Could not parse WPM priority as int".into())),
            },
            "WRO" => match &priority {
                Priority::IntValue(value) => match value {
                    1 => Ok(WorkOrderType::Wro(WROPriority::One)),
                    2 => Ok(WorkOrderType::Wro(WROPriority::Two)),
                    3 => Ok(WorkOrderType::Wro(WROPriority::Three)),
                    4 => Ok(WorkOrderType::Wro(WROPriority::Four)),
                    _ => Ok(WorkOrderType::Other),
                },
                _ => Err(ExcelLoadError("Could not parse WRO priority as int".into())),
            },

            _ => Ok(WorkOrderType::Other),
        },
        // Some(calamine::Data::Int(work_order_type)) => match work_order_type.as_str() {
        //     "WDF" => match &priority {
        //         Priority::IntValue(value) => match value {
        //             1 => Ok(WorkOrderType::Wdf(WDFPriority::One)),
        //             2 => Ok(WorkOrderType::Wdf(WDFPriority::Two)),
        //             3 => Ok(WorkOrderType::Wdf(WDFPriority::Three)),
        //             4 => Ok(WorkOrderType::Wdf(WDFPriority::Four)),
        //             _ => Ok(WorkOrderType::Other),
        //         },
        //         _ => Err(ExcelLoadError("Could not parse WDF priority as int".into())),
        //     },
        //     "WGN" => match &priority {
        //         Priority::IntValue(value) => match value {
        //             1 => Ok(WorkOrderType::Wgn(WGNPriority::One)),
        //             2 => Ok(WorkOrderType::Wgn(WGNPriority::Two)),
        //             3 => Ok(WorkOrderType::Wgn(WGNPriority::Three)),
        //             4 => Ok(WorkOrderType::Wgn(WGNPriority::Four)),
        //             _ => Ok(WorkOrderType::Other),
        //         },
        //         _ => Err(ExcelLoadError("Could not parse WGN priority as int".into())),
        //     },
        //     "WPM" => match &priority {
        //         Priority::StringValue(value) => match value.as_str() {
        //             "A" => Ok(WorkOrderType::Wpm(WPMPriority::A)),
        //             "B" => Ok(WorkOrderType::Wpm(WPMPriority::B)),
        //             "C" => Ok(WorkOrderType::Wpm(WPMPriority::C)),
        //             "D" => Ok(WorkOrderType::Wpm(WPMPriority::D)),
        //             _ => Ok(WorkOrderType::Other),
        //         },
        //         _ => Err(ExcelLoadError("Could not parse WPM priority as int".into())),
        //     },
        //     "WRO" => match &priority {
        //         Priority::IntValue(value) => match value {
        //             1 => Ok(WorkOrderType::Wro(WROPriority::One)),
        //             2 => Ok(WorkOrderType::Wro(WROPriority::Two)),
        //             3 => Ok(WorkOrderType::Wro(WROPriority::Three)),
        //             4 => Ok(WorkOrderType::Wro(WROPriority::Four)),
        //             _ => Ok(WorkOrderType::Other),
        //         },
        //         _ => Err(ExcelLoadError("Could not parse WRO priority as int".into())),
        //     },

        //     _ => Ok(WorkOrderType::Other),
        // },
        None => Ok(WorkOrderType::Other),
        _ => Err(ExcelLoadError(
            "Could not parse work order type as int".into(),
        )),
    }
    .expect("Could not parse order type")
}

#[derive(Debug)]
struct ExcelLoadError(String);

impl fmt::Display for ExcelLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ExcelLoadError: {}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_parse_date() {
        let date = parse_date("2021-01-01");
        assert_eq!(date, NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
    }

    #[test]
    fn test_date_to_period() {
        let periods = vec![
            Period::from_str("2023-W1-2").unwrap(),
            Period::from_str("2023-W3-4").unwrap(),
        ];

        let date: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 1, 10, 7, 0, 0).unwrap();
        assert_eq!(date_to_period(&periods, &date), periods[0].clone());
    }

    #[test]
    fn test_create_scheduling_environment() {
        let file_path = Path::new("test_data/input-ordinator-complete-2024-04-10.xlsx");
        let number_of_strategic_periods = 26;
        let number_of_tactical_periods = 4;
        let number_of_days = 56;

        let total_excel: TotalExcel = TotalExcel {
            file_path,
            number_of_strategic_periods,
            number_of_tactical_periods,
            number_of_days,
        };

        let scheduling_environment =
            SchedulingEnvironment::create_scheduling_environment(total_excel.clone());

        assert_eq!(
            scheduling_environment
                .unwrap()
                .clone_strategic_periods()
                .len(),
            number_of_strategic_periods as usize
        );

        let scheduling_environment =
            SchedulingEnvironment::create_scheduling_environment(total_excel);

        let number_of_work_orders = scheduling_environment
            .as_ref()
            .unwrap()
            .work_orders()
            .inner
            .len();
        let number_of_operations = scheduling_environment
            .as_ref()
            .unwrap()
            .work_orders()
            .inner
            .get(&WorkOrderNumber(2100106943))
            .unwrap()
            .operations()
            .len();

        assert_eq!(number_of_work_orders, 11288);
        assert_eq!(number_of_operations, 3);
    }
}
