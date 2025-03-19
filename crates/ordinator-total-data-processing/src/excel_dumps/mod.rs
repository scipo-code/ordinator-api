use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use rust_xlsxwriter::Worksheet;
use shared_types::AgentExports;
use shared_types::Asset;
use shared_types::ReasonForNotScheduling;
use shared_types::scheduling_environment::time_environment::day::Day;
use shared_types::scheduling_environment::time_environment::day::OptionDay;
use shared_types::scheduling_environment::work_order::WorkOrder;
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use shared_types::scheduling_environment::work_order::WorkOrders;
use shared_types::scheduling_environment::work_order::operation::ActivityNumber;
use shared_types::scheduling_environment::work_order::operation::Work;
use shared_types::scheduling_environment::work_order::work_order_analytic::status_codes::MaterialStatus;
use shared_types::scheduling_environment::work_order::work_order_analytic::status_codes::SystemStatusCodes;
use shared_types::scheduling_environment::work_order::work_order_analytic::status_codes::UserStatusCodes;
use shared_types::scheduling_environment::work_order::work_order_dates::unloading_point::UnloadingPoint;
use shared_types::scheduling_environment::work_order::work_order_info::functional_location::FunctionalLocation;
use shared_types::scheduling_environment::work_order::work_order_info::priority::Priority;
use shared_types::scheduling_environment::work_order::work_order_info::revision::Revision;
use shared_types::scheduling_environment::work_order::work_order_info::system_condition::SystemCondition;
use shared_types::scheduling_environment::work_order::work_order_info::work_order_type::WorkOrderType;
use shared_types::scheduling_environment::worker_environment::resources::Resources;

use crate::sap_mapper_and_types::DATS;

#[derive(Debug)]
struct AllRows(Vec<RowNames>);

impl AllRows {
    fn make_xlsx_dump(&self, asset: Asset) -> Result<PathBuf, rust_xlsxwriter::XlsxError> {
        let mut rust_dump = rust_xlsxwriter::Workbook::new();

        let worksheet: &mut Worksheet = rust_dump.add_worksheet();

        make_header_row(worksheet);

        assert!(!self.0.is_empty());
        for (row_count, row_values) in self.0.iter().enumerate() {
            let row_number: u32 = (row_count + 1) as u32;

            worksheet
                .write(row_number, 0, row_values.strategic_schedule.clone())
                .unwrap();
            worksheet
                .write(row_number, 1, row_values.tactical_schedule.clone())
                .unwrap();
            worksheet
                .write(row_number, 2, row_values.priority.clone())
                .unwrap();
            worksheet
                .write(row_number, 3, row_values.revision.clone())
                .unwrap();
            worksheet
                .write(row_number, 4, row_values.work_order_type.clone())
                .unwrap();
            worksheet
                .write(row_number, 5, row_values.main_work_ctr)
                .unwrap();
            worksheet
                .write(row_number, 6, row_values.operation_work_center)
                .unwrap();
            worksheet
                .write(row_number, 7, row_values.work_order_number.0)
                .unwrap();
            worksheet
                .write(row_number, 8, row_values.description_work_order.clone())
                .unwrap();
            worksheet
                .write(row_number, 9, row_values.operation_short_text.clone())
                .unwrap();
            worksheet
                .write(row_number, 10, row_values.material_status.clone())
                .unwrap();
            worksheet
                .write(row_number, 10, row_values.system_status.clone())
                .unwrap();
            worksheet
                .write(row_number, 11, row_values.user_status.clone())
                .unwrap();
            worksheet.write(row_number, 12, row_values.work).unwrap();
            worksheet
                .write(row_number, 13, row_values.actual_work)
                .unwrap();
            worksheet
                .write(row_number, 14, row_values.unloading_point.clone())
                .unwrap();
            worksheet
                .write(row_number, 15, row_values.basic_start_date.clone())
                .unwrap();
            worksheet
                .write(row_number, 16, row_values.basic_finish_date.clone())
                .unwrap();
            worksheet
                .write(row_number, 17, row_values.earliest_start_date.clone())
                .unwrap();
            worksheet
                .write(row_number, 18, row_values.earliest_finish_date.clone())
                .unwrap();
            worksheet
                .write(
                    row_number,
                    19,
                    row_values.earliest_allowed_start_date.clone(),
                )
                .unwrap();
            worksheet
                .write(
                    row_number,
                    20,
                    row_values.latest_allowed_finish_date.clone(),
                )
                .unwrap();
            worksheet
                .write(row_number, 21, row_values.activity)
                .unwrap();
            // worksheet
            //     .write(row_number, 22, row_values.opperation_system_status.clone())
            //     .unwrap();
            // worksheet
            //     .write(row_number, 23, row_values.opereration_user_status.clone())
            //     .unwrap();
            worksheet
                .write(row_number, 22, row_values.functional_location.clone())
                .unwrap();
            worksheet
                .write(row_number, 23, row_values.description_operation.clone())
                .unwrap();
            worksheet
                .write(row_number, 24, row_values.subnetwork_of.clone())
                .unwrap();
            worksheet
                .write(row_number, 25, row_values.system_condition.clone())
                .unwrap();
            worksheet
                .write(row_number, 26, row_values.maintenance_plan.clone())
                .unwrap();
            worksheet
                .write(row_number, 27, row_values.planner_group.clone())
                .unwrap();
            worksheet
                .write(row_number, 28, row_values.maintenance_plant.clone())
                .unwrap();
            worksheet
                .write(row_number, 29, row_values.pm_collective.clone())
                .unwrap();
            worksheet
                .write(row_number, 30, row_values.room.clone())
                .unwrap();
        }
        let xlsx_directory = dotenvy::var("EXCEL_DUMP_DIRECTORY").expect(
            "The excel dump directory environment path could not be found. Check the .env file",
        );
        let xlsx_name = format!("ordinator_dump_for_asset_{}.xlsx", asset);
        let xlsx_string = xlsx_directory + &xlsx_name;
        let xlsx_path = PathBuf::from(&xlsx_string);
        rust_dump.save(&xlsx_path)?;
        Ok(xlsx_path)
    }
}

#[derive(Debug)]
struct RowNames {
    strategic_schedule: ReasonForNotScheduling,
    tactical_schedule: OptionDay,
    priority: Priority,
    revision: Revision,
    work_order_type: WorkOrderType,
    main_work_ctr: Resources,
    operation_work_center: Resources,
    work_order_number: WorkOrderNumber,
    description_work_order: String,
    operation_short_text: String,
    material_status: MaterialStatus,
    system_status: SystemStatusCodes,
    user_status: UserStatusCodes,
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
    // operation_system_status: SystemStatusCodes,
    // operation_user_status: SystemStatusCodes,
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
/// The function will dump the excel file in the folder specified by the
/// EXCEL_DUMP_DIRECTORY environment variable.
pub fn create_excel_dump(
    asset: Asset,
    work_orders: WorkOrders,
    strategic_solution: HashMap<WorkOrderNumber, Option<Period>>,
    tactical_solution: HashMap<WorkOrderNumber, HashMap<ActivityNumber, Day>>,
) -> Result<PathBuf> {
    let mut all_rows: Vec<RowNames> = Vec::new();

    dbg!("Total WorkOrder(s):", work_orders.inner.len());
    let work_orders_by_asset: Vec<WorkOrder> = work_orders
        .inner
        .into_iter()
        .filter(|(_, wo)| wo.work_order_info.functional_location.asset == asset)
        .map(|(_, wo)| wo)
        .collect();

    for work_order in work_orders_by_asset {
        let mut sorted_operations = work_order.operations.0.iter().collect::<Vec<_>>();

        sorted_operations
            .sort_unstable_by(|value1, value2| value1.0.partial_cmp(value2.0).unwrap());

        let strategic_period = match strategic_solution.clone() {
            AgentExports::Strategic(solution) => {
                let reason_for_scheduling = solution
                    .get(&work_order.work_order_number)
                    .with_context(|| format!("{:#?}", work_order))?
                    .clone();

                match reason_for_scheduling {
                    Some(period) => ReasonForNotScheduling::Scheduled(period),
                    None => {
                        ReasonForNotScheduling::Unknown("COULD NOT BE SCHEDULED, WHY?".to_string())
                    }
                }
            }
            AgentExports::Tactical(_) => panic!(),
        };
        for activity in sorted_operations {
            let option_day = match tactical_solution.get(&work_order.work_order_number) {
                Some(tactical_day) => {
                    let mut days = tactical_day.iter().collect::<Vec<_>>();
                    days.sort();
                    OptionDay(Some(days[0].1.clone()))
                }
                None => OptionDay(None),
            };

            let one_row = RowNames {
                strategic_schedule: strategic_period.clone(),
                tactical_schedule: option_day,
                priority: work_order.work_order_info.priority.clone(),
                revision: work_order.work_order_info.revision.clone(),
                work_order_type: work_order.work_order_info.work_order_type.clone(),
                main_work_ctr: work_order.main_work_center,
                operation_work_center: activity.1.resource,
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
                    .clone()
                    .unwrap_or("WE DO NOT HAVE THIS FIELD FROM SAP YET".to_string()),
                material_status: work_order
                    .work_order_analytic
                    .user_status_codes
                    .clone()
                    .into(),
                system_status: work_order.work_order_analytic.system_status_codes.clone(),
                user_status: work_order.work_order_analytic.user_status_codes.clone(),
                work: activity.1.operation_info.work_remaining,
                actual_work: activity.1.operation_info.work_actual,
                unloading_point: activity.1.unloading_point.clone(),
                basic_start_date: work_order.work_order_dates.basic_start_date.into(),
                basic_finish_date: work_order.work_order_dates.basic_finish_date.into(),
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
                    .into(),
                latest_allowed_finish_date: work_order
                    .work_order_dates
                    .latest_allowed_finish_date
                    .into(),
                activity: *activity.0,
                // operation_system_status: work_order.status_codes().clone(),
                // operation_user_status: work_order.status_codes().clone(),
                functional_location: work_order.functional_location().clone(),
                description_operation: work_order
                    .work_order_info
                    .work_order_text
                    .operation_description
                    .clone()
                    .unwrap_or("WE DO NOT HAVE THIS SAP FIELD YET".to_string()),
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

    let xlsx_path = all_rows.make_xlsx_dump(asset).unwrap();

    Ok(xlsx_path)
}
fn make_header_row(worksheet: &mut Worksheet) {
    worksheet.write(0, 0, "strategic_schedule").unwrap();
    worksheet.write(0, 1, "tactical_schedule").unwrap();
    worksheet.write(0, 2, "priority").unwrap();
    worksheet.write(0, 3, "revision").unwrap();
    worksheet.write(0, 4, "work_order_type").unwrap();
    worksheet.write(0, 5, "main_work_ctr").unwrap();
    worksheet.write(0, 6, "operation_work_center").unwrap();
    worksheet.write(0, 7, "work_order_number").unwrap();
    worksheet.write(0, 8, "description_work_order").unwrap();
    worksheet.write(0, 9, "operation_short_text").unwrap();
    worksheet.write(0, 10, "system_status").unwrap();
    worksheet.write(0, 11, "user_status").unwrap();
    worksheet.write(0, 12, "work").unwrap();
    worksheet.write(0, 13, "actual_work").unwrap();
    worksheet.write(0, 14, "unloading_point").unwrap();
    worksheet.write(0, 15, "basic_start_date").unwrap();
    worksheet.write(0, 16, "basic_finish_date").unwrap();
    worksheet.write(0, 17, "earliest_start_date").unwrap();
    worksheet.write(0, 18, "earliest_finish_date").unwrap();
    worksheet
        .write(0, 19, "earliest_allowed_start_date")
        .unwrap();
    worksheet
        .write(0, 20, "latest_allowed_finish_date")
        .unwrap();
    worksheet.write(0, 21, "activity").unwrap();
    worksheet.write(0, 22, "opperation_system_status").unwrap();
    worksheet.write(0, 23, "opereration_user_status").unwrap();
    worksheet.write(0, 24, "functional_location").unwrap();
    worksheet.write(0, 25, "description_operation").unwrap();
    worksheet.write(0, 26, "subnetwork_of").unwrap();
    worksheet.write(0, 27, "system_condition").unwrap();
    worksheet.write(0, 28, "maintenance_plan").unwrap();
    worksheet.write(0, 29, "planner_group").unwrap();
    worksheet.write(0, 30, "maintenance_plant").unwrap();
    worksheet.write(0, 31, "pm_collective").unwrap();
    worksheet.write(0, 32, "room").unwrap();
}

#[derive(Debug, Clone)]
pub enum ReasonForNotScheduling {
    Scheduled(Period),
    Unknown(String),
}

impl IntoExcelData for ReasonForNotScheduling {
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = match self {
            ReasonForNotScheduling::Scheduled(period) => period.period_string(),
            ReasonForNotScheduling::Unknown(unknown) => unknown,
        };
        worksheet.write_string(row, col, value)
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
        format: &rust_xlsxwriter::Format,
    ) -> Result<&'a mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = match self {
            ReasonForNotScheduling::Scheduled(period) => period.period_string(),
            ReasonForNotScheduling::Unknown(unknown) => unknown,
        };
        worksheet.write_string_with_format(row, col, value, format)
    }
}
