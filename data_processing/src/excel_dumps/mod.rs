use crate::sap_mapper_and_types::DATS;
use chrono::NaiveDate;
use rust_xlsxwriter::{IntoExcelData, Workbook};
use std::{collections::HashMap, path::Path, time::Instant};

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
        SchedulingEnvironment, WorkOrders,
    },
    tactical::Days,
    AgentExports, Asset,
};

trait WriteXlsxRow {
    fn create_xlsx_row(&self) -> Vec<String>;
}

struct AllRows(Vec<RowNames>);

impl AllRows {
    fn make_xlsx_dump(&self, asset: Asset) -> Result<(), rust_xlsxwriter::XlsxError> {
        let mut rust_dump = rust_xlsxwriter::Workbook::new();

        let mut work_sheet = rust_dump.add_worksheet();

        for (row_number, row_values) in self.0.iter().enumerate() {
            work_sheet
                .write(row_number as u32, 0, row_values.priority.clone())
                .unwrap();
            work_sheet
                .write(row_number as u32, 1, row_values.revision.clone())
                .unwrap();
            work_sheet
                .write(row_number as u32, 2, row_values.work_order_type.clone())
                .unwrap();
            work_sheet
                .write(row_number as u32, 3, row_values.main_work_ctr.clone())
                .unwrap();
            work_sheet
                .write(
                    row_number as u32,
                    4,
                    row_values.operation_work_center.clone(),
                )
                .unwrap();
            work_sheet
                .write(row_number as u32, 5, row_values.work_order_number.0.clone())
                .unwrap();
            work_sheet
                .write(
                    row_number as u32,
                    6,
                    row_values.description_work_order.clone(),
                )
                .unwrap();
            work_sheet
                .write(
                    row_number as u32,
                    7,
                    row_values.operation_short_text.clone(),
                )
                .unwrap();
            work_sheet
                .write(row_number as u32, 8, row_values.system_status.clone())
                .unwrap();
            work_sheet
                .write(row_number as u32, 9, row_values.user_status.clone())
                .unwrap();
            work_sheet
                .write(row_number as u32, 10, row_values.work.clone())
                .unwrap();
            work_sheet
                .write(row_number as u32, 11, row_values.actual_work.clone())
                .unwrap();
            work_sheet
                .write(row_number as u32, 12, row_values.unloading_point.clone())
                .unwrap();
            work_sheet
                .write(row_number as u32, 13, row_values.basic_start_date.clone())
                .unwrap();
            work_sheet
                .write(row_number as u32, 14, row_values.basic_finish_date.clone())
                .unwrap();
            work_sheet
                .write(
                    row_number as u32,
                    15,
                    row_values.earliest_start_date.clone(),
                )
                .unwrap();
            work_sheet
                .write(
                    row_number as u32,
                    16,
                    row_values.earliest_finish_date.clone(),
                )
                .unwrap();
            work_sheet
                .write(
                    row_number as u32,
                    17,
                    row_values.earliest_allowed_start_date.clone(),
                )
                .unwrap();
            work_sheet
                .write(
                    row_number as u32,
                    18,
                    row_values.latest_allowed_finish_date.clone(),
                )
                .unwrap();
            work_sheet
                .write(row_number as u32, 19, row_values.activity.0.clone())
                .unwrap();
            work_sheet
                .write(
                    row_number as u32,
                    20,
                    row_values.opperation_system_status.clone(),
                )
                .unwrap();
            work_sheet
                .write(
                    row_number as u32,
                    21,
                    row_values.opereration_user_status.clone(),
                )
                .unwrap();
            work_sheet
                .write(
                    row_number as u32,
                    22,
                    row_values.functional_location.clone(),
                )
                .unwrap();
            work_sheet
                .write(
                    row_number as u32,
                    23,
                    row_values.description_operation.clone(),
                )
                .unwrap();
            work_sheet
                .write(row_number as u32, 24, row_values.subnetwork_of.clone())
                .unwrap();
            work_sheet
                .write(row_number as u32, 25, row_values.system_condition.clone())
                .unwrap();
            work_sheet
                .write(row_number as u32, 26, row_values.maintenance_plan.clone())
                .unwrap();
            work_sheet
                .write(row_number as u32, 27, row_values.planner_group.clone())
                .unwrap();
            work_sheet
                .write(row_number as u32, 28, row_values.maintenance_plant.clone())
                .unwrap();
            work_sheet
                .write(row_number as u32, 29, row_values.pm_collective.clone())
                .unwrap();
            work_sheet
                .write(row_number as u32, 30, row_values.room.clone())
                .unwrap();
        }
        let current_time = Instant::now();

        let xlsx_directory = dotenvy::var("EXCEL_DUMP_DIRECTORY").expect(
            "The excel dump directory environment path could not be found. Check the .env file",
        );
        let xlsx_name = format!("ordinator_xlsx_dump_{:?}_{}", current_time, asset);

        let xlsx_string = xlsx_directory + &xlsx_name;
        let xlsx_path = Path::new(&xlsx_string);

        rust_dump.save(&xlsx_path)
    }
}

struct RowNames {
    priority: Priority,
    revision: Revision,
    work_order_type: WorkOrderType,
    main_work_ctr: MainResources,
    operation_work_center: Resources,
    work_order_number: WorkOrderNumber,
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
    work_orders: WorkOrders,
    strategic_solution: AgentExports,
    tactical_solution: AgentExports,
) -> Result<(), std::io::Error> {
    let mut all_rows: Vec<RowNames> = Vec::new();

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
                work_order_type: work_order.work_order_type().clone(),
                main_work_ctr: work_order.main_work_center.clone(),
                operation_work_center: activity.1.resource.clone(),
                work_order_number: work_order.work_order_number,
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

    all_rows.make_xlsx_dump(asset).unwrap();

    Ok(())
}
