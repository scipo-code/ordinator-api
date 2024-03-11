pub mod display;
pub mod strategic_algorithm;
pub mod strategic_message;

use crate::agents::strategic_agent::strategic_algorithm::StrategicAlgorithm;
use crate::agents::strategic_agent::strategic_message::ScheduleIteration;
use crate::models::SchedulingEnvironment;

use actix::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;

use crate::agents::tactical_agent::TacticalAgent;

use super::SetAddr;

/// This is the primary struct for the scheduler agent.
#[allow(dead_code)]
pub struct StrategicAgent {
    platform: String,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    strategic_agent_algorithm: StrategicAlgorithm,
    tactical_agent_addr: Option<Addr<TacticalAgent>>,
}

impl Actor for StrategicAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        self.strategic_agent_algorithm.populate_priority_queues();
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
        scheduler_agent_algorithm: StrategicAlgorithm,
        tactical_agent_addr: Option<Addr<TacticalAgent>>,
    ) -> Self {
        Self {
            platform,
            scheduling_environment,
            strategic_agent_algorithm: scheduler_agent_algorithm,
            tactical_agent_addr,
        }
    }
}

impl Handler<SetAddr> for StrategicAgent {
    type Result = ();

    fn handle(&mut self, msg: SetAddr, _ctx: &mut Context<Self>) {
        match msg {
            SetAddr::Tactical(addr) => {
                self.tactical_agent_addr = Some(addr);
            }
            _ => {
                println!("The strategic agent received an Addr<T>, where T is not a valid Actor");
                todo!()
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::models::work_order::order_type::WorkOrderType;
    use crate::models::work_order::priority::Priority;
    use chrono::{TimeZone, Utc};

    use super::strategic_algorithm::AlgorithmResources;
    use super::strategic_message::tests::TestRequest;
    use super::strategic_message::tests::TestResponse;
    use std::collections::HashMap;
    use std::collections::HashSet;

    use super::{
        strategic_algorithm::{OptimizedWorkOrders, PriorityQueues},
        *,
    };
    use shared_messages::resources::Resources;

    use crate::models::work_order::operation::Operation;
    use crate::models::work_order::order_type::WDFPriority;
    use crate::models::{
        time_environment::period::Period,
        work_order::{
            functional_location::FunctionalLocation, order_dates::OrderDates,
            order_text::OrderText, revision::Revision, status_codes::StatusCodes,
            system_condition::SystemCondition, unloading_point::UnloadingPoint,
        },
    };
    use crate::models::{work_order::*, WorkOrders};

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

        let scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            AlgorithmResources::new(resource_capacity),
            AlgorithmResources::new(resource_loadings),
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            HashSet::new(),
            periods,
            true,
        );

        let mut manual_resources = HashMap::new();

        let mut period_hash_map = HashMap::new();
        period_hash_map.insert(period.get_period_string(), 300.0);

        manual_resources.insert(Resources::MtnMech, period_hash_map.clone());
        manual_resources.insert(Resources::MtnElec, period_hash_map.clone());
        manual_resources.insert(Resources::Prodtech, period_hash_map.clone());

        let scheduler_agent = StrategicAgent::new(
            "test".to_string(),
            Arc::new(Mutex::new(SchedulingEnvironment::default())),
            scheduler_agent_algorithm,
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

        // let scheduler_overview = scheduler_agent.extract_state_to_scheduler_overview();

        // assert_eq!(scheduler_overview.len(), 3);
    }
}
