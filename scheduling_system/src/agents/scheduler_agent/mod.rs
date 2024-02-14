pub mod display;
pub mod scheduler_algorithm;
pub mod scheduler_message;

use crate::agents::scheduler_agent::scheduler_algorithm::SchedulerAgentAlgorithm;
use crate::agents::scheduler_agent::scheduler_message::ScheduleIteration;
use crate::api::websocket_agent::WebSocketAgent;
use crate::models::time_environment::period::Period;
use crate::models::work_order::order_type::WorkOrderType;
use crate::models::work_order::priority::Priority;
use crate::models::work_order::status_codes::MaterialStatus;
use crate::models::SchedulingEnvironment;

use actix::prelude::*;
use shared_messages::resources::Resources;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::agents::work_planner_agent::WorkPlannerAgent;

/// This is the primary struct for the scheduler agent.
#[allow(dead_code)]
pub struct StrategicAgent {
    platform: String,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    scheduler_agent_algorithm: SchedulerAgentAlgorithm,
    ws_agent_addr: Option<Addr<WebSocketAgent>>,
    work_planner_agent_addr: Option<Addr<WorkPlannerAgent>>,
}

impl StrategicAgent {
    pub fn set_ws_agent_addr(&mut self, ws_agent_addr: Addr<WebSocketAgent>) {
        self.ws_agent_addr = Some(ws_agent_addr);
    }
}

impl Actor for StrategicAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        self.scheduler_agent_algorithm.populate_priority_queues();
        ctx.notify(ScheduleIteration {})
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        println!("SchedulerAgent is stopped");
    }
}

impl StrategicAgent {
    pub fn new(
        platform: String,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        scheduler_agent_algorithm: SchedulerAgentAlgorithm,
        ws_agent_addr: Option<Addr<WebSocketAgent>>,
        work_planner_agent_addr: Option<Addr<WorkPlannerAgent>>,
    ) -> Self {
        Self {
            platform,
            scheduling_environment,
            scheduler_agent_algorithm,
            ws_agent_addr,
            work_planner_agent_addr,
        }
    }
}

/// This implementation will update the current state of the scheduler agent.
///
/// I have an issue with how the scheduled work orders should be handled. I think that there are
/// multiple approaches to solving this problem. The queue idea is good but then I would have to
/// update the other queues if the work order is present in one of those queues. I could also just
/// bypass the whole thing. Hmm... I have misunderstood something here. Should I make the solution
/// scheduled_work_orders are the once that are scheduled. But there is also the question of the
/// scheduled field in the central data structure. I should find out where that comes from and
///
/// So here we update the state of the application, but what about the queues? I after the work
/// order has been scheduled in the front end we need to update the queues. As well so that the
/// work order is scheduled through the process. We should add the work order to the unloading point
/// queue but what will happen when the work order is unscheduled again at a later point? This is
/// much more difficult to reason about. I think that the best approach is
///
/// All of this should be handled in the update scheduler state function. There can be no other way
/// Remember that if this becomes complex we should refactor the code.

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SchedulingOverviewData {
    scheduled_period: String,
    scheduled_start: String,
    unloading_point: String,
    material_date: String,
    work_order_number: u32,
    activity: String,
    work_center: String,
    work_remaining: String,
    number: u32,
    notes_1: String,
    notes_2: String,
    order_description: String,
    object_description: String,
    order_user_status: String,
    order_system_status: String,
    functional_location: String,
    revision: String,
    earliest_start_datetime: String,
    earliest_finish_datetime: String,
    earliest_allowed_starting_date: String,
    latest_allowed_finish_date: String,
    order_type: String,
    priority: String,
}

/// Now the problem is that the many work orders may not even get a status, in this approach.
/// This is an issue. Now when we get the work_order_number the entry could be non-existent.
///
/// Here it is not the alrithm that should be the one that should be used to generate the overview
/// I think that here we should use the work orders from the scheduling environment to extract the
/// scheduling environment correctly. This is a good point.
impl StrategicAgent {
    fn extract_state_to_scheduler_overview(&self) -> Vec<SchedulingOverviewData> {
        let mut scheduling_overview_data: Vec<SchedulingOverviewData> = Vec::new();

        let work_orders = self
            .scheduling_environment
            .lock()
            .unwrap()
            .clone_work_orders();

        for (work_order_number, work_order) in work_orders.inner {
            for (operation_number, operation) in work_order.get_operations().clone() {
                let scheduling_overview_data_item = SchedulingOverviewData {
                    scheduled_period: match self
                        .scheduler_agent_algorithm
                        .get_optimized_work_order(&work_order_number)
                    {
                        Some(order_period) => match order_period.get_scheduled_period().as_ref() {
                            Some(scheduled_period) => scheduled_period.get_period_string().clone(),
                            None => "not scheduled".to_string(),
                        },
                        None => "not scheduled".to_string(),
                    },
                    scheduled_start: work_order.get_order_dates().basic_start_date.to_string(),
                    unloading_point: work_order.get_unloading_point().clone().string,
                    material_date: match work_order.get_status_codes().material_status {
                        MaterialStatus::Smat => "SMAT".to_string(),
                        MaterialStatus::Nmat => "NMAT".to_string(),
                        MaterialStatus::Cmat => "CMAT".to_string(),
                        MaterialStatus::Wmat => "WMAT".to_string(),
                        MaterialStatus::Pmat => "PMAT".to_string(),
                        MaterialStatus::Unknown => "Implement control tower".to_string(),
                    },
                    work_order_number,
                    activity: operation_number.clone().to_string(),
                    work_center: operation.work_center.variant_name(),
                    work_remaining: operation.work_remaining.to_string(),
                    number: operation.number,
                    notes_1: work_order.get_order_text().notes_1.clone(),
                    notes_2: work_order.get_order_text().notes_2.clone().to_string(),
                    order_description: work_order.get_order_text().order_description.clone(),
                    object_description: work_order.get_order_text().object_description.clone(),
                    order_user_status: work_order.get_order_text().order_user_status.clone(),
                    order_system_status: work_order.get_order_text().order_system_status.clone(),
                    functional_location: work_order.get_functional_location().clone().string,
                    revision: work_order.get_revision().clone().string,
                    earliest_start_datetime: operation.earliest_start_datetime.to_string(),
                    earliest_finish_datetime: operation.earliest_finish_datetime.to_string(),
                    earliest_allowed_starting_date: work_order
                        .get_order_dates()
                        .earliest_allowed_start_date
                        .to_string(),
                    latest_allowed_finish_date: work_order
                        .get_order_dates()
                        .latest_allowed_finish_date
                        .to_string(),
                    order_type: match work_order.get_order_type().clone() {
                        WorkOrderType::Wdf(_wdf_priority) => "WDF".to_string(),
                        WorkOrderType::Wgn(_wgn_priority) => "WGN".to_string(),
                        WorkOrderType::Wpm(_wpm_priority) => "WPM".to_string(),
                        WorkOrderType::Other => "Missing Work Order Type".to_string(),
                    },
                    priority: match work_order.get_priority().clone() {
                        Priority::IntValue(i) => i.to_string(),
                        Priority::StringValue(s) => s.to_string(),
                    },
                };
                scheduling_overview_data.push(scheduling_overview_data_item);
            }
        }
        scheduling_overview_data
    }
}

/// This is a good point. We should make the type as narrow as possible. This means that we should
/// implement everything that is algorithm specific in the SchedulerAgentAlgorithm. This is a
/// crucial insight.

/// This function should be reformulated? I think that we should make sure to create in such a way
/// that. We need an inner hashmap for each of the different
fn transform_hashmap_to_nested_hashmap(
    resources: HashMap<Resources, HashMap<Period, f64>>,
) -> HashMap<Resources, HashMap<String, f64>> {
    let mut resources_hash_map = HashMap::new();
    for (resource, periods) in resources {
        let mut periods_hash_map = HashMap::new();
        for (period, capacity) in periods {
            periods_hash_map.insert(period.get_period_string(), capacity);
        }
        resources_hash_map.insert(resource, periods_hash_map);
    }
    resources_hash_map
}

#[cfg(test)]
mod tests {

    use chrono::{TimeZone, Utc};
    use shared_messages::strategic::strategic_periods_message::StrategicPeriodsMessage;
    use shared_messages::strategic::strategic_resources_message::StrategicResourcesMessage;
    use shared_messages::strategic::strategic_scheduling_message::SingleWorkOrder;

    use super::scheduler_algorithm::AlgorithmResources;
    use super::scheduler_message::tests::TestRequest;
    use super::scheduler_message::tests::TestResponse;

    use super::{
        scheduler_algorithm::{OptimizedWorkOrders, PriorityQueues},
        *,
    };

    use crate::models::work_order::operation::Operation;
    use crate::models::work_order::order_type::WDFPriority;
    use crate::models::worker_environment::WorkerEnvironment;
    use crate::models::{
        time_environment::period::Period,
        work_order::{
            functional_location::FunctionalLocation, order_dates::OrderDates,
            order_text::OrderText, revision::Revision, status_codes::StatusCodes,
            system_condition::SystemCondition, unloading_point::UnloadingPoint,
        },
    };
    use crate::models::{work_order::*, WorkOrders};
    use shared_messages::strategic::strategic_scheduling_message::StrategicSchedulingMessage;

    #[test]
    fn test_scheduler_agent_initialization() {
        //todo!()
    }

    #[actix_rt::test]
    async fn test_scheduler_agent_handle() {
        let mut work_orders = WorkOrders::new();
        let mut work_load = HashMap::new();

        work_load.insert(Resources::MtnMech, 20.0);
        work_load.insert(Resources::MtnElec, 40.0);
        work_load.insert(Resources::Prodtech, 60.0);

        let work_order = WorkOrder::new(
            2200002020,
            false,
            1000,
            Priority::new_int(1),
            100.0,
            HashMap::new(),
            work_load,
            vec![],
            vec![],
            vec![],
            WorkOrderType::Wdf(WDFPriority::new(1)),
            SystemCondition::new(),
            StatusCodes::new_default(),
            OrderDates::new_test(),
            Revision::new_default(),
            UnloadingPoint::new_default(),
            FunctionalLocation::new_default(),
            OrderText::new_default(),
            false,
        );

        work_orders.insert(work_order.clone());

        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 0, 0, 0).unwrap();
        let end_date = start_date
            + chrono::Duration::days(13)
            + chrono::Duration::hours(23)
            + chrono::Duration::minutes(59)
            + chrono::Duration::seconds(59);
        let period = Period::new(1, start_date, end_date);

        let mut resource_capacity: HashMap<Resources, HashMap<Period, f64>> = HashMap::new();
        let mut resource_loadings: HashMap<Resources, HashMap<Period, f64>> = HashMap::new();

        let mut period_hash_map_150 = HashMap::new();
        let mut period_hash_map_0 = HashMap::new();
        period_hash_map_150.insert(period.clone(), 150.0);
        period_hash_map_0.insert(period.clone(), 0.0);

        resource_capacity.insert(Resources::MtnMech, period_hash_map_150.clone());
        resource_capacity.insert(Resources::MtnElec, period_hash_map_150.clone());
        resource_capacity.insert(Resources::Prodtech, period_hash_map_150.clone());

        resource_loadings.insert(Resources::MtnMech, period_hash_map_0.clone());
        resource_loadings.insert(Resources::MtnElec, period_hash_map_0.clone());
        resource_loadings.insert(Resources::Prodtech, period_hash_map_0.clone());

        let periods: Vec<Period> = vec![Period::new_from_string("2023-W47-48").unwrap()];

        let scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            AlgorithmResources::new(resource_capacity),
            AlgorithmResources::new(resource_loadings),
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            periods,
            true,
        );

        let schedule_single_work_order =
            SingleWorkOrder::new(2200002020, "2023-W47-48".to_string());

        let strategic_scheduling_message =
            StrategicSchedulingMessage::Schedule(schedule_single_work_order);

        let mut manual_resources = HashMap::new();

        let mut period_hash_map = HashMap::new();
        period_hash_map.insert(period.get_period_string(), 300.0);

        manual_resources.insert(Resources::MtnMech, period_hash_map.clone());
        manual_resources.insert(Resources::MtnElec, period_hash_map.clone());
        manual_resources.insert(Resources::Prodtech, period_hash_map.clone());

        let strategic_resources_message =
            StrategicResourcesMessage::new_set_resources(manual_resources);

        let strategic_periods_message = StrategicPeriodsMessage {
            period_lock: HashMap::new(),
        };

        let scheduler_agent = StrategicAgent::new(
            "test".to_string(),
            Arc::new(Mutex::new(SchedulingEnvironment::default())),
            scheduler_agent_algorithm,
            None,
            None,
        );

        let live_scheduler_agent = scheduler_agent.start();

        let test_response: TestResponse = live_scheduler_agent
            .send(TestRequest {})
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            *test_response
                .manual_resources_capacity
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period)
                .unwrap(),
            150.0
        );
        assert_eq!(
            *test_response
                .manual_resources_capacity
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period)
                .unwrap(),
            150.0
        );
        assert_eq!(
            *test_response
                .manual_resources_capacity
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period)
                .unwrap(),
            150.0
        );
    }

    #[test]
    fn test_extract_state_to_scheduler_overview() {
        let mut operations: HashMap<u32, Operation> = HashMap::new();

        let operation_1 = Operation::new(
            10,
            1,
            Resources::MtnMech,
            1.0,
            1.0,
            1.0,
            1,
            Utc.with_ymd_and_hms(2022, 1, 1, 7, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2022, 1, 1, 9, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2022, 1, 1, 7, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2022, 1, 1, 9, 0, 0).unwrap(),
        );

        let operation_2 = Operation::new(
            20,
            1,
            Resources::MtnMech,
            1.0,
            1.0,
            1.0,
            1,
            Utc.with_ymd_and_hms(2022, 1, 1, 7, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2022, 1, 1, 9, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2022, 1, 1, 7, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2022, 1, 1, 9, 0, 0).unwrap(),
        );

        let operation_3 = Operation::new(
            30,
            1,
            Resources::MtnMech,
            1.0,
            1.0,
            1.0,
            1,
            Utc.with_ymd_and_hms(2022, 1, 1, 7, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2022, 1, 1, 9, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2022, 1, 1, 7, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2022, 1, 1, 9, 0, 0).unwrap(),
        );

        operations.insert(10, operation_1);
        operations.insert(20, operation_2);
        operations.insert(30, operation_3);

        let work_order_1 = WorkOrder::new(
            2200002020,
            false,
            1000,
            Priority::new_int(1),
            100.0,
            operations,
            HashMap::new(),
            vec![],
            vec![],
            vec![],
            WorkOrderType::Wdf(WDFPriority::new(1)),
            SystemCondition::new(),
            StatusCodes::new_default(),
            OrderDates::new_test(),
            Revision::new_default(),
            UnloadingPoint::new_default(),
            FunctionalLocation::new_default(),
            OrderText::new_default(),
            false,
        );

        let mut work_orders = WorkOrders::new();

        work_orders.insert(work_order_1);

        let scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            AlgorithmResources::default(),
            AlgorithmResources::default(),
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            vec![],
            true,
        );

        let scheduling_environment = SchedulingEnvironment::new(
            work_orders,
            WorkerEnvironment::new(),
            Vec::<Period>::new(),
            None,
        );

        let scheduler_agent = StrategicAgent::new(
            "test".to_string(),
            Arc::new(Mutex::new(scheduling_environment)),
            scheduler_agent_algorithm,
            None,
            None,
        );

        let scheduler_overview = scheduler_agent.extract_state_to_scheduler_overview();

        assert_eq!(scheduler_overview.len(), 3);
    }
}
