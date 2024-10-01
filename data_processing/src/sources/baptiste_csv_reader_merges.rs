use std::collections::HashMap;

use chrono::{DateTime, TimeDelta};
use shared_types::scheduling_environment::{
    time_environment::period::Period,
    work_order::{
        self, operation::Work, status_codes::{self, MaterialStatus, StatusCodes}, work_order_dates::WorkOrderDates, ActivityRelation, WorkOrder, WorkOrderAnalytic, WorkOrderInfo, WorkOrderNumber
    },
    worker_environment::resources::MainResources,
    WorkOrders,
};

use super::{
    baptiste_csv_reader::{
        FLOCTechnicaID, FunctionalLocationsCsv, OPRObjectNumber, OPRRoutingNumber,
        OperationsStatusCsv, SecondaryLocationsCsv, WOObjectNumber, WOStatusId, WorkCenterCsv,
        WorkOperationsCsv, WorkOrdersCsv, WorkOrdersStatusCsv, WorkOrdersStatusCsvAggregated,
        WBSID,
    },
    excel::date_to_period,
};

fn create_work_orders(
    work_orders: HashMap<WorkOrderNumber, WorkOrdersCsv>,
    work_center: HashMap<WBSID, WorkCenterCsv>,
    work_operations: HashMap<OPRRoutingNumber, WorkOperationsCsv>,
    work_orders_status: WorkOrdersStatusCsvAggregated,
    operations_status: HashMap<OPRObjectNumber, OperationsStatusCsv>,
    functional_locations: HashMap<FLOCTechnicaID, FunctionalLocationsCsv>,
    periods: &[Period],
) -> HashMap<WorkOrderNumber, WorkOrder> {
    let inner_work_orders = HashMap::new();
    for (work_order_number, work_order_csv) in work_orders {
        let main_work_center: MainResources = MainResources::new_from_string(
            work_center
                .get(&work_order_csv.WO_WBS_ID)
                .unwrap()
                .WBS_Name
                .clone(),
        );

        let relations: Vec<ActivityRelation> = Vec::new();

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

        let earliest_allowed_start_date = work_order_csv.WO_Earliest_Allowed_Start_Date;
        let latest_allowed_finish_date = work_order_csv.WO_Latest_Allowed_Finish_Date;

        let duration = work_order_csv.WO_Basic_End_Date - work_order_csv.WO_Basic_Start_Date;

        let earliest_allowed_start_period = date_to_period(periods, &earliest_allowed_start_date);
        let latest_allowed_finish_period = date_to_period(periods, &latest_allowed_finish_date);

        let work_order_dates: WorkOrderDates = WorkOrderDates::new(
            earliest_allowed_start_date,
            latest_allowed_finish_date,
            earliest_allowed_start_period,
            latest_allowed_finish_period,
            work_order_csv.WO_Basic_Start_Date,
            work_order_csv.WO_Basic_End_Date,
            duration,
            None,
            None,
            None,
        );

        
        let work_order_info: WorkOrderInfo = WorkOrderInfo::new(
            work_order_csv.WO_Priority, 
            work_order_csv.WO_Order_Type, 

            work_order_csv., 
            work_order_csv., 
            work_order_csv., 
            work_order_csv., 
            work_order_csv., )
        // for operation in wo
        // let operations: HashMap<ActivityNumber, Operation> = HashMap::new();

        // let work_order = WorkOrder::new(
        //     work_order_number,
        //     main_work_center,
        //     work_order_analytic,
        //     work_order_dates,
        //     ,
        //     , )
    }
    inner_work_orders
}
