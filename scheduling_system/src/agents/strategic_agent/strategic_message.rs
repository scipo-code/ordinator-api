use actix::prelude::*;
use shared_messages::agent_error::AgentError;
use shared_messages::strategic::strategic_status_message::StrategicStatusMessage;
use shared_messages::strategic::StrategicRequest;
use shared_messages::StatusMessage;
use tracing::{error, info, instrument};

use crate::agents::strategic_agent::StrategicAgent;
use crate::agents::traits::LargeNeighborHoodSearch;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ScheduleIteration {}

impl Handler<ScheduleIteration> for StrategicAgent {
    type Result = ();

    #[instrument(skip_all)]
    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut Self::Context) -> Self::Result {
        let rng: &mut rand::rngs::ThreadRng = &mut rand::thread_rng();

        let mut temporary_schedule = self.strategic_agent_algorithm.clone();

        temporary_schedule.unschedule_random_work_orders(50, rng);

        temporary_schedule.schedule();

        temporary_schedule.calculate_objective();

        if temporary_schedule.get_objective_value()
            < self.strategic_agent_algorithm.get_objective_value()
        {
            self.strategic_agent_algorithm = temporary_schedule;

            // self.update_tactical_agent();
        }

        ctx.notify(ScheduleIteration {});
    }
}

impl Handler<StrategicRequest> for StrategicAgent {
    type Result = Result<String, AgentError>;

    #[instrument(level = "debug", skip_all)]
    fn handle(&mut self, msg: StrategicRequest, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            StrategicRequest::Status(strategic_status_message) => match strategic_status_message {
                StrategicStatusMessage::General => {
                    let scheduling_status = self.scheduling_environment.lock().unwrap().to_string();
                    let strategic_objective = self
                        .strategic_agent_algorithm
                        .get_objective_value()
                        .to_string();

                    let optimized_work_orders =
                        self.strategic_agent_algorithm.get_optimized_work_orders();

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
                    let work_orders = self.strategic_agent_algorithm.get_optimized_work_orders();

                    if !self
                        .strategic_agent_algorithm
                        .get_periods()
                        .iter()
                        .map(|period| period.get_period_string())
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
                            Some(scheduled_period) => {
                                scheduled_period.get_period_string() == period
                            }
                            None => false,
                        })
                        .map(|(work_order_number, _)| *work_order_number)
                        .collect();
                    let message =
                        self.format_selected_work_orders(work_orders_by_period, Some(period));

                    Ok(message)
                }
            },
            StrategicRequest::ScheduleIteration => {
                _ctx.notify(ScheduleIteration {});
                Ok("Schedule iteration completed".to_string())
            }
            StrategicRequest::Scheduling(scheduling_message) => self
                .strategic_agent_algorithm
                .update_scheduling_state(scheduling_message),
            StrategicRequest::Resources(resources_message) => self
                .strategic_agent_algorithm
                .update_resources_state(resources_message),
            StrategicRequest::Periods(periods_message) => {
                let mut scheduling_env_lock = self.scheduling_environment.lock().unwrap();

                let periods = scheduling_env_lock.get_mut_periods();

                for period_id in periods_message.periods.iter() {
                    if periods.last().unwrap().get_id() + 1 == *period_id {
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
            self.strategic_agent_algorithm.get_objective_value()
        )
    }
}

#[cfg(test)]
pub mod tests {
    use core::panic;
    use std::collections::HashSet;

    use super::*;
    use std::collections::HashMap;

    use crate::agents::strategic_agent::strategic_algorithm::{
        OptimizedWorkOrder, OptimizedWorkOrders, PriorityQueues, StrategicAlgorithm,
    };

    use shared_messages::resources::Resources;
    use shared_messages::strategic::strategic_scheduling_message::{
        SingleWorkOrder, StrategicSchedulingMessage,
    };

    use crate::agents::strategic_agent::strategic_algorithm::AlgorithmResources;
    use crate::models::time_environment::period::Period;

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
            true,
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
                .get_optimized_work_orders()
                .get(&2200002020)
                .as_ref()
                .unwrap()
                .get_locked_in_period()
                .as_ref()
                .unwrap()
                .get_period_string(),
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
            true,
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
                .get_optimized_work_order(&2100023841)
                .unwrap()
                .get_locked_in_period(),
            Some(Period::new_from_string("2023-W49-50").unwrap())
        );
        assert_eq!(
            scheduler_agent_algorithm
                .get_optimized_work_order(&2100023841)
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
            true,
        );

        scheduler_agent_algorithm.calculate_objective();

        // This test fails because the objective value in not initialized
        assert_eq!(scheduler_agent_algorithm.get_objective_value(), 2000.0);
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
                objective_value: self.strategic_agent_algorithm.get_objective_value(),
                manual_resources_capacity: self
                    .strategic_agent_algorithm
                    .get_resources_capacities()
                    .inner
                    .clone(),
                manual_resources_loading: self
                    .strategic_agent_algorithm
                    .get_resources_loadings()
                    .inner
                    .clone(),
                priority_queues: PriorityQueues::new(),
                optimized_work_orders: OptimizedWorkOrders::new(
                    self.strategic_agent_algorithm
                        .get_optimized_work_orders()
                        .clone(),
                ),
                periods: self.strategic_agent_algorithm.get_periods().clone(),
            })
        }
    }

    #[test]
    fn test_handler_message_to_frontend() {}
}
