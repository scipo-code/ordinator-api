use crate::sap_mapper_and_types::DATS;
use chrono::NaiveDate;
// use rust_xlsxwriter::prelude::*;
use std::collections::HashMap;

use shared_types::{
    scheduling_environment::{
        time_environment::period::Period,
        work_order::{
            functional_location::FunctionalLocation,
            operation::{ActivityNumber, Work},
            priority::Priority,
            revision::Revision,
            status_codes::StatusCodes,
            system_condition::SystemCondition,
            unloading_point::UnloadingPoint,
            work_order_type::WorkOrderType,
            WorkOrder, WorkOrderActivity, WorkOrderNumber,
        },
        worker_environment::resources::{MainResources, Resources},
        SchedulingEnvironment,
    },
    tactical::Days,
    Asset,
};

struct AllRows(Vec<RowNames>);
// impl AllRows {
//     fn make_xlsx_dump(&self) -> _ {
//         let mut rust_dump = rust_xlsxwriter::Workbook::new();

//         let mut work_sheet = rust_dump.add_worksheet();

//         work_sheet.write(, , )
//     }
// }

struct RowNames {
    priority: Priority,
    revision: Revision,
    order_type: WorkOrderType,
    main_work_ctr: MainResources,
    oper_work_center: Resources,
    order: WorkOrderNumber,
    description_work_order: String,
    operation_short_text: String,
    system_status: StatusCodes,
    user_status: StatusCodes,
    work: Work,
    actual_work: Work,
    unloading_point: UnloadingPoint,
    basic_start_date: DATS,
    basic_finish_date: DATS,
    earliest_start_date: DATS,
    earliest_finish_date: DATS,
    earliest_allowed_start_date: DATS,
    latest_allowed_finish_date: DATS,
    activity: ActivityNumber,
    opperation_system_status: StatusCodes,
    opereration_user_status: StatusCodes,
    functional_location: FunctionalLocation,
    description_operation: String,
    subnetwork_of: String,
    system_condition: SystemCondition,
    maintenance_plan: String,
    planner_group: String,
    maintenance_plant: String,
    pm_collective: String,
    room: String,
}

/// This function will create an excel dump based on the current state of the:
/// * SchedulingEnvironment
/// * StrategicAlgorithm
/// * TacticalAlgorithm
///
/// The function will dump the excel file in the folder specified by the EXCEL_DUMP_DIRECTORY
/// environment variable.
pub fn create_excel_dump(
    asset: Asset,
    scheduling_environment: SchedulingEnvironment,
    strategic_solution: HashMap<WorkOrderNumber, Period>,
    tactical_solution: HashMap<WorkOrderActivity, Days>,
) -> Result<(), std::io::Error> {
    let mut all_rows: Vec<RowNames> = Vec::new();
    let work_orders = scheduling_environment.work_orders().clone();

    let work_orders_by_asset: Vec<WorkOrder> = work_orders
        .inner
        .into_iter()
        .filter(|(won, wo)| wo.work_order_info.functional_location.asset == asset)
        .map(|(won, wo)| wo)
        .collect();

    for work_order in work_orders_by_asset {
        let mut sorted_operations = work_order.operations.iter().collect::<Vec<_>>();

        sorted_operations
            .sort_unstable_by(|value1, value2| value1.0.partial_cmp(value2.0).unwrap());

        for activity in sorted_operations {
            let one_row = RowNames {
                priority: work_order.priority().clone(),
                revision: work_order.revision().clone(),
                order_type: work_order.work_order_type().clone(),
                main_work_ctr: work_order.main_work_center.clone(),
                oper_work_center: activity.1.resource.clone(),
                order: work_order.work_order_number,
                description_work_order: work_order
                    .work_order_info
                    .work_order_text
                    .order_description
                    .clone(),
                operation_short_text: work_order
                    .work_order_info
                    .work_order_text
                    .operation_description
                    .clone(),
                system_status: work_order.status_codes().clone(),
                user_status: work_order.status_codes().clone(),
                work: activity.1.work_remaining().clone(),
                actual_work: activity.1.operation_info.work_actual.clone(),
                unloading_point: work_order.unloading_point().clone(),
                basic_start_date: work_order
                    .work_order_dates
                    .basic_start_date
                    .date_naive()
                    .into(),
                basic_finish_date: work_order
                    .work_order_dates
                    .basic_finish_date
                    .date_naive()
                    .into(),
                earliest_start_date: activity
                    .1
                    .operation_dates
                    .earliest_start_datetime
                    .date_naive()
                    .into(),
                earliest_finish_date: activity
                    .1
                    .operation_dates
                    .earliest_finish_datetime
                    .date_naive()
                    .into(),
                earliest_allowed_start_date: work_order
                    .work_order_dates
                    .earliest_allowed_start_date
                    .date_naive()
                    .into(),
                latest_allowed_finish_date: work_order
                    .work_order_dates
                    .latest_allowed_finish_date
                    .date_naive()
                    .into(),
                activity: activity.0.clone(),
                opperation_system_status: work_order.status_codes().clone(),
                opereration_user_status: work_order.status_codes().clone(),
                functional_location: work_order.functional_location().clone(),
                description_operation: work_order
                    .work_order_info
                    .work_order_text
                    .operation_description
                    .clone(),
                subnetwork_of: work_order
                    .work_order_info
                    .work_order_info_detail
                    .subnetwork
                    .clone(),
                system_condition: work_order.work_order_info.system_condition.clone(),
                maintenance_plan: work_order
                    .work_order_info
                    .work_order_info_detail
                    .maintenance_plan
                    .clone(),
                planner_group: work_order
                    .work_order_info
                    .work_order_info_detail
                    .planner_group
                    .clone(),
                maintenance_plant: work_order
                    .work_order_info
                    .work_order_info_detail
                    .maintenance_plant
                    .clone(),
                pm_collective: work_order
                    .work_order_info
                    .work_order_info_detail
                    .pm_collective
                    .clone(),
                room: work_order
                    .work_order_info
                    .work_order_info_detail
                    .room
                    .clone(),
            };

            all_rows.push(one_row);
        }
    }
    let all_rows = AllRows(all_rows);

    // all_rows.make_xlsx_dump();

    Ok(())
}
