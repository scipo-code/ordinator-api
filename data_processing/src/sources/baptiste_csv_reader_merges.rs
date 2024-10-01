use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, TimeDelta, Timelike};
use shared_types::scheduling_environment::{
    time_environment::{day::Day, period::Period},
    work_order::{
        self,
        functional_location::FunctionalLocation,
        operation::{operation_analytic::{self, OperationAnalytic}, operation_info::{self, OperationInfo}, ActivityNumber, Operation, OperationDates, Work},
        revision::Revision,
        status_codes::{self, MaterialStatus, StatusCodes},
        system_condition::SystemCondition,
        work_order_dates::WorkOrderDates,
        work_order_text::WorkOrderText,
        ActivityRelation, WorkOrder, WorkOrderAnalytic, WorkOrderInfo, WorkOrderNumber,
    },
    worker_environment::resources::{MainResources, Resources},
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
    work_operations_csv: HashMap<(WorkOrderNumber, ActivityNumber), WorkOperationsCsv>,
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

        let functional_location = functional_locations
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

        let work_order_info: WorkOrderInfo = WorkOrderInfo::new(
            work_order_csv.WO_Priority,
            work_order_csv.WO_Order_Type,
            FunctionalLocation::new(functional_location),
            work_order_text,
            Revision::new(work_order_csv.WO_Revision),
            SystemCondition::from_str(&work_order_csv.WO_System_Condition),
            work_order_info_detail,
        );

        let mut operations = HashMap::new();
        for (work_order_activity, operation_csv) in work_operations_csv {

            let resources = Resources::from_str(&work_center.get(&operation_csv.OPR_WBS_ID).unwrap().WBS_Name);
            
            let operation_info = OperationInfo::new(
                operation_csv.OPR_Workers_Numbers,
                operation_csv.OPR_Planned_Work ,
                operation_csv.OPR_Actual_Work ,
                operation_csv.OPR_Planned_Work ,
                None,
            );

            let operation_analytic = OperationAnalytic::new(
                Work::from(1.0), 
                operation_csv.OPR_Planned_Work,
            );

            // TODO start here
            let operation_start = DateTime::with_hour(, )

            let operation_dates = OperationDates::new(
                Day::new(, )
                Day::new(, ), 
                operation_csv., 
                , )
            
            let operation = Operation::new(
                operation_csv.0.1, 
                resources,
                operation_info, 
                operation_analytic, 
                , )
            
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
