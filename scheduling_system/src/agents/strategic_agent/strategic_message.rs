use actix::prelude::*;
use serde::{Deserialize, Serialize};
use shared_messages::status::StatusRequest;
use shared_messages::strategic::strategic_status_message::StrategicStatusMessage;
use shared_messages::strategic::StrategicRequest;
use std::collections::HashMap;
use std::fmt::Write;
use std::fmt::{self, Display};
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, instrument, trace};

use crate::agents::strategic_agent::{self, StrategicAgent};
use crate::api::websocket_agent::WebSocketAgent;
use crate::models::time_environment::period::Period;
use shared_messages::resources::Resources;

struct SchedulerResources<'a>(&'a HashMap<(Resources, String), f64>);

impl Display for SchedulerResources<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "--------------------------")?;
        for ((resource, period), capacity) in self.0 {
            writeln!(
                f,
                "Resource: {:?}\nPeriod: {}\nCapacity: {}",
                resource, period, capacity
            )?;
        }
        write!(f, "--------------------------")
    }
}

#[derive(Deserialize, Debug)]
pub struct UpdatePeriod {
    pub periods: Vec<Period>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ScheduleIteration {}

impl Handler<ScheduleIteration> for StrategicAgent {
    type Result = ResponseActFuture<Self, ()>;

    #[instrument(skip(self, _msg, ctx))]
    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut Self::Context) -> Self::Result {
        // event!(tracing::Level::INFO , "schedule_iteration_message");
        let rng: &mut rand::rngs::ThreadRng = &mut rand::thread_rng();

        let previous_schedule = self.scheduler_agent_algorithm.clone();

        self.scheduler_agent_algorithm
            .unschedule_random_work_orders(50, rng);

        self.scheduler_agent_algorithm.schedule_normal_work_orders();

        self.scheduler_agent_algorithm.schedule_forced_work_orders();

        self.scheduler_agent_algorithm.calculate_objective();

        if previous_schedule.get_objective_value()
            < self.scheduler_agent_algorithm.get_objective_value()
        {
            self.scheduler_agent_algorithm = previous_schedule;
        }

        dbg!(&self.scheduler_agent_algorithm.get_objective_value());

        debug!(
            "Objective value: {}",
            self.scheduler_agent_algorithm.get_objective_value()
        );

        #[cfg(debug_assertions)]
        //self.scheduler_agent_algorithm.check_strategic_state();
        let actor_addr = ctx.address().clone();

        let fut = async move {
            sleep(Duration::from_secs(0)).await;
            actor_addr.do_send(ScheduleIteration {});
        };

        Box::pin(actix::fut::wrap_future::<_, Self>(fut))
    }
}

enum MessageToFrontend {
    Overview,
    Loading,
    Period,
}

impl Message for MessageToFrontend {
    type Result = ();
}

#[derive(Serialize)]
pub struct SuccesMessage {}

impl Message for SuccesMessage {
    type Result = ();
}

#[derive(serde::Serialize, Debug)]
pub struct LoadingMessage {
    pub frontend_message_type: String,
    pub manual_resources_loading: HashMap<Resources, HashMap<String, f64>>,
}

impl Message for LoadingMessage {
    type Result = ();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OverviewMessage {
    pub frontend_message_type: String,
    pub scheduling_overview_data: Vec<strategic_agent::SchedulingOverviewData>,
}

/// The Scheduler Output should contain all that is needed to make
impl Message for OverviewMessage {
    type Result = ();
}

#[derive(Serialize, Debug)]
pub struct PeriodMessage {
    pub frontend_message_type: String,
    pub periods: Vec<Period>,
}

impl Message for PeriodMessage {
    type Result = ();
}

impl Handler<StrategicRequest> for StrategicAgent {
    type Result = ();

    fn handle(&mut self, msg: StrategicRequest, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            StrategicRequest::Status(strategic_status_message) => match strategic_status_message {
                StrategicStatusMessage::General => {
                    let scheduling_status = self.scheduling_environment.lock().unwrap().to_string();
                    let strategic_objective = self
                        .scheduler_agent_algorithm
                        .get_objective_value()
                        .to_string();

                    let optimized_work_orders =
                        self.scheduler_agent_algorithm.get_optimized_work_orders();

                    let number_of_strategic_work_orders = optimized_work_orders.len();
                    let mut scheduled_count = 0;
                    for (work_order_number, optimized_work_order) in optimized_work_orders {
                        if optimized_work_order.get_scheduled_period().is_some() {
                            scheduled_count += 1;
                        }
                    }

                    let scheduling_status = format!(
                        "{}\nWith objectives: \n  strategic objective of: {}\n    {} of {} work orders scheduled",
                        scheduling_status, strategic_objective, scheduled_count, number_of_strategic_work_orders
                    );
                    match self.ws_addr.as_ref() {
                        Some(addr) => addr
                            .do_send(shared_messages::Response::Success(Some(scheduling_status))),
                        None => {
                            println!(
                                "No WebSocketAgentAddr set yet, so no message sent to frontend"
                            )
                        }
                    }

                    info!("SchedulerAgentReceived a Status message");
                }
                StrategicStatusMessage::Period(period) => {
                    let work_orders = self.scheduler_agent_algorithm.get_optimized_work_orders();

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
                    let mut message = String::new();

                    writeln!(
                        message,
                        "Work orders scheduled for period: {} are: ",
                        period,
                    )
                    .unwrap();

                    write!(message, "                      |AWCS|SECE|TYPE|PRIO|VEN*|",);

                    let work_orders: crate::models::WorkOrders = self
                        .scheduling_environment
                        .lock()
                        .unwrap()
                        .clone_work_orders();

                    println!("work_orders_by_period: {:?}", work_orders_by_period);

                    for work_order_number in work_orders_by_period {
                        writeln!(
                            message,
                            "    Work order: {}    |{:<5}|{:<5}|{:?}|{:?}|{:<5}|",
                            work_order_number,
                            work_orders
                                .inner
                                .get(&work_order_number)
                                .unwrap()
                                .get_status_codes()
                                .awsc,
                            work_orders
                                .inner
                                .get(&work_order_number)
                                .unwrap()
                                .get_status_codes()
                                .sece,
                            work_orders
                                .inner
                                .get(&work_order_number)
                                .unwrap()
                                .get_order_type(),
                            work_orders
                                .inner
                                .get(&work_order_number)
                                .unwrap()
                                .get_priority(),
                            work_orders
                                .inner
                                .get(&work_order_number)
                                .unwrap()
                                .is_vendor(),
                        );
                    }

                    match self.ws_addr.as_ref() {
                        Some(addr) => {
                            addr.do_send(shared_messages::Response::Success(Some(message)))
                        }
                        None => {
                            println!(
                                "No WebSocketAgentAddr set yet, so no message sent to frontend"
                            )
                        }
                    }
                }
            },
            StrategicRequest::Scheduling(scheduling_message) => {
                info!(
                    target: "SchedulerRequest::Input",
                    message = ?scheduling_message,
                    "received a message from the frontend"
                );

                let response: shared_messages::Response = self
                    .scheduler_agent_algorithm
                    .update_scheduling_state(scheduling_message);

                match self.ws_addr.as_ref() {
                    Some(addr) => addr.do_send(response),
                    None => {
                        println!("No WebSocketAgentAddr set yet, so no message sent to frontend")
                    }
                }
            }
            StrategicRequest::Resources(resources_message) => {
                let response: shared_messages::Response = self
                    .scheduler_agent_algorithm
                    .update_resources_state(resources_message);

                match self.ws_addr.as_ref() {
                    Some(addr) => addr.do_send(response),
                    None => {
                        println!("No WebSocketAgentAddr set yet, so no message sent to frontend")
                    }
                }
            }
            StrategicRequest::Periods(periods_message) => {
                info!(
                    "SchedulerAgentReceived a PeriodMessage message: {:?}",
                    periods_message
                );

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
                self.scheduler_agent_algorithm.set_periods(periods.to_vec());
            }
        }
    }
}

impl Handler<StatusRequest> for StrategicAgent {
    type Result = ();
    fn handle(&mut self, msg: StatusRequest, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            StatusRequest::GetWorkOrderStatus(work_order_number) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let cloned_work_orders = scheduling_environment_guard.clone_work_orders();

                if let Some(work_order_status) = cloned_work_orders.inner.get(&work_order_number) {
                    let work_order_status = work_order_status.to_string();
                    match self.ws_addr.as_ref() {
                        Some(addr) => addr
                            .do_send(shared_messages::Response::Success(Some(work_order_status))),
                        None => {
                            println!(
                                "No WebSocketAgentAddr set yet, so no message sent to frontend"
                            )
                        }
                    }
                }
            }
            StatusRequest::GetPeriods => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let periods = scheduling_environment_guard.clone_periods();

                let periods_string: String = periods
                    .iter()
                    .map(|period| period.get_period_string())
                    .collect::<Vec<String>>()
                    .join(",");

                match self.ws_addr.as_ref() {
                    Some(addr) => {
                        addr.do_send(shared_messages::Response::Success(Some(periods_string)))
                    }
                    None => {
                        println!("No WebSocketAgentAddr set yet, so no message sent to frontend")
                    }
                }
            }
        }
    }
}

impl Handler<MessageToFrontend> for StrategicAgent {
    type Result = ();

    #[instrument(
        fields(scheduler_message_type = "message_to_frontend"),
        skip(msg, self, _ctx)
    )]
    fn handle(&mut self, msg: MessageToFrontend, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            MessageToFrontend::Overview => {
                let scheduling_overview_data = self.extract_state_to_scheduler_overview().clone();
                let overview_message = OverviewMessage {
                    frontend_message_type: "frontend_scheduler_overview".to_string(),
                    scheduling_overview_data,
                };
                trace!(
                    scheduler_message = "scheduler overview message",
                    "scheduler_frontend_overview_message: {:?}",
                    overview_message
                );
                match self.ws_addr.as_ref() {
                    Some(ws_agent) => {
                        ws_agent.do_send(overview_message);
                    }
                    None => {
                        info!("No WebSocketAgentAddr set yet, so no message sent to frontend")
                    }
                }
            }
            MessageToFrontend::Loading => {
                let nested_loadings = strategic_agent::transform_hashmap_to_nested_hashmap(
                    self.scheduler_agent_algorithm
                        .get_resources_loadings()
                        .inner
                        .clone(),
                );

                let scheduler_frontend_loading_message = LoadingMessage {
                    frontend_message_type: "frontend_scheduler_loading".to_string(),
                    manual_resources_loading: nested_loadings,
                };
                trace!(
                    scheduler_message = "scheduler loading message",
                    "scheduler_frontend_loading_message: {:?}",
                    scheduler_frontend_loading_message
                );
                match self.ws_addr.as_ref() {
                    Some(ws_agent) => {
                        ws_agent.do_send(scheduler_frontend_loading_message);
                    }
                    None => {
                        info!("No WebSocketAgentAddr set yet, so no message sent to frontend")
                    }
                }
            }
            MessageToFrontend::Period => {
                self.scheduling_environment
                    .lock()
                    .unwrap()
                    .set_periods(self.scheduler_agent_algorithm.get_periods().clone());

                let scheduler_frontend_period_message = PeriodMessage {
                    frontend_message_type: "frontend_scheduler_periods".to_string(),
                    periods: self.scheduling_environment.lock().unwrap().clone_periods(),
                };
                trace!(
                    scheduler_message = "scheduler period message",
                    "scheduler_frontend_period_message: {:?}",
                    scheduler_frontend_period_message
                );
                match self.ws_addr.as_ref() {
                    Some(ws_agent) => {
                        ws_agent.do_send(scheduler_frontend_period_message);
                    }
                    None => {
                        info!("No WebSocketAgentAddr set yet, so no message sent to frontend")
                    }
                }
            }
        }
    }
}

impl Handler<SetAgentAddrMessage<WebSocketAgent>> for StrategicAgent {
    type Result = ();

    fn handle(
        &mut self,
        msg: SetAgentAddrMessage<WebSocketAgent>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.set_ws_agent_addr(msg.addr);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetAgentAddrMessage<T: actix::Actor> {
    pub addr: Addr<T>,
}

#[cfg(test)]
pub mod tests {
    use core::panic;
    use std::collections::HashSet;

    use super::*;

    use crate::agents::strategic_agent::strategic_algorithm::{
        OptimizedWorkOrder, OptimizedWorkOrders, PriorityQueues, StrategicAlgorithm,
    };

    use shared_messages::strategic::strategic_scheduling_message::{
        SingleWorkOrder, StrategicSchedulingMessage,
    };
    use tests::strategic_agent::strategic_algorithm::AlgorithmResources;

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

        scheduler_agent_algorithm.update_scheduling_state(strategic_scheduling_internal);

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

        scheduler_agent_algorithm.update_scheduling_state(strategic_scheduling_message);

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
                objective_value: self.scheduler_agent_algorithm.get_objective_value(),
                manual_resources_capacity: self
                    .scheduler_agent_algorithm
                    .get_resources_capacities()
                    .inner
                    .clone(),
                manual_resources_loading: self
                    .scheduler_agent_algorithm
                    .get_resources_loadings()
                    .inner
                    .clone(),
                priority_queues: self.scheduler_agent_algorithm.get_priority_queues().clone(),
                optimized_work_orders: OptimizedWorkOrders::new(
                    self.scheduler_agent_algorithm
                        .get_optimized_work_orders()
                        .clone(),
                ),
                periods: self.scheduler_agent_algorithm.get_periods().clone(),
            })
        }
    }

    #[test]
    fn test_handler_message_to_frontend() {}
}
