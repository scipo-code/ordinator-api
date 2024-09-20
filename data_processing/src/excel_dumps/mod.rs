use std::collections::HashMap;

use shared_types::{
    scheduling_environment::{
        time_environment::period::Period,
        work_order::{WorkOrder, WorkOrderActivity, WorkOrderNumber},
        SchedulingEnvironment,
    },
    tactical::Days,
    Asset,
};

struct RowNames {
    priority: String,
    revision: String,
    order_type: String,
    main_work_ctr: String,
    oper_work_center: String,
    order: String,
    description_work_order: String,
    opr_short_text: String,
    system_status: String,
    user_status: String,
    work: String,
    actual_work: String,
    unloading_point: String,
    basic_start_date: String,
    basic_finish_date: String,
    earliest_start_date: String,
    earliest_allowed_start_date: String,
    latest_allowed_finish_date: String,
    activity: String,
    opperation_system_status: String,
    opereration_user_status: String,
    functional_location: String,
    description_operation: String,
    subnetwork_of: String,
    system_condition: String,
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
) {
    let work_orders = scheduling_environment.work_orders().clone();

    let work_orders_by_asset: Vec<WorkOrder> = work_orders
        .inner
        .into_iter()
        .filter(|(won, wo)| wo.work_order_info.functional_location.asset == asset)
        .map(|(won, wo)| wo)
        .collect();

    for work_order in work_orders_by_asset {
        work_order.
    }
}

pub fn create_one_row() -> RowNames {}
