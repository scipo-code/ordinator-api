use rayon::prelude::*;
use std::{collections::HashMap, fs, path::PathBuf, str::FromStr};

use chrono::{Duration, NaiveDate, NaiveTime, Utc};
use serde::Deserialize;
use shared_types::scheduling_environment::{
    time_environment::{day::Day, period::Period},
    work_order::{
        self,
        functional_location::FunctionalLocation,
        operation::{
            operation_analytic::OperationAnalytic, operation_info::OperationInfo, Operation,
            OperationDates, Work,
        },
        priority::Priority,
        revision::Revision,
        status_codes::{MaterialStatus, StatusCodes},
        system_condition::SystemCondition,
        unloading_point::UnloadingPoint,
        work_order_dates::WorkOrderDates,
        work_order_text::WorkOrderText,
        work_order_type::WorkOrderType,
        WorkOrder, WorkOrderAnalytic, WorkOrderInfo, WorkOrderNumber,
    },
    worker_environment::resources::{MainResources, Resources},
    WorkOrders,
};

use crate::sap_mapper_and_types::{DATS, TIMS};

use super::baptiste_csv_reader::{
    populate_csv_structures, FLOCTechnicaID, FunctionalLocationsCsv, OperationsStatusCsv,
    OperationsStatusCsvAggregated, WorkCenterCsv, WorkOperations, WorkOperationsCsv, WorkOrdersCsv,
    WorkOrdersStatusCsv, WorkOrdersStatusCsvAggregated, WBSID,
};

pub fn load_csv_data(file_path: PathBuf, periods: &[Period]) -> WorkOrders {
    let contents = fs::read_to_string(file_path).unwrap();

    let file_paths: BaptisteToml = toml::from_str(&contents).unwrap();

    let functional_locations_csv =
        populate_csv_structures::<FunctionalLocationsCsv>(file_paths.mid_functional_locations)
            .expect("Could not read the csv file");

    let operations_status_csv =
        populate_csv_structures::<OperationsStatusCsv>(file_paths.mid_operations_status)
            .expect("Could not load the csv file");

    let work_center_csv = populate_csv_structures::<WorkCenterCsv>(file_paths.mid_work_center)
        .expect("Could not read the csv file");

    let work_operations_csv =
        populate_csv_structures::<WorkOperationsCsv>(file_paths.mid_work_operations)
            .expect("Could not read the csv file");

    let work_orders_csv = populate_csv_structures::<WorkOrdersCsv>(file_paths.mid_work_orders)
        .expect("Could not read the csv file");

    let work_orders_status_csv =
        populate_csv_structures::<WorkOrdersStatusCsv>(file_paths.mid_work_orders_status)
            .expect("Could not read the csv file");

    let work_orders_status_agg = WorkOrdersStatusCsvAggregated::new(work_orders_status_csv.clone());

    let operations_status_agg = OperationsStatusCsvAggregated::new(operations_status_csv.clone());

    let work_operations = WorkOperations::new(&work_orders_csv, &work_operations_csv);

    let work_orders_inner = create_work_orders(
        functional_locations_csv.clone(),
        operations_status_agg,
        periods,
        work_center_csv.clone(),
        work_operations,
        work_orders_csv.clone(),
        work_orders_status_agg,
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
    assert!(work_operations_csv.inner.len() > 0);
    let mut inner_work_orders = HashMap::new();

    let mut count = 0;

    for (work_order_number, work_order_csv) in work_orders {
        count += 1;
        dbg!(count);
        let main_work_center: MainResources = MainResources::new_from_string(
            work_center
                .get(&work_order_csv.WO_WBS_ID)
                .unwrap()
                .WBS_Name
                .clone(),
        );

        let status_codes_string = work_orders_status.inner.get(&work_order_csv.WO_Status_ID);

        let status_codes = match status_codes_string {
            Some(string) => {
                if !string.contains("REL") {
                    continue;
                }
                let pcnf_pattern = regex::Regex::new(r"PCNF").unwrap();
                let awsc_pattern = regex::Regex::new(r"AWSC").unwrap();
                let well_pattern = regex::Regex::new(r"WELL").unwrap();
                let sch_pattern = regex::Regex::new(r"SCH").unwrap();
                let sece_pattern = regex::Regex::new(r"SECE").unwrap();

                let material_status: MaterialStatus =
                    MaterialStatus::from_status_code_string(&string);

                StatusCodes {
                    material_status,
                    pcnf: pcnf_pattern.is_match(&string),
                    awsc: awsc_pattern.is_match(&string),
                    well: well_pattern.is_match(&string),
                    sch: sch_pattern.is_match(&string),
                    sece: sece_pattern.is_match(&string),
                    unloading_point: false, // Assuming default value; modify as needed
                }
            }
            None => StatusCodes::default(),
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
            DATS(work_order_csv.WO_Earliest_Allowed_Start_Date)
                .try_into()
                .expect("The WorkOrders that have invalid EASD are filtered out");

        let latest_allowed_finish_date: NaiveDate =
            DATS(work_order_csv.WO_Latest_Allowed_Finish_Date)
                .try_into()
                .expect("The WorkOrders that have invalid EASD are filtered out");

        let basic_start: NaiveDate = DATS(work_order_csv.WO_Basic_Start_Date)
            .try_into()
            .expect("The WorkOrders that have invalid EASD are filtered out");

        let basic_finish: NaiveDate = DATS(work_order_csv.WO_Basic_End_Date)
            .try_into()
            .expect("The WorkOrders that have invalid EASD are filtered out");

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

        let functional_location =
            &functional_locations.get(&work_order_csv.WO_Functional_Location_Number);

        let functional_location = match functional_location {
            Some(functional_location_csv) => &functional_location_csv.FLOC_Name,
            None => "WARN: FUNCTIONAL_LOCATION MISSING IS THIS CORRECT?",
        };

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

        let priority = Priority::dyn_new(Box::new(work_order_csv.WO_Priority));

        let work_order_type = WorkOrderType::new(&work_order_csv.WO_Order_Type, priority.clone())
            .expect("Invalid WorkOrderType's should have been filtered out");

        let work_order_info: WorkOrderInfo = WorkOrderInfo::new(
            priority,
            work_order_type,
            FunctionalLocation::new(functional_location.to_string()),
            work_order_text,
            Revision::new(work_order_csv.WO_Revision),
            SystemCondition::from_str(&work_order_csv.WO_System_Condition).unwrap(),
            work_order_info_detail,
        );

        let mut operations = HashMap::new();
        for (work_order_activity, operation_csv) in work_operations_csv
            .inner
            .get(&work_order_number)
            .cloned()
            .unwrap_or_default()
        {
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
            let naive_start_DATS: NaiveDate = DATS(operation_csv.OPR_Start_Date.clone()).try_into().expect("The OPR_Start_Date should have been filtered out, we should not experience this error.");
            let naive_start_TIMS: NaiveTime = TIMS(operation_csv.OPR_Start_Time.clone()).try_into().expect("The OPR_Start_Time should have been filtered out, we should not experience this error.");

            let naive_end_DATS: NaiveDate = DATS(operation_csv.OPR_End_Date.clone()).try_into().expect("The OPR_End_Date should have been filtered out, we should not experience this error.");
            let naive_end_TIMS: NaiveTime = TIMS(operation_csv.OPR_End_Time.clone()).try_into().expect("The OPR_End_Time should have been filtered out, we should not experience this error.");

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
                work_order_activity,
                resources.unwrap(),
                unloading_point,
                operation_info,
                operation_analytic,
                operation_dates,
            );
            operations.insert(work_order_activity, operation);
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
