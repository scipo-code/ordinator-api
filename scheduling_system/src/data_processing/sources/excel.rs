use calamine::{Data, Error, Reader, Xlsx};
use core::{fmt, num};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use tracing::{event, info, warn};

use crate::models::time_environment::period::Period;
use crate::models::time_environment::TimeEnvironment;
use crate::models::work_order::system_condition::SystemCondition;

use crate::models::work_order::functional_location::FunctionalLocation;
use crate::models::work_order::order_dates::OrderDates;
use crate::models::work_order::order_text::OrderText;
use crate::models::work_order::order_type::{WDFPriority, WGNPriority, WPMPriority};
use crate::models::work_order::order_type::{WROPriority, WorkOrderType};
use crate::models::work_order::priority::Priority;
use crate::models::work_order::revision::Revision;
use crate::models::work_order::status_codes::{MaterialStatus, StatusCodes};
use crate::models::work_order::unloading_point::UnloadingPoint;
use crate::models::work_order::{ActivityRelation, WorkOrder};
use crate::models::worker_environment::WorkerEnvironment;
use crate::models::{SchedulingEnvironment, WorkOrders};
use chrono::{
    naive, DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone, Timelike, Utc,
    Weekday,
};
use shared_messages::resources::Resources;

extern crate regex;

use crate::models::work_order::operation::Operation;

#[derive(Debug)]
struct ExcelLoadError(String);

impl fmt::Display for ExcelLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ExcelLoadError: {}", self.0)
    }
}

/// This function will load data from excel. It is crucial that the approach is modular and scalable
/// so that it will always be possible to add new data sources and data transformers in the future.
///
pub fn load_data_file(
    file_path: &Path,
    number_of_periods: u32,
    number_of_days: u32,
) -> Result<SchedulingEnvironment, calamine::Error> {
    let mut workbook: Xlsx<_> = calamine::open_workbook(file_path)?;
    println!("Successfully loaded file.");

    let sheet: &calamine::Range<calamine::Data> = &workbook
        .worksheet_range_at(0)
        .ok_or(calamine::Error::Msg("Cannot find work order sheet"))?
        .expect("Could not load work order sheet.");

    let mut work_orders: WorkOrders = WorkOrders::new();
    let worker_environment: WorkerEnvironment = WorkerEnvironment::new();

    let strategic_periods: Vec<Period> = create_periods(number_of_periods).unwrap_or_else(|_| {
        panic!(
            "Could not create periods in {} at line {}",
            file!(),
            line!()
        )
    });

    let tactical_days = |number_of_days: u32| -> Vec<NaiveDate> {
        let mut days: Vec<NaiveDate> = Vec::new();
        let mut date = Local::now().date_naive();

        for _ in 0..number_of_days {
            days.push(date);
            date = date.succ_opt().unwrap();
        }
        days
    };

    populate_work_orders(&mut work_orders, &strategic_periods, sheet)
        .expect("could not populate the work orders");

    let time_environment = TimeEnvironment::new(strategic_periods, tactical_days(number_of_days));

    let scheduling_environment =
        SchedulingEnvironment::new(work_orders, worker_environment, time_environment);
    Ok(scheduling_environment)
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

    for row in sheet.rows().skip(1) {
        let mut work_order_number: u32 = 0;
        match row[5] {
            Data::Empty => continue,
            _ => (),
        }

        if let Some(&index) = header_to_index.get("Order") {
            if index < row.len() {
                let value = &row[index];
                match value {
                    calamine::Data::Empty => continue,
                    calamine::Data::String(s) => match s.parse::<u32>() {
                        Ok(n) => work_order_number = n,
                        Err(e) => {
                            println!("Could not parse work order number as string: {}", e)
                        }
                    },
                    calamine::Data::Int(s) => work_order_number = *s as u32,
                    calamine::Data::Float(s) => work_order_number = *s as u32,
                    _ => {
                        todo!("Handle other cases of calamine::Data");
                    }
                }
            }
        }

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
/// The fact that I want to extend this means that we should initialize the work order with a default value.
/// This means that the WorkOrder type should receive a new method, that will create a new
/// instance that can then be used to populate the work_orders HashMap.
///
/// The operations field is a little more complex as we could have multiple different rows that
///
/// The operations field is a little more complex as we could have multiple different rows that
/// write to the same work order. This means that we need to check if the work order already exists
///
///
/// The problem is to find the right approach that makes the function work for both work
///
/// Maybe we should just initialize the operations as empty here and then simply always run the
///
/// Maybe we should just initialize the operations as empty here and then simply always run the
/// operation reading on each row! Yes that is the approach that I want to take.
fn create_new_work_order(
    row: &[calamine::Data],
    header_to_index: &HashMap<String, usize>,
    periods: &[Period],
) -> Result<WorkOrder, Error> {
    let work_order_type_possible_headers = ["Order Type", "Order_Type"];

    let work_order_type_data =
        get_data_from_headers(row, header_to_index, &work_order_type_possible_headers);

    let priority = match row
        .get(
            *header_to_index
                .get("Priority")
                .ok_or("Priority header not found")?,
        )
        .cloned()
    {
        Some(calamine::Data::Int(n)) => Priority::IntValue(n as u32),
        Some(calamine::Data::String(s)) => {
            match s.parse::<u32>() {
                Ok(num) => Priority::IntValue(num), // If successful, use the integer value
                Err(_) => Priority::StringValue(s), // If not, fall back to using the string
            }
        }
        Some(calamine::Data::Float(n)) => Priority::IntValue(n as u32),
        _ => Priority::StringValue(String::new()),
    };

    Ok(WorkOrder::new(
        match row
            .get(
                *header_to_index
                    .get("Order")
                    .ok_or("Order header not found")?,
            )
            .cloned()
        {
            Some(calamine::Data::Int(n)) => n as u32,
            Some(calamine::Data::Float(n)) => n as u32,
            Some(calamine::Data::String(s)) => s.parse::<u32>().unwrap_or(0),
            _ => 0,
        },
        false,
        0,
        priority.clone(),
        0.0,
        HashMap::<u32, Operation>::new(),
        HashMap::<Resources, f64>::new(),
        Vec::<ActivityRelation>::new(),
        extract_order_type_and_priority(work_order_type_data, priority),
        SystemCondition::new(),
        extract_status_codes(row, header_to_index).expect("Failed to extract StatusCodes"),
        extract_order_dates(row, header_to_index, periods).expect("Failed to extract OrderDates"),
        extract_revision(row, header_to_index).expect("Failed to extract Revision"),
        extract_unloading_point(row, header_to_index, periods)
            .expect("Failed to extract UnloadingPoint"),
        extract_functional_location(row, header_to_index)
            .expect("Failed to extract FunctionalLocation"),
        extract_order_text(row, header_to_index).expect("Failed to extract OrderText"),
        false,
    ))
}

fn create_new_operation(
    row: &[calamine::Data],
    header_to_index: &HashMap<String, usize>,
) -> Result<Operation, Error> {
    let default_future_date = Utc.with_ymd_and_hms(2026, 1, 1, 7, 0, 0).unwrap();

    let work_possible_headers = ["Remaining Work", "Work_Remaining", "Work_Planned", "Work"];
    let earliest_start_date_headers = [
        "Earliest_Start_Date",
        "Earliest start date",
        "Earliest StrDate",
    ];
    let earliest_start_time_headers = ["Earliest start time", "Earliest_Start_Time"];
    let earliest_finish_date_headers = [
        "Earliest_Finish_Date",
        "Earliest_End_Date",
        "Earliest finish date",
        "Earliest end date",
    ];
    let earliest_finish_time_headers = [
        "Earliest_Finish_Time",
        "Latest_Finish_Time",
        "Earliest finish time",
    ];
    let work_center_headers = [
        "Work_Center",
        "Work Center",
        "Work center",
        "Oper.WorkCenter",
    ];
    let actual_work_headers = [
        "Work_Actual",
        "Work Actual",
        "Actual work",
        "Work Actual (Hrs)",
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

    Ok(Operation {
        activity: match row
            .get(
                *header_to_index
                    .get("Activity")
                    .ok_or("Activity header not found")?,
            )
            .cloned()
        {
            Some(calamine::Data::Int(n)) => n as u32,
            Some(calamine::Data::Float(n)) => n as u32,
            Some(calamine::Data::String(s)) => s.parse::<u32>().unwrap_or(0),
            _ => 0,
        },
        number: match row
            .get(*header_to_index.get("Number").unwrap_or(&(1 as usize)))
            .cloned()
        {
            Some(calamine::Data::Int(n)) => n as u32,
            Some(calamine::Data::Float(n)) => n as u32,
            Some(calamine::Data::String(s)) => s.parse::<u32>().unwrap_or(0),
            _ => 0,
        },
        work_center: match work_center_data.cloned() {
            Some(calamine::Data::String(s)) => Resources::new_from_string(s),
            _ => return Err(Error::Msg("Could not parse work center as string")),
        },
        preparation_time: 0.0,
        work_remaining: match work_remaining_data.cloned() {
            Some(calamine::Data::Int(n)) => n as f64,
            Some(calamine::Data::Float(n)) => n as f64,
            Some(calamine::Data::String(s)) => s.parse::<f64>().unwrap_or(0.0),
            _ => 100000.0,
        },
        work_performed: match actual_work_data.cloned() {
            Some(calamine::Data::Int(n)) => n as f64,
            Some(calamine::Data::Float(n)) => n as f64,
            Some(calamine::Data::String(s)) => s.parse::<f64>().unwrap_or(0.0),
            _ => 0.0,
        },
        work_adjusted: 0.0,
        operating_time: 0.0,
        duration: match header_to_index.get("Duration") {
            Some(index) => match row.get(*index).cloned() {
                Some(calamine::Data::Int(n)) => n as u32,
                Some(calamine::Data::Float(n)) => n as u32,
                Some(calamine::Data::String(s)) => {
                    s.parse::<u32>().expect("Duration is not a valid number")
                }
                _ => 0,
            },
            None => 0,
        },
        possible_start: default_future_date,
        target_finish: default_future_date,
        earliest_start_datetime: {
            let date = match earliest_start_date_data.cloned() {
                Some(calamine::Data::String(s)) => parse_date(&s),
                Some(calamine::Data::DateTime(s)) => {
                    let start = NaiveDate::from_ymd_opt(1900, 1, 1).expect("DATE");
                    let date = start.checked_add_signed(Duration::days(s.as_f64() as i64 - 2));
                    date.unwrap()
                }
                _ => return Err(Error::Msg("Could not parse Earliest_Start_Date as string")),
            };

            let time = match earliest_start_time_data.cloned() {
                Some(calamine::Data::String(s)) => {
                    match NaiveTime::parse_from_str(&s, "%H:%M:%s") {
                        Ok(naive_date) => naive_date,
                        Err(_) => {
                            println!(
                                "Could not parse earliest_start_time_data from string: {}",
                                s
                            );
                            NaiveTime::from_hms_opt(7, 0, 0).unwrap()
                        }
                    }
                }
                Some(calamine::Data::DateTime(s)) => excel_time_to_hh_mm_ss(s.as_f64() as f64),
                _ => {
                    event!(
                        tracing::Level::WARN,
                        "Could not parse earliest_start_time is not present"
                    );
                    NaiveTime::from_hms_opt(7, 0, 0).unwrap()
                }
            };

            Utc.from_utc_datetime(&naive::NaiveDateTime::new(date, time))
        },
        earliest_finish_datetime: {
            let date = match earliest_finish_date_data.cloned() {
                Some(calamine::Data::String(s)) => parse_date(&s),
                Some(calamine::Data::DateTime(s)) => {
                    let start = NaiveDate::from_ymd_opt(1900, 1, 1).expect("DATE");
                    let date = start.checked_add_signed(Duration::days(s.as_f64() as i64 - 2));
                    date.unwrap()
                }

                _ => {
                    info!("Could not earliest_finish_date_data as string");
                    NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
                }
            };

            let time = match earliest_finish_time_data.cloned() {
                Some(calamine::Data::String(s)) => {
                    match NaiveTime::parse_from_str(&s, "%H:%M:%s") {
                        Ok(naive_date) => naive_date,
                        Err(_) => {
                            println!(
                                "Could not parse earliest_finish_time_data from string: {}",
                                s
                            );
                            NaiveTime::from_hms_opt(7, 0, 0).unwrap()
                        }
                    }
                }
                Some(calamine::Data::DateTime(s)) => excel_time_to_hh_mm_ss(s.as_f64()),
                _ => NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            };
            Utc.from_utc_datetime(&naive::NaiveDateTime::new(date, time))
        },
    })
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
    ];
    let user_status_possible_headers = [
        "User_Status",
        "User Status",
        "Order User Status",
        "User status",
    ];
    let op_status_possible_headers = ["Opr_User_Status", "Op User Status", "Oper.UserStatus"];

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
) -> Result<OrderDates, Error> {
    let earliest_allowed_start_date_possible_headers = [
        "Earliest Allowed Start Date",
        "Earliest_Start_Date",
        "Earliest start date",
        "Earl.start date",
    ];
    let latest_allowed_finish_date_possible_headers = ["Latst Allowd.FinDate"];
    let basic_start_possible_headers = ["Basic_Start_Date", "Basic Start Date", "Bas. start date"];
    let basic_finish_possible_headers =
        ["Basic_Finish_Date", "Basic Finish Date", "Basic fin. date"];

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
    dbg!(latest_allowed_finish_date_data.clone());
    let latest_allowed_finish_date = match latest_allowed_finish_date_data.cloned() {
        Some(calamine::Data::String(s)) => parse_date(&s),
        Some(calamine::Data::DateTime(s)) => {
            let start = NaiveDate::from_ymd_opt(1900, 1, 1).expect("DATE");
            let date = start.checked_add_signed(Duration::days(s.as_f64() as i64 - 2));
            date.unwrap()
        }
        Some(calamine::Data::Empty) => NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        _ => {
            return Err(Error::Msg(
                "Could not parse latest_allowed_finish_date_data as string",
            ))
        }
    };

    let basic_start_date: Result<NaiveDate, Error> = match basic_start_data.cloned() {
        Some(calamine::Data::String(s)) => Ok(parse_date(&s)),
        Some(_) => Err(Error::Msg("Could not parse basic_start_data as string")),
        None => Err(Error::Msg("Basic start date is None")),
    };

    let basic_finish_date = match basic_finish_data.cloned() {
        Some(calamine::Data::String(s)) => Ok(parse_date(&s)),
        Some(_) => Err(Error::Msg("Could not parse basic finish as string")),
        None => Err(Error::Msg("Basic finish date is None")),
    };

    let basic_start_date_additional = basic_start_date
        .unwrap_or(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
        .and_hms_opt(7, 0, 0)
        .unwrap();
    let basic_finish_date_additional = basic_finish_date
        .unwrap_or(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
        .and_hms_opt(7, 0, 0)
        .unwrap();

    Ok(OrderDates {
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
            DateTime::<Utc>::from_naive_utc_and_offset(
                earliest_allowed_start_date.and_hms_opt(7, 0, 0).unwrap(),
                Utc,
            ),
        ),
        latest_allowed_finish_period: date_to_period(
            periods,
            DateTime::<Utc>::from_naive_utc_and_offset(
                latest_allowed_finish_date.and_hms_opt(7, 0, 0).unwrap(),
                Utc,
            ),
        ),
        basic_start_date: DateTime::<Utc>::from_naive_utc_and_offset(
            basic_start_date_additional,
            Utc,
        ),
        basic_finish_date: DateTime::<Utc>::from_naive_utc_and_offset(
            basic_finish_date_additional,
            Utc,
        ),
        duration: basic_finish_date_additional.signed_duration_since(basic_start_date_additional),
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
                .get("Revision")
                .ok_or("Revision header not found")?,
        )
        .cloned()
    {
        Some(calamine::Data::String(s)) => s,
        _ => return Err(Error::Msg("Could not parse revision as string")),
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
    let unloading_point_possible_headers = ["Unloading_Point", "Unloading Point"];

    let unloading_point_data =
        get_data_from_headers(row, header_to_index, &unloading_point_possible_headers);

    let string = match unloading_point_data.cloned() {
        Some(calamine::Data::String(s)) => s,
        Some(calamine::Data::Int(n)) => n.to_string(),
        Some(calamine::Data::Float(n)) => n.to_string(),
        Some(calamine::Data::Bool(b)) => b.to_string(),
        Some(calamine::Data::Error(e)) => e.to_string(),
        Some(calamine::Data::Empty) => String::from("Empty"),
        None => String::from("None"),
        _ => return Err(Error::Msg("Could not parse unloading point as string")),
    };

    let (start_week, _end_week, present) = _extract_weeks(&string);
    let start_date = _week_to_date(start_week, true);

    if present {
        Ok(UnloadingPoint {
            string,
            present,
            period: {
                Some(
                    match periods
                        .iter()
                        .find(|period| period.get_start_date() == start_date)
                    {
                        Some(period) => period.clone(),
                        None => periods.last().unwrap().clone(),
                    }
                    .clone(),
                )
            },
        })
    } else {
        Ok(UnloadingPoint {
            string,
            present,
            period: None,
        })
    }
}

fn _week_to_date(week_number: u32, start_of_week: bool) -> DateTime<Utc> {
    let today_date = chrono::Local::now().naive_local();
    let current_year = today_date.year();
    let current_week = today_date.iso_week().week();

    // Determine the target year based on the week number and current date
    let target_year = if week_number >= current_week {
        current_year
    } else {
        current_year + 1
    };

    // Compute the date corresponding to the start of the target week (Monday)
    let new_year_date = NaiveDate::from_ymd_opt(target_year, 1, 1); // January 1st of the target year
    let first_week_day = new_year_date.unwrap().weekday();
    let offset: Duration = if first_week_day.num_days_from_sunday()
        <= Weekday::Mon.num_days_from_sunday()
    {
        Duration::days(
            (Weekday::Mon.num_days_from_sunday() - first_week_day.num_days_from_sunday()) as i64,
        )
    } else {
        Duration::days(
            (7 - (first_week_day.num_days_from_sunday() - Weekday::Mon.num_days_from_sunday()))
                as i64,
        )
    };

    let start_date = new_year_date.unwrap() + offset + Duration::weeks(week_number as i64 - 1);
    let time = NaiveTime::from_hms_opt(0, 0, 0);
    let naive_datetime = start_date.and_time(time.unwrap());
    let start_datetime = Utc.from_utc_datetime(&naive_datetime);
    if start_of_week {
        start_datetime
    } else {
        start_datetime + Duration::days(6)
    }
}

fn _extract_weeks(input_string: &str) -> (u32, u32, bool) {
    let re = regex::Regex::new(r"W(\d+)-(\d+)").unwrap();
    let captures = re.captures(input_string);

    if let Some(cap) = captures {
        let start_week = cap[1].parse::<u32>().unwrap_or(0);
        let end_week = cap[2].parse::<u32>().unwrap_or(0);
        (start_week, end_week, true)
    } else {
        (0, 0, false)
    }
}

fn extract_functional_location(
    row: &[calamine::Data],
    header_to_index: &HashMap<String, usize>,
) -> Result<FunctionalLocation, Error> {
    let functional_location_possible_headers = ["functional_location", "Functional Location"];

    let functional_location_data =
        get_data_from_headers(row, header_to_index, &functional_location_possible_headers);

    let string = functional_location_data.cloned();

    match string {
        Some(s) => match s {
            calamine::Data::String(s) => Ok(FunctionalLocation { string: s }),
            _ => Err(Error::Msg("Could not parse functional location as string")),
        },
        None => Ok(FunctionalLocation {
            string: "None".to_string(),
        }),
    }
}

fn extract_order_text(
    row: &[calamine::Data],
    header_to_index: &HashMap<String, usize>,
) -> Result<OrderText, Error> {
    let notes_1_possible_headers = ["Notes_1", "notes_1", "Notes 1"];
    let notes_2_possible_headers = ["Notes_2", "Notes 2", "Notes_2"];
    let description_1_possible_headers = [
        "Object Description",
        "Description_1",
        "Description 1",
        "Description_1",
        "Description",
    ];
    let description_2_possible_headers = [
        "Order Description",
        "Description_2",
        "Description 2",
        "Description",
    ];
    let operation_description_possible_headers = [
        "Short_Text",
        "Operation Description",
        "Opr. short text",
        "Operation Description",
    ];
    let system_status_possible_headers = [
        "System_Status",
        "System Status",
        "Order System Status",
        "Op.SystemStatus",
    ];
    let user_status_possible_headers = [
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
        Some(calamine::Data::String(s)) => s.parse::<u32>().unwrap_or(8),
        Some(calamine::Data::Int(n)) => *n as u32,
        None => 5,
        _ => {
            warn!("Could not parse notes_2 as an integer {}", line!());
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

    Ok(OrderText {
        order_system_status,
        order_user_status,
        order_description,
        operation_description,
        object_description,
        notes_1,
        notes_2,
    })
}

fn date_to_period(periods: &[Period], date: DateTime<Utc>) -> Period {
    let period: Option<Period> = periods
        .iter()
        .find(|period| period.get_start_date() <= date && period.get_end_date() >= date)
        .cloned();

    match period {
        Some(period) => period,
        None => {
            let mut first_period = periods.first().unwrap().clone();
            let mut counter = 0;
            loop {
                counter += 1;
                first_period = first_period - Duration::weeks(2);
                if first_period.get_start_date() <= date && first_period.get_end_date() >= date {
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

fn create_periods(number_of_periods: u32) -> Result<Vec<Period>, Error> {
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

#[cfg(test)]
mod tests {
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
            Period::new_from_string("2023-W1-2").unwrap(),
            Period::new_from_string("2023-W3-4").unwrap(),
        ];

        let date: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 1, 10, 7, 0, 0).unwrap();
        assert_eq!(date_to_period(&periods, date), periods[0].clone());
    }

    #[test]
    fn test_load_data_file() {
        let file_path = Path::new("test_data/export.XLSX");
        let number_of_periods = 26;
        let number_of_days = 56;

        let scheduling_environment = load_data_file(file_path, number_of_periods, number_of_days);

        assert_eq!(
            scheduling_environment.unwrap().clone_periods().len(),
            number_of_periods as usize
        );

        let scheduling_environment = load_data_file(file_path, number_of_periods, number_of_days);

        let number_of_work_orders = scheduling_environment
            .as_ref()
            .unwrap()
            .get_work_orders()
            .inner
            .len();
        let number_of_operations = scheduling_environment
            .as_ref()
            .unwrap()
            .get_work_orders()
            .inner
            .get(&2100024139)
            .unwrap()
            .get_operations()
            .len();

        assert_eq!(number_of_work_orders, 1227);
        assert_eq!(number_of_operations, 12);
    }
}
