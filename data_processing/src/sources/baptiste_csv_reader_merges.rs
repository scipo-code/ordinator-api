use std::{collections::HashMap, fs, path::PathBuf, str::FromStr};

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Utc};
use serde::Deserialize;
use shared_types::scheduling_environment::{
    time_environment::{day::Day, period::Period},
    work_order::{
        self, functional_location::FunctionalLocation, operation::{
            operation_analytic::OperationAnalytic, operation_info::OperationInfo, Operation,
            OperationDates, Work,
        }, priority::Priority, revision::Revision, status_codes::{MaterialStatus, StatusCodes}, system_condition::SystemCondition, unloading_point::UnloadingPoint, work_order_dates::WorkOrderDates, work_order_text::WorkOrderText, WorkOrder, WorkOrderAnalytic, WorkOrderInfo, WorkOrderNumber
    },
    worker_environment::resources::{MainResources, Resources},
    WorkOrders,
};

use crate::sap_mapper_and_types::{DATS, TIMS};

use super::baptiste_csv_reader::{
    populate_csv_structures, ContainerType, FLOCTechnicaID, FunctionalLocationsCsv,
    OperationsStatusCsv, OperationsStatusCsvAggregated, WorkCenterCsv, WorkOperations,
    WorkOperationsCsv, WorkOrdersCsv, WorkOrdersStatusCsv, WorkOrdersStatusCsvAggregated, WBSID,
};

pub fn load_csv_data(file_path: PathBuf, periods: &[Period]) -> WorkOrders {
    let contents = fs::read_to_string(file_path).unwrap();

    let file_paths: BaptisteToml = toml::from_str(&contents).unwrap();

    let mut functional_locations_csv_container = ContainerType::HashMap(HashMap::new());
    let functional_locations_csv: &mut ContainerType<FunctionalLocationsCsv> =
        populate_csv_structures(
            file_paths.mid_functional_locations,
            &mut functional_locations_csv_container,
        )
        .expect("Could not read the csv file");

    let mut operations_status_csv_container = ContainerType::Vec(Vec::<OperationsStatusCsv>::new());
    let operations_status_csv: &mut ContainerType<OperationsStatusCsv> = populate_csv_structures(
        file_paths.mid_operations_status,
        &mut operations_status_csv_container,
    )
    .expect("Could not load the csv file");

    let mut work_center_csv_container =
        ContainerType::HashMap(HashMap::<WBSID, WorkCenterCsv>::new());
    let work_center_csv: &mut ContainerType<WorkCenterCsv> =
        populate_csv_structures(file_paths.mid_work_center, &mut work_center_csv_container)
            .expect("Could not read the csv file");

    let mut work_operations_csv_container = ContainerType::Vec(Vec::new());
    let work_operations_csv: &mut ContainerType<WorkOperationsCsv> = populate_csv_structures(
        file_paths.mid_work_operations,
        &mut work_operations_csv_container,
    )
    .expect("Could not read the csv file");

    let mut work_orders_csv_container = ContainerType::HashMap(HashMap::new());
    let work_orders_csv: &mut ContainerType<WorkOrdersCsv> =
        populate_csv_structures(file_paths.mid_work_orders, &mut work_orders_csv_container)
            .expect("Could not read the csv file");

    let mut work_orders_status_csv_container = ContainerType::Vec(Vec::new());
    let work_orders_status_csv: &mut ContainerType<WorkOrdersStatusCsv> = populate_csv_structures(
        file_paths.mid_work_orders_status,
        &mut work_orders_status_csv_container,
    )
    .expect("Could not read the csv file");

    let functional_locations = if let ContainerType::HashMap(functional_locations_csv_container) =
        functional_locations_csv
    {
        functional_locations_csv_container
    } else {
        panic!();
    };

    let operations_status =
        if let ContainerType::Vec(operations_status_csv_container) = operations_status_csv {
            operations_status_csv_container
        } else {
            panic!();
        };
    let work_center = if let ContainerType::HashMap(work_center_csv_container) = work_center_csv {
        work_center_csv_container
    } else {
        panic!();
    };
    let work_operations =
        if let ContainerType::Vec(work_operations_csv_container) = work_operations_csv {
            work_operations_csv_container
        } else {
            panic!();
        };
    let work_orders = if let ContainerType::HashMap(work_orders_csv_container) = work_orders_csv {
        work_orders_csv_container
    } else {
        panic!();
    };
    let work_orders_status =
        if let ContainerType::Vec(work_orders_status_csv_container) = work_orders_status_csv {
            work_orders_status_csv_container
        } else {
            panic!();
        };

    let work_orders_status = WorkOrdersStatusCsvAggregated::new(work_orders_status.clone());

    let operations_status = OperationsStatusCsvAggregated::new(operations_status.clone());

    let work_operations = WorkOperations::new(&work_orders, work_operations.clone());

    let work_orders_inner = create_work_orders(
        functional_locations.clone(),
        operations_status,
        periods,
        work_center.clone(),
        work_operations,
        work_orders.clone(),
        work_orders_status,
    );

    WorkOrders {
        inner: work_orders_inner,
    }
}

#[derive(Deserialize)]
struct BaptisteToml {
    mid_functional_locations: PathBuf,
    mid_operations_status: PathBuf,
    mid_secondary_locations: PathBuf,
    mid_work_center: PathBuf,
    mid_work_operations: PathBuf,
    mid_work_orders: PathBuf,
    mid_work_orders_status: PathBuf,
}

#[allow(non_snake_case)]
fn create_work_orders(
    functional_locations: HashMap<FLOCTechnicaID, FunctionalLocationsCsv>,
    operations_status: OperationsStatusCsvAggregated,
    periods: &[Period],
    work_center: HashMap<WBSID, WorkCenterCsv>,
    work_operations_csv: WorkOperations,
    work_orders: HashMap<WorkOrderNumber, WorkOrdersCsv>,
    work_orders_status: WorkOrdersStatusCsvAggregated,
) -> HashMap<WorkOrderNumber, WorkOrder> {
    let mut inner_work_orders = HashMap::new();
    for (work_order_number, work_order_csv) in work_orders {
        let main_work_center: MainResources = MainResources::new_from_string(
            work_center
                .get(&work_order_csv.WO_WBS_ID)
                .unwrap()
                .WBS_Name
                .clone(),
        );

        let status_codes_string = work_orders_status
            .inner
            .get(&work_order_csv.WO_Status_ID)
            .unwrap();

        let pcnf_pattern = regex::Regex::new(r"PCNF").unwrap();
        let awsc_pattern = regex::Regex::new(r"AWSC").unwrap();
        let well_pattern = regex::Regex::new(r"WELL").unwrap();
        let sch_pattern = regex::Regex::new(r"SCH").unwrap();
        let sece_pattern = regex::Regex::new(r"SECE").unwrap();

        let material_status: MaterialStatus =
            MaterialStatus::from_status_code_string(&status_codes_string);

        let status_codes = StatusCodes {
            material_status,
            pcnf: pcnf_pattern.is_match(&status_codes_string),
            awsc: awsc_pattern.is_match(&status_codes_string),
            well: well_pattern.is_match(&status_codes_string),
            sch: sch_pattern.is_match(&status_codes_string),
            sece: sece_pattern.is_match(&status_codes_string),
            unloading_point: false, // Assuming default value; modify as needed
        };

        // self.initialize_work_load();
        // self.initialize_weight();
        // self.initialize_vendor();
        // self.initialize_material(periods);

        let work_order_analytic: WorkOrderAnalytic = WorkOrderAnalytic::new(
            0,
            Work::from(0.0),
            HashMap::new(),
            false,
            false,
            status_codes,
        );

        let earliest_allowed_start_date: NaiveDate =
            DATS(work_order_csv.WO_Earliest_Allowed_Start_Date).into();
        let latest_allowed_finish_date: NaiveDate =
            DATS(work_order_csv.WO_Latest_Allowed_Finish_Date).into();

        let basic_start: NaiveDate = DATS(work_order_csv.WO_Basic_Start_Date).into();
        let basic_finish: NaiveDate = DATS(work_order_csv.WO_Basic_End_Date).into();

        let duration = basic_finish - basic_start;

        let earliest_allowed_start_period = date_to_period(periods, &earliest_allowed_start_date);
        let latest_allowed_finish_period = date_to_period(periods, &latest_allowed_finish_date);

        let work_order_dates: WorkOrderDates = WorkOrderDates::new(
            earliest_allowed_start_date,
            latest_allowed_finish_date,
            earliest_allowed_start_period,
            latest_allowed_finish_period,
            basic_start,
            basic_finish,
            duration,
            None,
            None,
            None,
        );

        let functional_location = &functional_locations
            .get(&work_order_csv.WO_Functional_Location_Number)
            .unwrap()
            .FLOC_Name;

        let work_order_text = WorkOrderText::new(
            None,
            None,
            work_order_csv.WO_Header_Description,
            None,
            None,
            None,
            None,
        );

        let work_order_info_detail = work_order::WorkOrderInfoDetail::new(
            work_order_csv.WO_SubNetwork_ID,
            work_order_csv.WO_Plan_Maintenance_Number,
            work_order_csv.WO_Planner_Group,
            work_order_csv.WO_Maintenance_Plan_Name,
            "PM_COLLECTIVE_MISSING_TODO".to_string(),
            "ROOM_MISSING_TODO".to_string(),
        );

        let priority = Priority::new_int(``) work_order_csv.WO_Priority;

        let work_order_info: WorkOrderInfo = WorkOrderInfo::new(
            work_order_csv.WO_Priority,
            work_order_csv.WO_Order_Type,
            FunctionalLocation::new(functional_location.clone()),
            work_order_text,
            Revision::new(work_order_csv.WO_Revision),
            SystemCondition::from_str(&work_order_csv.WO_System_Condition).unwrap(),
            work_order_info_detail,
        );

        let mut operations = HashMap::new();
        for (work_order_activity, operation_csv) in &work_operations_csv.inner {
            let resources =
                Resources::from_str(&work_center.get(&operation_csv.OPR_WBS_ID).unwrap().WBS_Name);

            let unloading_point: UnloadingPoint =
                UnloadingPoint::new(operation_csv.OPR_Scheduled_Work.clone(), periods);

            let planned_work: Option<Work> = {
                let parse_option = operation_csv.OPR_Planned_Work.clone().parse::<f64>();
                match parse_option {
                    Ok(work) => Some(Work::from(work)),
                    Err(_) => None,
                }
            };

            let actual_work: Option<Work> = {
                let parse_option = operation_csv.OPR_Planned_Work.clone().parse::<f64>();
                match parse_option {
                    Ok(work) => Some(Work::from(work)),
                    Err(_) => None,
                }
            };
            let remaining_work: Option<Work> = {
                let parse_option = operation_csv.OPR_Planned_Work.clone().parse::<f64>();
                match parse_option {
                    Ok(work) => Some(Work::from(work)),
                    Err(_) => None,
                }
            };

            let operation_info = OperationInfo::new(
                operation_csv.OPR_Workers_Numbers,
                planned_work.clone(),
                actual_work,
                remaining_work,
                None,
            );

            let operation_analytic = OperationAnalytic::new(Work::from(1.0), planned_work);

            // TODO start here

            // We need to use the DATS here! I think that is the only way forward! I think that to scale this
            // we also need to be very clear on the remaining types of the system.

            let naive_start_DATS: NaiveDate = DATS(operation_csv.OPR_Start_Date.clone()).into();
            let naive_start_TIMS: NaiveTime = TIMS(operation_csv.OPR_Start_Time.clone()).into();

            let naive_end_DATS: NaiveDate = DATS(operation_csv.OPR_End_Date.clone()).into();
            let naive_end_TIMS: NaiveTime = TIMS(operation_csv.OPR_End_Time.clone()).into();

            let naive_start_datetime = naive_start_DATS.and_time(naive_start_TIMS);
            let naive_end_datetime = naive_end_DATS.and_time(naive_end_TIMS);

            let utc_start_datetime = naive_start_datetime.and_utc();
            let utc_end_datetime = naive_end_datetime.and_utc();

            let operation_dates = OperationDates::new(
                Day::new(0, Utc::now()),
                Day::new(0, Utc::now()),
                utc_start_datetime,
                utc_end_datetime,
            );

            let operation = Operation::new(
                work_order_activity.1,
                resources.unwrap(),
                unloading_point,
                operation_info,
                operation_analytic,
                operation_dates,
            );
            operations.insert(work_order_activity.1, operation);
        }

        let work_order = WorkOrder::new(
            work_order_number,
            main_work_center,
            operations,
            Vec::new(),
            work_order_analytic,
            work_order_dates,
            work_order_info,
        );
        inner_work_orders.insert(work_order_number, work_order);
    }
    inner_work_orders
}

fn date_to_period(periods: &[Period], date_time: &NaiveDate) -> Period {
    let period: Option<Period> = periods
        .iter()
        .find(|period| {
            period.start_date().date_naive() <= *date_time
                && period.end_date().date_naive() >= *date_time
        })
        .cloned();

    match period {
        Some(period) => period,
        None => {
            let mut first_period = periods.first().unwrap().clone();
            let mut counter = 0;
            loop {
                counter += 1;
                first_period = first_period - Duration::weeks(2);
                if first_period.start_date().date_naive() <= *date_time
                    && first_period.end_date().date_naive() >= *date_time
                {
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
