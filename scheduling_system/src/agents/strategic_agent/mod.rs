pub mod display;
pub mod strategic_algorithm;

use crate::agents::strategic_agent::strategic_algorithm::StrategicAlgorithm;
use crate::agents::traits::LargeNeighborHoodSearch;
use crate::models::SchedulingEnvironment;

use actix::prelude::*;
use shared_messages::agent_error::AgentError;
use shared_messages::strategic::strategic_status_message::StrategicStatusMessage;
use shared_messages::strategic::StrategicRequest;
use shared_messages::StatusMessage;
use tracing::info;

use std::sync::Arc;
use std::sync::Mutex;
use tracing::error;
use tracing::instrument;
use tracing::warn;

use crate::agents::tactical_agent::TacticalAgent;

use super::SendState;
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
        warn!("StrategicAgent has started for platform: {}", self.platform);
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
        strategic_agent_algorithm: StrategicAlgorithm,
        tactical_agent_addr: Option<Addr<TacticalAgent>>,
    ) -> Self {
        Self {
            platform,
            scheduling_environment,
            strategic_agent_algorithm,
            tactical_agent_addr,
        }
    }

    pub fn update_tactical_agent(&self) {
        let tactical_work_orders = self.strategic_agent_algorithm.tactical_work_orders();

        match &self.tactical_agent_addr {
            Some(tactical_agent_addr) => {
                tactical_agent_addr.do_send(SendState::Strategic(tactical_work_orders));
            }
            None => {
                error!(
                    "The StrategicAgent cannot update the TacticalAgent as its address is not set"
                );
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ScheduleIteration {}

impl Handler<ScheduleIteration> for StrategicAgent {
    type Result = ();

    #[instrument(level = "trace", skip_all)]
    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut Self::Context) -> Self::Result {
        let rng: &mut rand::rngs::ThreadRng = &mut rand::thread_rng();

        let mut temporary_schedule = self.strategic_agent_algorithm.clone();

        temporary_schedule.unschedule_random_work_orders(50, rng);

        temporary_schedule.schedule();

        temporary_schedule.calculate_objective();

        if temporary_schedule.objective_value() < self.strategic_agent_algorithm.objective_value() {
            self.strategic_agent_algorithm = temporary_schedule;
            
            info!(strategic_objective_value = %self.strategic_agent_algorithm.objective_value());

            self.update_tactical_agent();
        }
        ctx.notify(ScheduleIteration {});
    }
}

impl Handler<StrategicRequest> for StrategicAgent {
    type Result = Result<String, AgentError>;

    #[instrument(level = "info", skip_all)]
    fn handle(&mut self, msg: StrategicRequest, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            StrategicRequest::Status(strategic_status_message) => match strategic_status_message {
                StrategicStatusMessage::General => {
                    let scheduling_status = self.scheduling_environment.lock().unwrap().to_string();
                    let strategic_objective =
                        self.strategic_agent_algorithm.objective_value().to_string();

                    let optimized_work_orders =
                        self.strategic_agent_algorithm.optimized_work_orders();

                    let number_of_strategic_work_orders = optimized_work_orders.len();
                    let mut scheduled_count = 0;
                    for optimized_work_order in optimized_work_orders.values() {
                        if optimized_work_order.get_scheduled_period().is_some() {
                            scheduled_count += 1;
                        }
                    }

                    let scheduling_status = format!(
                    "{}\nWith objectives: \n  strategic objective of: {}\n    {} of {} work orders scheduled",
                    scheduling_status, strategic_objective, scheduled_count, number_of_strategic_work_orders
                    );
                    Ok(scheduling_status)
                }
                StrategicStatusMessage::Period(period) => {
                    let work_orders = self.strategic_agent_algorithm.optimized_work_orders();

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

                    let work_orders_by_period: Vec<u32> = work_orders
                        .iter()
                        .filter(|(_, opt_wo)| match opt_wo.get_scheduled_period() {
                            Some(scheduled_period) => scheduled_period.period_string() == period,
                            None => false,
                        })
                        .map(|(work_order_number, _)| *work_order_number)
                        .collect();
                    let message =
                        self.format_selected_work_orders(work_orders_by_period, Some(period));

                    Ok(message)
                }
            },

            StrategicRequest::Scheduling(scheduling_message) => self
                .strategic_agent_algorithm
                .update_scheduling_state(scheduling_message),
            StrategicRequest::Resources(resources_message) => self
                .strategic_agent_algorithm
                .update_resources_state(resources_message),
            StrategicRequest::Periods(periods_message) => {
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
                Ok("Periods updated".to_string())
            }
        }
    }
}

impl Handler<StatusMessage> for StrategicAgent {
    type Result = String;

    fn handle(&mut self, _msg: StatusMessage, _ctx: &mut Self::Context) -> Self::Result {
        format!(
            "Objective: {}",
            self.strategic_agent_algorithm.objective_value()
        )
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

    use chrono::{TimeZone, Utc};
    use shared_messages::strategic::strategic_scheduling_message::SingleWorkOrder;
    use shared_messages::strategic::strategic_scheduling_message::StrategicSchedulingMessage;
    use tests::strategic_algorithm::OptimizedWorkOrder;

    use super::strategic_algorithm::AlgorithmResources;
    use std::collections::HashMap;
    use std::collections::HashSet;

    use super::{
        strategic_algorithm::{OptimizedWorkOrders, PriorityQueues},
        *,
    };
    use shared_messages::resources::Resources;

    use crate::models::time_environment::period::Period;
    use crate::models::work_order::operation::Operation;
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

        let work_order = WorkOrder::default();

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
        );

        let mut manual_resources = HashMap::new();

        let mut period_hash_map = HashMap::new();
        period_hash_map.insert(period.period_string(), 300.0);

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

        let operation_1 = Operation::new_test(10, Resources::MtnMech, 1.0);

        let operation_2 = Operation::new_test(20, Resources::MtnMech, 1.0);

        let operation_3 = Operation::new_test(30, Resources::MtnMech, 1.0);

        operations.insert(10, operation_1);
        operations.insert(20, operation_2);
        operations.insert(30, operation_3);

        let work_order_1 = WorkOrder::default();

        let mut work_orders = WorkOrders::new();

        work_orders.insert(work_order_1);
    }

    #[test]
    fn test_update_scheduler_state() {
        let period_string: String = "2023-W47-48".to_string();

        let schedule_work_order = SingleWorkOrder::new(2200002020, period_string);

        let strategic_scheduling_internal =
            StrategicSchedulingMessage::Schedule(schedule_work_order);

        let periods: Vec<Period> = vec![Period::new_from_string("2023-W47-48").unwrap()];

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            AlgorithmResources::default(),
            AlgorithmResources::default(),
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            HashSet::new(),
            periods.clone(),
        );

        let optimized_work_order = OptimizedWorkOrder::new(
            None,
            Some(periods[0].clone()),
            HashSet::new(),
            None,
            1000,
            HashMap::new(),
        );

        scheduler_agent_algorithm.set_optimized_work_order(2200002020, optimized_work_order);

        scheduler_agent_algorithm
            .update_scheduling_state(strategic_scheduling_internal)
            .unwrap();

        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders()
                .get(&2200002020)
                .as_ref()
                .unwrap()
                .get_locked_in_period()
                .as_ref()
                .unwrap()
                .period_string(),
            "2023-W47-48"
        );
    }

    #[test]
    fn test_input_scheduler_message_from() {
        let schedule_single_work_order =
            SingleWorkOrder::new(2100023841, "2023-W49-50".to_string());

        let strategic_scheduling_message =
            StrategicSchedulingMessage::Schedule(schedule_single_work_order);

        assert_eq!(
            match strategic_scheduling_message {
                StrategicSchedulingMessage::Schedule(ref schedule_single_work_order) => {
                    schedule_single_work_order.get_work_order_number()
                }
                _ => panic!("wrong message type"),
            },
            2100023841
        );

        assert_eq!(
            match strategic_scheduling_message {
                StrategicSchedulingMessage::Schedule(ref schedule_single_work_order) => {
                    schedule_single_work_order.get_period_string()
                }
                _ => panic!("wrong message type"),
            },
            "2023-W49-50".to_string()
        );

        let mut work_load = HashMap::new();

        work_load.insert(Resources::VenMech, 16.0);

        let periods: Vec<Period> = vec![Period::new_from_string("2023-W49-50").unwrap()];

        let mut capacities = HashMap::new();
        let mut loadings = HashMap::new();

        let mut periods_hash_map_0 = HashMap::new();
        let mut periods_hash_map_16 = HashMap::new();

        periods_hash_map_0.insert(Period::new_from_string("2023-W49-50").unwrap(), 0.0);

        periods_hash_map_16.insert(Period::new_from_string("2023-W49-50").unwrap(), 16.0);

        capacities.insert(Resources::VenMech, periods_hash_map_16);

        loadings.insert(Resources::VenMech, periods_hash_map_0);

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            AlgorithmResources::new(capacities),
            AlgorithmResources::new(loadings),
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            HashSet::new(),
            periods.clone(),
        );

        let optimized_work_order = OptimizedWorkOrder::new(
            None,
            Some(periods[0].clone()),
            HashSet::new(),
            None,
            1000,
            work_load,
        );

        scheduler_agent_algorithm.set_optimized_work_order(2100023841, optimized_work_order);

        scheduler_agent_algorithm
            .update_scheduling_state(strategic_scheduling_message)
            .unwrap();

        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_order(&2100023841)
                .unwrap()
                .get_locked_in_period(),
            Some(Period::new_from_string("2023-W49-50").unwrap())
        );
        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_order(&2100023841)
                .unwrap()
                .get_scheduled_period(),
            None
        );
        // assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("VEN_MECH".to_string(), "2023-W49-50".to_string()), 16.0);
    }

    //
    #[test]
    fn test_calculate_objective_value() {
        let mut optimized_work_orders = OptimizedWorkOrders::new(HashMap::new());

        let optimized_work_order = OptimizedWorkOrder::new(
            Some(Period::new_from_string("2023-W49-50").unwrap()),
            Some(Period::new_from_string("2023-W49-50").unwrap()),
            HashSet::new(),
            Some(Period::new_from_string("2023-W47-48").unwrap()),
            1000,
            HashMap::new(),
        );

        optimized_work_orders.insert_optimized_work_order(2100023841, optimized_work_order);

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            AlgorithmResources::default(),
            AlgorithmResources::default(),
            PriorityQueues::new(),
            optimized_work_orders,
            HashSet::new(),
            vec![],
        );

        scheduler_agent_algorithm.calculate_objective();

        // This test fails because the objective value in not initialized
        assert_eq!(scheduler_agent_algorithm.objective_value(), 2000.0);
    }

    pub struct TestRequest {}

    impl Message for TestRequest {
        type Result = Option<TestResponse>;
    }

    pub struct TestResponse {
        pub objective_value: f64,
        pub manual_resources_capacity: HashMap<Resources, HashMap<Period, f64>>,
        pub manual_resources_loading: HashMap<Resources, HashMap<Period, f64>>,
        pub priority_queues: PriorityQueues<u32, u32>,
        pub optimized_work_orders: OptimizedWorkOrders,
        pub periods: Vec<Period>,
    }

    impl Handler<TestRequest> for StrategicAgent {
        type Result = Option<TestResponse>; // Or relevant part of the state

        fn handle(&mut self, _msg: TestRequest, _: &mut Context<Self>) -> Self::Result {
            // Return the state or part of it
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
                optimized_work_orders: OptimizedWorkOrders::new(
                    self.strategic_agent_algorithm
                        .optimized_work_orders()
                        .clone(),
                ),
                periods: self.strategic_agent_algorithm.periods().clone(),
            })
        }
    }

    #[test]
    fn test_handler_message_to_frontend() {}
}
