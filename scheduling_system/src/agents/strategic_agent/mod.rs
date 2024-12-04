pub mod assert_functions;
pub mod display;
pub mod message_handlers;
pub mod strategic_algorithm;

use crate::agents::strategic_agent::strategic_algorithm::StrategicAlgorithm;
use crate::agents::traits::LargeNeighborHoodSearch;
use anyhow::Result;
use shared_types::scheduling_environment::SchedulingEnvironment;

use actix::prelude::*;
use shared_types::Asset;
use tracing::event;
use tracing::Level;

use std::sync::Arc;
use std::sync::Mutex;
use tracing::instrument;
use tracing::warn;

use super::orchestrator::NotifyOrchestrator;
use super::ScheduleIteration;
use crate::agents::tactical_agent::TacticalAgent;

pub struct StrategicAgent {
    asset: Asset,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub strategic_algorithm: StrategicAlgorithm,
    pub tactical_agent_addr: Option<Addr<TacticalAgent>>,
    pub notify_orchestrator: NotifyOrchestrator,
}

impl Actor for StrategicAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        self.strategic_algorithm.populate_priority_queues();
        event!(
            Level::INFO,
            "StrategicAgent has started for asset: {}",
            self.asset
        );
        self.strategic_algorithm
            .schedule()
            .expect("StrategicAlgorithm.schedule() method failed");
        ctx.notify(ScheduleIteration {})
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        println!("SchedulerAgent is stopped");
    }
}

impl StrategicAgent {
    pub fn new(
        asset: Asset,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        strategic_agent_algorithm: StrategicAlgorithm,
        tactical_agent_addr: Option<Addr<TacticalAgent>>,
        notify_orchestrator: NotifyOrchestrator,
    ) -> Self {
        Self {
            asset,
            scheduling_environment,
            strategic_algorithm: strategic_agent_algorithm,
            tactical_agent_addr,
            notify_orchestrator,
        }
    }
}

impl Handler<ScheduleIteration> for StrategicAgent {
    type Result = Result<()>;

    #[instrument(level = "trace", skip_all)]
    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut Self::Context) -> Self::Result {
        // So here we should load instead! Yes we should load in the data and then continue
        self.strategic_algorithm.load_shared_solution();

        self.strategic_algorithm.schedule_forced_work_orders();

        let rng: &mut rand::rngs::ThreadRng = &mut rand::thread_rng();

        self.strategic_algorithm.calculate_objective_value();
        let old_strategic_solution = self.strategic_algorithm.strategic_solution.clone();

        self.strategic_algorithm
            .unschedule_random_work_orders(50, rng)
            .expect("Unscheduling random work order should always be possible");

        self.strategic_algorithm
            .schedule()
            .expect("StrategicAlgorithm.schedule method failed");
        // self.assert_aggregated_load().unwrap();
        let (tardiness, penalty) = self.strategic_algorithm.calculate_objective_value();

        if self.strategic_algorithm.strategic_solution.objective_value
            < old_strategic_solution.objective_value
        {
            self.strategic_algorithm.make_atomic_pointer_swap();

            event!(Level::INFO, strategic_objective_value = %self.strategic_algorithm.strategic_solution.objective_value,
                scheduled_work_orders = ?self.strategic_algorithm.strategic_solution.strategic_periods.iter().filter(|ele| ele.1.is_some()).count(),
                total_work_orders = ?self.strategic_algorithm.strategic_solution.strategic_periods.len(),
                tardiness = tardiness,
                penalty = penalty,
                percentage_utilization_by_period = ?self.strategic_algorithm.calculate_utilization(),
            );
        } else {
            self.strategic_algorithm.strategic_solution = old_strategic_solution;
        }

        ctx.wait(
            tokio::time::sleep(tokio::time::Duration::from_millis(
                dotenvy::var("STRATEGIC_THROTTLING")
                    .expect("The STRATEGIC_THROTTLING environment variable should always be set")
                    .parse::<u64>()
                    .expect("The STRATEGIC_THROTTLING environment variable have to be an u64 compatible type"),
            ))
            .into_actor(self),
        );

        ctx.notify(ScheduleIteration {});
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use operation::OperationBuilder;
    use shared_types::scheduling_environment::work_order::operation::ActivityNumber;
    use shared_types::scheduling_environment::work_order::operation::Work;
    use shared_types::strategic::strategic_request_scheduling_message::ScheduleChange;
    use shared_types::strategic::strategic_request_scheduling_message::StrategicSchedulingRequest;
    use shared_types::strategic::Periods;
    use shared_types::strategic::StrategicObjectiveValue;
    use shared_types::strategic::StrategicResources;
    use strategic_algorithm::ForcedWorkOrder;
    use tests::strategic_algorithm::strategic_parameters::StrategicParameter;
    use tests::strategic_algorithm::strategic_parameters::StrategicParameters;
    use unloading_point::UnloadingPoint;

    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::str::FromStr;

    use crate::agents::ArcSwapSharedSolution;

    use super::{strategic_algorithm::PriorityQueues, *};
    use shared_types::scheduling_environment::worker_environment::resources::Resources;

    use shared_types::scheduling_environment::work_order::operation::Operation;
    use shared_types::scheduling_environment::work_order::*;
    use shared_types::scheduling_environment::WorkOrders;

    use shared_types::scheduling_environment::time_environment::period::Period;

    // #[test]
    // fn test_scheduler_agent_handle() {
    //     let mut work_orders = WorkOrders::default();
    //     let mut work_load = HashMap::new();

    //     work_load.insert(Resources::MtnMech, 20.0);
    //     work_load.insert(Resources::MtnElec, 40.0);
    //     work_load.insert(Resources::Prodtech, 60.0);

    //     let work_order = WorkOrder::default();

    //     work_orders.insert(work_order.clone());

    //     let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 0, 0, 0).unwrap();
    //     let end_date = start_date
    //         + chrono::Duration::days(13)
    //         + chrono::Duration::hours(23)
    //         + chrono::Duration::minutes(59)
    //         + chrono::Duration::seconds(59);
    //     let period = Period::new(1, start_date, end_date);

    //     let mut resource_capacity: HashMap<Resources, Periods> = HashMap::new();
    //     let mut resource_loadings: HashMap<Resources, Periods> = HashMap::new();

    //     let mut period_hash_map_150 = HashMap::new();
    //     let mut period_hash_map_0 = HashMap::new();
    //     period_hash_map_150.insert(period.clone(), 150.0);
    //     period_hash_map_0.insert(period.clone(), 0.0);

    //     resource_capacity.insert(Resources::MtnMech, Periods(period_hash_map_150.clone()));
    //     resource_capacity.insert(Resources::MtnElec, Periods(period_hash_map_150.clone()));
    //     resource_capacity.insert(Resources::Prodtech, Periods(period_hash_map_150.clone()));

    //     resource_loadings.insert(Resources::MtnMech, Periods(period_hash_map_0.clone()));
    //     resource_loadings.insert(Resources::MtnElec, Periods(period_hash_map_0.clone()));
    //     resource_loadings.insert(Resources::Prodtech, Periods(period_hash_map_0.clone()));

    //     let periods: Vec<Period> = vec![Period::from_str("2023-W47-48").unwrap()];

    //     let scheduler_agent_algorithm = StrategicAlgorithm::new(
    //         0.0,
    //         StrategicResources::new(resource_capacity),
    //         StrategicResources::new(resource_loadings),
    //         PriorityQueues::new(),
    //         OptimizedWorkOrders::new(HashMap::new()),
    //         HashSet::new(),
    //         periods,
    //     );

    //     let mut manual_resources = HashMap::new();

    //     let mut period_hash_map = HashMap::new();
    //     period_hash_map.insert(period.period_string(), 300.0);

    //     manual_resources.insert(Resources::MtnMech, period_hash_map.clone());
    //     manual_resources.insert(Resources::MtnElec, period_hash_map.clone());
    //     manual_resources.insert(Resources::Prodtech, period_hash_map.clone());

    //     let scheduler_agent = StrategicAgent::new(
    //         Asset::DF,
    //         Arc::new(Mutex::new(SchedulingEnvironment::default())),
    //         scheduler_agent_algorithm,
    //         None,
    //     );

    //     let strategic_addr = scheduler_agent.start();

    //     let test_response = strategic_addr.send(TestRequest {});

    //     assert_eq!(
    //         *test_response
    //             .manual_resources_capacity
    //             .get(&Resources::MtnMech)
    //             .unwrap()
    //             .0
    //             .get(&period)
    //             .unwrap(),
    //         150.0
    //     );
    //     assert_eq!(
    //         *test_response
    //             .manual_resources_capacity
    //             .get(&Resources::MtnElec)
    //             .unwrap()
    //             .0
    //             .get(&period)
    //             .unwrap(),
    //         150.0
    //     );
    //     assert_eq!(
    //         *test_response
    //             .manual_resources_capacity
    //             .get(&Resources::Prodtech)
    //             .unwrap()
    //             .0
    //             .get(&period)
    //             .unwrap(),
    //         150.0
    //     );
    // }

    #[test]
    fn test_extract_state_to_scheduler_overview() {
        let mut operations: HashMap<u32, Operation> = HashMap::new();

        let unloading_point = UnloadingPoint::default();
        let operation_1 = OperationBuilder::new(
            ActivityNumber(10),
            unloading_point.clone(),
            Resources::MtnMech,
            Some(Work::from(1.0)),
        )
        .build();

        let operation_2 = OperationBuilder::new(
            ActivityNumber(20),
            unloading_point.clone(),
            Resources::MtnMech,
            Some(Work::from(1.0)),
        )
        .build();

        let operation_3 = OperationBuilder::new(
            ActivityNumber(30),
            unloading_point.clone(),
            Resources::MtnMech,
            Some(Work::from(1.0)),
        )
        .build();

        operations.insert(10, operation_1);
        operations.insert(20, operation_2);
        operations.insert(30, operation_3);

        let work_order_1 = WorkOrder::work_order_test();

        let mut work_orders = WorkOrders::default();

        work_orders.insert(work_order_1);
    }

    #[test]
    fn test_update_scheduler_state() {
        let work_order_number = WorkOrderNumber(2200002020);
        let vec_work_order_number = vec![work_order_number];
        let period_string: String = "2023-W47-48".to_string();

        let schedule_work_order = ScheduleChange::new(vec_work_order_number, period_string);

        let strategic_scheduling_internal =
            StrategicSchedulingRequest::Schedule(schedule_work_order);

        let periods: Vec<Period> = vec![Period::from_str("2023-W47-48").unwrap()];

        let optimized_work_orders =
            StrategicParameters::new(HashMap::new(), StrategicResources::default());

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            PriorityQueues::new(),
            optimized_work_orders,
            ArcSwapSharedSolution::default().into(),
            HashSet::new(),
            periods.clone(),
        );

        let optimized_work_order = StrategicParameter::new(
            Some(periods[0].clone()),
            HashSet::new(),
            periods.first().unwrap().clone(),
            1000,
            HashMap::new(),
        );

        scheduler_agent_algorithm.set_strategic_parameter(work_order_number, optimized_work_order);

        scheduler_agent_algorithm
            .update_scheduling_state(strategic_scheduling_internal)
            .unwrap();

        assert_eq!(
            scheduler_agent_algorithm
                .strategic_parameters
                .strategic_work_order_parameters
                .get(&work_order_number)
                .as_ref()
                .unwrap()
                .locked_in_period
                .as_ref()
                .unwrap()
                .period_string(),
            "2023-W47-48"
        );
    }

    #[test]
    fn test_input_scheduler_message_from() {
        let work_order_number = WorkOrderNumber(2100023841);
        let vec_work_order_number = vec![work_order_number];
        let schedule_single_work_order =
            ScheduleChange::new(vec_work_order_number, "2023-W49-50".to_string());

        let strategic_scheduling_message =
            StrategicSchedulingRequest::Schedule(schedule_single_work_order);

        assert_eq!(
            match strategic_scheduling_message {
                StrategicSchedulingRequest::Schedule(ref schedule_single_work_order) => {
                    schedule_single_work_order.work_order_number[0].0
                }
                _ => panic!("wrong message type"),
            },
            work_order_number.0
        );

        assert_eq!(
            match strategic_scheduling_message {
                StrategicSchedulingRequest::Schedule(ref schedule_single_work_order) => {
                    schedule_single_work_order.period_string()
                }
                _ => panic!("wrong message type"),
            },
            "2023-W49-50".to_string()
        );

        let mut work_load = HashMap::new();

        work_load.insert(Resources::VenMech, Work::from(16.0));

        let periods: Vec<Period> = vec![Period::from_str("2023-W49-50").unwrap()];

        let mut capacities = HashMap::new();
        let mut loadings = HashMap::new();

        let mut periods_hash_map_0 = HashMap::new();
        let mut periods_hash_map_16 = HashMap::new();

        periods_hash_map_0.insert(Period::from_str("2023-W49-50").unwrap(), Work::from(0.0));

        periods_hash_map_16.insert(Period::from_str("2023-W49-50").unwrap(), Work::from(16.0));

        capacities.insert(Resources::VenMech, Periods(periods_hash_map_16));

        loadings.insert(Resources::VenMech, Periods(periods_hash_map_0));

        let mut strategic_algorithm = StrategicAlgorithm::new(
            PriorityQueues::new(),
            StrategicParameters::new(HashMap::new(), StrategicResources::default()),
            ArcSwapSharedSolution::default().into(),
            HashSet::new(),
            periods.clone(),
        );

        let strategic_parameter = StrategicParameter::new(
            Some(periods[0].clone()),
            HashSet::new(),
            periods.first().unwrap().clone(),
            1000,
            work_load,
        );

        strategic_algorithm.set_strategic_parameter(work_order_number, strategic_parameter);

        strategic_algorithm
            .update_scheduling_state(strategic_scheduling_message)
            .unwrap();

        assert_eq!(
            strategic_algorithm
                .strategic_parameter(&work_order_number)
                .unwrap()
                .locked_in_period,
            Some(Period::from_str("2023-W49-50").unwrap())
        );

        assert_eq!(
            strategic_algorithm
                .strategic_periods()
                .get(&work_order_number),
            None
        );
        // assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("VEN_MECH".to_string(), "2023-W49-50".to_string()), 16.0);
    }

    #[test]
    fn test_calculate_objective_value() {
        let work_order_number = WorkOrderNumber(2100023841);
        let mut strategic_parameters =
            StrategicParameters::new(HashMap::new(), StrategicResources::default());

        let strategic_parameter = StrategicParameter::new(
            Some(Period::from_str("2023-W49-50").unwrap()),
            HashSet::new(),
            Period::from_str("2023-W47-48").unwrap(),
            1000,
            HashMap::new(),
        );

        strategic_parameters
            .insert_strategic_parameter(WorkOrderNumber(2100023841), strategic_parameter);

        let mut strategic_algorithm = StrategicAlgorithm::new(
            PriorityQueues::new(),
            strategic_parameters,
            ArcSwapSharedSolution::default().into(),
            HashSet::new(),
            vec![],
        );

        strategic_algorithm
            .strategic_solution
            .strategic_periods
            .insert(work_order_number, None);

        strategic_algorithm
            .schedule_forced_work_order(&(work_order_number, ForcedWorkOrder::Locked));

        strategic_algorithm.calculate_objective_value();

        assert_eq!(strategic_algorithm.strategic_solution.objective_value, 2000);
    }

    pub struct TestRequest {}

    impl Message for TestRequest {
        type Result = Option<TestResponse>;
    }

    #[allow(dead_code)]
    pub struct TestResponse {
        pub objective_value: StrategicObjectiveValue,
        pub manual_resources_capacity: HashMap<Resources, Periods>,
        pub manual_resources_loading: HashMap<Resources, Periods>,
        pub priority_queues: PriorityQueues<WorkOrderNumber, u64>,
        pub optimized_work_orders: StrategicParameters,
        pub periods: Vec<Period>,
    }

    impl Handler<TestRequest> for StrategicAgent {
        type Result = Option<TestResponse>;

        fn handle(&mut self, _msg: TestRequest, _: &mut Context<Self>) -> Self::Result {
            Some(TestResponse {
                objective_value: self.strategic_algorithm.strategic_solution.objective_value,
                manual_resources_capacity: self
                    .strategic_algorithm
                    .resources_capacities()
                    .inner
                    .clone(),
                manual_resources_loading: self
                    .strategic_algorithm
                    .resources_loadings()
                    .inner
                    .clone(),
                priority_queues: PriorityQueues::new(),
                optimized_work_orders: StrategicParameters::new(
                    self.strategic_algorithm
                        .strategic_parameters
                        .strategic_work_order_parameters
                        .clone(),
                    StrategicResources::default(),
                ),
                periods: self.strategic_algorithm.periods().clone(),
            })
        }
    }
}
