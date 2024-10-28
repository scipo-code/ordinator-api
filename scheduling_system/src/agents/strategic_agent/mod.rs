pub mod assert_functions;
pub mod display;
pub mod strategic_algorithm;

use crate::agents::strategic_agent::strategic_algorithm::StrategicAlgorithm;
use crate::agents::traits::LargeNeighborHoodSearch;
use shared_types::scheduling_environment::work_order::status_codes::MaterialStatus;
use shared_types::scheduling_environment::work_order::WorkOrder;
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use shared_types::scheduling_environment::SchedulingEnvironment;

use actix::prelude::*;
use shared_types::agent_error::AgentError;
use shared_types::strategic::strategic_request_status_message::StrategicStatusMessage;
use shared_types::strategic::strategic_response_periods::StrategicResponsePeriods;
use shared_types::strategic::strategic_response_scheduling::StrategicResponseScheduling;
use shared_types::strategic::strategic_response_status::OptimizedWorkOrderResponse;
use shared_types::strategic::strategic_response_status::StrategicResponseStatus;
use shared_types::strategic::strategic_response_status::WorkOrderResponse;
use shared_types::strategic::strategic_response_status::WorkOrdersStatus;
use shared_types::strategic::StrategicRequestMessage;
use shared_types::strategic::StrategicResponseMessage;
use shared_types::AgentExports;
use shared_types::Asset;
use shared_types::SolutionExportMessage;
use strategic_algorithm::optimized_work_orders::StrategicParametersBuilder;
use tracing::event;
use tracing::info;
use tracing::span;
use tracing::Level;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tracing::error;
use tracing::instrument;
use tracing::warn;

use crate::agents::tactical_agent::TacticalAgent;

use super::ScheduleIteration;
use super::SetAddr;
use super::StateLink;
use super::StateLinkWrapper;
use super::UpdateWorkOrderMessage;

pub struct StrategicAgent {
    asset: Asset,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub strategic_agent_algorithm: StrategicAlgorithm,
    pub tactical_agent_addr: Option<Addr<TacticalAgent>>,
}

impl Actor for StrategicAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        self.strategic_agent_algorithm.populate_priority_queues();
        info!("StrategicAgent has started for asset: {}", self.asset);
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
    ) -> Self {
        Self {
            asset,
            scheduling_environment,
            strategic_agent_algorithm,
            tactical_agent_addr,
        }
    }

    pub fn update_tactical_agent(&self) {
        let span = span!(Level::INFO, "strategic_tactical_state_link");
        let _enter = span.enter();
        let locked_scheduling_environment = self.scheduling_environment.lock().unwrap();

        let tactical_periods = locked_scheduling_environment.tactical_periods();
        let tactical_work_orders = self
            .strategic_agent_algorithm
            .tactical_work_orders(tactical_periods.to_vec());

        match &self.tactical_agent_addr {
            Some(tactical_agent_addr) => {
                let state_link = StateLink::Strategic(tactical_work_orders);

                event!(Level::INFO, state_link = ?state_link);
                let state_link_wrapper = StateLinkWrapper::new(state_link, span.clone());

                tactical_agent_addr.do_send(state_link_wrapper);
            }
            None => {
                error!(
                    "The StrategicAgent cannot update the TacticalAgent as its address is not set"
                );
            }
        }
    }
}

impl Handler<ScheduleIteration> for StrategicAgent {
    type Result = ();

    #[instrument(level = "trace", skip_all)]
    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut Self::Context) -> Self::Result {
        self.strategic_agent_algorithm.schedule_forced_work_orders();

        let rng: &mut rand::rngs::ThreadRng = &mut rand::thread_rng();

        let current_objective_value = self.strategic_agent_algorithm.objective_value();

        self.strategic_agent_algorithm
            .unschedule_random_work_orders(50, rng);

        self.strategic_agent_algorithm.schedule();

        self.strategic_agent_algorithm.calculate_objective_value();

        if self.strategic_agent_algorithm.objective_value() < current_objective_value {
            // TODO: If the solution is better we should make a pointer swap instead! Not a overwrite!
            // This feels so much better!
            self.strategic_agent_algorithm
                .make_atomic_pointer_swap_for_with_the_better_strategic_solution();

            info!(strategic_objective_value = %self.strategic_agent_algorithm.objective_value());

            self.update_tactical_agent();
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
    }
}

impl Handler<StrategicRequestMessage> for StrategicAgent {
    type Result = Result<StrategicResponseMessage, AgentError>;

    #[instrument(level = "info", skip_all)]
    fn handle(
        &mut self,
        strategic_request_message: StrategicRequestMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        match strategic_request_message {
            StrategicRequestMessage::Status(strategic_status_message) => {
                match strategic_status_message {
                    StrategicStatusMessage::General => {
                        let strategic_objective = self.strategic_agent_algorithm.objective_value();

                        let optimized_work_orders =
                            self.strategic_agent_algorithm.optimized_work_orders();

                        let number_of_strategic_work_orders = optimized_work_orders.len();

                        let asset = &self.asset;

                        let number_of_periods = self.strategic_agent_algorithm.periods().len();

                        let strategic_response_status = StrategicResponseStatus::new(
                            asset.clone(),
                            strategic_objective,
                            number_of_strategic_work_orders,
                            number_of_periods,
                        );

                        let strategic_response_message =
                            StrategicResponseMessage::Status(strategic_response_status);
                        Ok(strategic_response_message)
                    }
                    StrategicStatusMessage::Period(period) => {
                        let optimized_work_orders =
                            self.strategic_agent_algorithm.optimized_work_orders();

                        if !self
                            .strategic_agent_algorithm
                            .periods()
                            .iter()
                            .map(|period| period.period_string())
                            .collect::<Vec<_>>()
                            .contains(&period)
                        {
                            return Err(AgentError::StateUpdateError(
                                "Period not found in the the scheduling environment".to_string(),
                            ));
                        }

                        let work_orders_by_period: HashMap<WorkOrderNumber, WorkOrderResponse> =
                            self.strategic_agent_algorithm
                                .strategic_periods()
                                .iter()
                                .filter(|(_, sch_per)| match sch_per {
                                    Some(scheduled_period) => {
                                        scheduled_period.period_string() == period
                                    }
                                    None => false,
                                })
                                .map(|(work_order_number, scheduled_period)| {
                                    let work_order = self
                                        .scheduling_environment
                                        .lock()
                                        .unwrap()
                                        .work_orders()
                                        .inner
                                        .get(work_order_number)
                                        .unwrap()
                                        .clone();
                                    let work_order_response = WorkOrderResponse::new(
                                        work_order
                                            .work_order_dates
                                            .earliest_allowed_start_period
                                            .clone(),
                                        work_order.work_order_info.clone(),
                                        work_order.work_order_analytic.vendor,
                                        work_order.work_order_analytic.work_order_weight,
                                        work_order.work_order_analytic.system_status_codes,
                                        work_order.work_order_analytic.user_status_codes,
                                        Some(OptimizedWorkOrderResponse::new(
                                            scheduled_period.clone().unwrap(),
                                            optimized_work_orders
                                                .get(&work_order_number)
                                                .unwrap()
                                                .locked_in_period
                                                .clone(),
                                            optimized_work_orders
                                                .get(&work_order_number)
                                                .unwrap()
                                                .excluded_periods
                                                .clone(),
                                            optimized_work_orders
                                                .get(&work_order_number)
                                                .unwrap()
                                                .latest_period
                                                .clone(),
                                        )),
                                    );
                                    (*work_order_number, work_order_response)
                                })
                                .collect();

                        let work_orders_in_period = WorkOrdersStatus::new(work_orders_by_period);

                        let strategic_response_message =
                            StrategicResponseMessage::WorkOrder(work_orders_in_period);

                        Ok(strategic_response_message)
                    }
                }
            }
            StrategicRequestMessage::Scheduling(scheduling_message) => {
                let scheduling_output: Result<StrategicResponseScheduling, AgentError> = self
                    .strategic_agent_algorithm
                    .update_scheduling_state(scheduling_message);

                self.strategic_agent_algorithm.calculate_objective_value();
                info!(strategic_objective_value = %self.strategic_agent_algorithm.objective_value());
                Ok(StrategicResponseMessage::Scheduling(
                    scheduling_output.unwrap(),
                ))
            }
            StrategicRequestMessage::Resources(resources_message) => {
                let resources_output = self
                    .strategic_agent_algorithm
                    .update_resources_state(resources_message);

                self.strategic_agent_algorithm.calculate_objective_value();
                info!(strategic_objective_value = %self.strategic_agent_algorithm.objective_value());
                Ok(StrategicResponseMessage::Resources(
                    resources_output.unwrap(),
                ))
            }
            StrategicRequestMessage::Periods(periods_message) => {
                let mut scheduling_env_lock = self.scheduling_environment.lock().unwrap();

                let periods = scheduling_env_lock.periods_mut();

                for period_id in periods_message.periods.iter() {
                    if periods.last().unwrap().id() + 1 == *period_id {
                        let new_period =
                            periods.last().unwrap().clone() + chrono::Duration::weeks(2);
                        periods.push(new_period);
                    } else {
                        error!("periods not handled correctly");
                    }
                }
                self.strategic_agent_algorithm.set_periods(periods.to_vec());
                let strategic_response_periods = StrategicResponsePeriods::new(periods.clone());
                Ok(StrategicResponseMessage::Periods(
                    strategic_response_periods,
                ))
            }
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

impl Handler<UpdateWorkOrderMessage> for StrategicAgent {
    type Result = ();

    fn handle(&mut self, update_work_order: UpdateWorkOrderMessage, _ctx: &mut Context<Self>) {
        let locked_scheduling_environment = self.scheduling_environment.lock().unwrap();

        let periods = locked_scheduling_environment.periods().clone();

        let work_order: &WorkOrder = locked_scheduling_environment
            .work_orders()
            .inner
            .get(&update_work_order.0)
            .unwrap();

        let optimized_work_order_builder = StrategicParametersBuilder::new();

        let optimized_work_order = optimized_work_order_builder
            .build_from_work_order(&work_order, &periods)
            .build();
        assert!(work_order.work_order_analytic.work_order_weight == optimized_work_order.weight);
        if let Some(period) =
            Into::<MaterialStatus>::into(work_order.work_order_analytic.user_status_codes.clone())
                .period_delay(&periods)
        {
            assert!(&optimized_work_order.excluded_periods.contains(&period));
        }

        self.strategic_agent_algorithm
            .strategic_parameters
            .inner
            .insert(update_work_order.0, optimized_work_order);
    }
}

impl Handler<SolutionExportMessage> for StrategicAgent {
    type Result = Option<AgentExports>;

    fn handle(&mut self, _msg: SolutionExportMessage, _ctx: &mut Self::Context) -> Self::Result {
        let mut strategic_solution = HashMap::new();
        for (work_order_number, scheduled_period) in
            self.strategic_agent_algorithm.strategic_periods().iter()
        {
            strategic_solution.insert(*work_order_number, scheduled_period.clone().unwrap());
        }
        Some(AgentExports::Strategic(strategic_solution))
    }
}

#[cfg(test)]
mod tests {

    use shared_types::scheduling_environment::work_order::operation::ActivityNumber;
    use shared_types::scheduling_environment::work_order::operation::Work;
    use shared_types::strategic::strategic_request_scheduling_message::SingleWorkOrder;
    use shared_types::strategic::strategic_request_scheduling_message::StrategicSchedulingRequest;
    use shared_types::strategic::Periods;
    use shared_types::strategic::StrategicResources;
    use tests::strategic_algorithm::optimized_work_orders::StrategicParameter;
    use tests::strategic_algorithm::optimized_work_orders::StrategicParameters;
    use unloading_point::UnloadingPoint;

    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::str::FromStr;

    use crate::init::agent_factory::StrategicTacticalSolutionArcSwap;

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
        let operation_1 = Operation::builder(
            ActivityNumber(10),
            unloading_point.clone(),
            Resources::MtnMech,
            Some(Work::from(1.0)),
        )
        .build();

        let operation_2 = Operation::builder(
            ActivityNumber(20),
            unloading_point.clone(),
            Resources::MtnMech,
            Some(Work::from(1.0)),
        )
        .build();

        let operation_3 = Operation::builder(
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
        let period_string: String = "2023-W47-48".to_string();

        let schedule_work_order = SingleWorkOrder::new(work_order_number, period_string);

        let strategic_scheduling_internal =
            StrategicSchedulingRequest::Schedule(schedule_work_order);

        let periods: Vec<Period> = vec![Period::from_str("2023-W47-48").unwrap()];

        let optimized_work_orders = StrategicParameters::new(HashMap::new());
        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::default(),
            StrategicResources::default(),
            PriorityQueues::new(),
            optimized_work_orders,
            StrategicTacticalSolutionArcSwap::default().into(),
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

        scheduler_agent_algorithm.set_optimized_work_order(work_order_number, optimized_work_order);

        scheduler_agent_algorithm
            .update_scheduling_state(strategic_scheduling_internal)
            .unwrap();

        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders()
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
        let schedule_single_work_order =
            SingleWorkOrder::new(work_order_number, "2023-W49-50".to_string());

        let strategic_scheduling_message =
            StrategicSchedulingRequest::Schedule(schedule_single_work_order);

        assert_eq!(
            match strategic_scheduling_message {
                StrategicSchedulingRequest::Schedule(ref schedule_single_work_order) => {
                    schedule_single_work_order.work_order_number.0
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

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::new(capacities),
            StrategicResources::new(loadings),
            PriorityQueues::new(),
            StrategicParameters::new(HashMap::new()),
            StrategicTacticalSolutionArcSwap::default().into(),
            HashSet::new(),
            periods.clone(),
        );

        let optimized_work_order = StrategicParameter::new(
            Some(periods[0].clone()),
            HashSet::new(),
            periods.first().unwrap().clone(),
            1000,
            work_load,
        );

        scheduler_agent_algorithm.set_optimized_work_order(work_order_number, optimized_work_order);

        scheduler_agent_algorithm
            .update_scheduling_state(strategic_scheduling_message)
            .unwrap();

        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_order(&work_order_number)
                .unwrap()
                .locked_in_period,
            Some(Period::from_str("2023-W49-50").unwrap())
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .strategic_periods()
                .get(&work_order_number)
                .unwrap(),
            None
        );
        // assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("VEN_MECH".to_string(), "2023-W49-50".to_string()), 16.0);
    }

    //
    #[test]
    fn test_calculate_objective_value() {
        let mut optimized_work_orders = StrategicParameters::new(HashMap::new());

        let optimized_work_order = StrategicParameter::new(
            Some(Period::from_str("2023-W49-50").unwrap()),
            HashSet::new(),
            Period::from_str("2023-W47-48").unwrap(),
            1000,
            HashMap::new(),
        );

        optimized_work_orders
            .insert_optimized_work_order(WorkOrderNumber(2100023841), optimized_work_order);

        let mut strategic_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            StrategicResources::default(),
            StrategicResources::default(),
            PriorityQueues::new(),
            optimized_work_orders,
            StrategicTacticalSolutionArcSwap::default().into(),
            HashSet::new(),
            vec![],
        );

        strategic_agent_algorithm.calculate_objective_value();

        // This test fails because the objective value in not initialized
        assert_eq!(strategic_agent_algorithm.objective_value(), 2000.0);
    }

    pub struct TestRequest {}

    impl Message for TestRequest {
        type Result = Option<TestResponse>;
    }

    #[allow(dead_code)]
    pub struct TestResponse {
        pub objective_value: f64,
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
                objective_value: self.strategic_agent_algorithm.objective_value(),
                manual_resources_capacity: self
                    .strategic_agent_algorithm
                    .resources_capacities()
                    .inner
                    .clone(),
                manual_resources_loading: self
                    .strategic_agent_algorithm
                    .resources_loadings()
                    .inner
                    .clone(),
                priority_queues: PriorityQueues::new(),
                optimized_work_orders: StrategicParameters::new(
                    self.strategic_agent_algorithm
                        .optimized_work_orders()
                        .clone(),
                ),
                periods: self.strategic_agent_algorithm.periods().clone(),
            })
        }
    }
}
