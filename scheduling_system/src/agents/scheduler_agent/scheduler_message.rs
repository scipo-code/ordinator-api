use actix::prelude::*;
use serde::{Deserialize, Serialize};
use shared_messages::strategic::strategic_status_message::StrategicStatusMessage;
use shared_messages::strategic::StrategicRequests;
use std::collections::HashMap;
use std::fmt::{self, Display};
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, instrument, trace};

use crate::agents::scheduler_agent::scheduler_algorithm::QueueType;
use crate::agents::scheduler_agent::{self, SchedulerAgent};
use crate::api::websocket_agent::WebSocketAgent;
use crate::models::time_environment::period::Period;
use shared_messages::resources::Resources;

use shared_messages::strategic::strategic_periods_message::StrategicPeriodsMessage;
use shared_messages::strategic::strategic_resources_message::StrategicResourcesMessage;
use shared_messages::strategic::strategic_scheduling_message::{
    ScheduleSingleWorkOrder, StrategicSchedulingMessage, WorkOrderPeriodMapping,
};

#[derive(Debug)]
pub struct StrategicSchedulingInternal {
    schedule_work_orders: Vec<ScheduleSingleWorkOrder>,
}

pub struct StrategicResourcesInternal {
    manual_resources: HashMap<(Resources, String), f64>,
}

pub struct StrategicPeriodsInternal {
    period_lock: HashMap<String, bool>,
}

impl StrategicSchedulingInternal {
    pub fn get_schedule_work_orders(&self) -> &Vec<ScheduleSingleWorkOrder> {
        &self.schedule_work_orders
    }
}

impl StrategicResourcesInternal {
    pub fn get_manual_resources(&self) -> HashMap<(Resources, String), f64> {
        self.manual_resources.clone()
    }
}

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

impl From<StrategicSchedulingMessage> for StrategicSchedulingInternal {
    fn from(message: StrategicSchedulingMessage) -> Self {
        match message {
            StrategicSchedulingMessage::Schedule(message) => StrategicSchedulingInternal {
                schedule_work_orders: vec![message],
            },
        }
    }
}

impl From<StrategicResourcesMessage> for StrategicResourcesInternal {
    fn from(message: StrategicResourcesMessage) -> Self {
        let mut manual_resources_map: HashMap<(Resources, String), f64> = HashMap::new();
        for res in message.get_manual_resources() {
            manual_resources_map.insert((res.resource, res.period.period_string), res.capacity);
        }
        StrategicResourcesInternal {
            manual_resources: manual_resources_map,
        }
    }
}

impl From<StrategicPeriodsMessage> for StrategicPeriodsInternal {
    fn from(message: StrategicPeriodsMessage) -> Self {
        StrategicPeriodsInternal {
            period_lock: message.period_lock,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct UpdatePeriod {
    pub periods: Vec<Period>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ScheduleIteration {}

impl Handler<ScheduleIteration> for SchedulerAgent {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut Self::Context) -> Self::Result {
        // event!(tracing::Level::INFO , "schedule_iteration_message");
        let rng: &mut rand::rngs::ThreadRng = &mut rand::thread_rng();
        self.scheduler_agent_algorithm
            .unschedule_random_work_orders(5, rng);

        self.scheduler_agent_algorithm
            .schedule_normal_work_orders(QueueType::Normal);

        self.scheduler_agent_algorithm.schedule_forced_work_orders();
        // self.scheduler_agent_algorithm.schedule_work_orders_by_type(QueueType::UnloadingAndManual);

        self.scheduler_agent_algorithm.calculate_objective();

        debug!(
            "Objective value: {}",
            self.scheduler_agent_algorithm.get_objective_value()
        );

        let actor_addr = ctx.address().clone();

        let fut = async move {
            sleep(Duration::from_secs(1)).await;
            actor_addr.do_send(ScheduleIteration {});
        };

        if self.scheduler_agent_algorithm.changed() {
            trace!(
                agent = "scheduler_agent",
                name = self.platform.clone(),
                message = "change occured in optimized work orders"
            );

            // ctx.notify(MessageToFrontend::Overview);
            // ctx.notify(MessageToFrontend::Loading);

            self.scheduler_agent_algorithm.set_changed(false);
        }

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
    pub manual_resources_loading: HashMap<String, HashMap<String, f64>>,
}

impl Message for LoadingMessage {
    type Result = ();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OverviewMessage {
    pub frontend_message_type: String,
    pub scheduling_overview_data: Vec<scheduler_agent::SchedulingOverviewData>,
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

impl Handler<StrategicRequests> for SchedulerAgent {
    type Result = ();

    fn handle(&mut self, msg: StrategicRequests, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            StrategicRequests::Status(strategic_status_message) => match strategic_status_message {
                StrategicStatusMessage::General => {
                    let scheduling_status = self.scheduling_environment.lock().unwrap().to_string();
                    let strategic_objective = self
                        .scheduler_agent_algorithm
                        .get_objective_value()
                        .to_string();
                    let scheduling_status = format!(
                        "{}\nWith objectives: \n  strategic objective of: {}",
                        scheduling_status, strategic_objective
                    );
                    dbg!();
                    match self.ws_agent_addr.as_ref() {
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

                    dbg!(period.clone());
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

                    dbg!(work_orders_by_period.clone());
                    let message = format!(
                        "Work orders scheduled for period: {} are: {:?}",
                        period, work_orders_by_period
                    );

                    match self.ws_agent_addr.as_ref() {
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
            StrategicRequests::Scheduling(scheduling_message) => {
                let strategic_scheduling_internal: StrategicSchedulingInternal =
                    scheduling_message.into();
                info!(
                    target: "SchedulerRequest::Input",
                    message = ?strategic_scheduling_internal,
                    "received a message from the frontend"
                );
                self.scheduler_agent_algorithm
                    .update_scheduling_state(strategic_scheduling_internal);

                match self.ws_agent_addr.as_ref() {
                    Some(addr) => addr.do_send(shared_messages::Response::Success(None)),
                    None => {
                        println!("No WebSocketAgentAddr set yet, so no message sent to frontend")
                    }
                }
            }
            StrategicRequests::Resources(resources_message) => {
                let manual_resources = self
                    .scheduler_agent_algorithm
                    .get_manual_resources_loadings()
                    .clone();
                let manual_resources = serde_json::to_string(&manual_resources).unwrap();

                match self.ws_agent_addr.as_ref() {
                    Some(addr) => {
                        addr.do_send(shared_messages::Response::Success(Some(manual_resources)))
                    }
                    None => {
                        println!("No WebSocketAgentAddr set yet, so no message sent to frontend")
                    }
                }
            }
            StrategicRequests::Periods(periods_message) => {
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

impl Handler<MessageToFrontend> for SchedulerAgent {
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
                match self.ws_agent_addr.as_ref() {
                    Some(ws_agent) => {
                        ws_agent.do_send(overview_message);
                    }
                    None => {
                        info!("No WebSocketAgentAddr set yet, so no message sent to frontend")
                    }
                }
            }
            MessageToFrontend::Loading => {
                let nested_loadings = scheduler_agent::transform_hashmap_to_nested_hashmap(
                    self.scheduler_agent_algorithm
                        .get_manual_resources_loadings()
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
                match self.ws_agent_addr.as_ref() {
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
                match self.ws_agent_addr.as_ref() {
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

impl Handler<SetAgentAddrMessage<WebSocketAgent>> for SchedulerAgent {
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
    use std::collections::HashSet;

    use super::*;
    use chrono::{TimeZone, Utc};

    use crate::models::work_order::WorkOrder;
    use crate::{
        agents::scheduler_agent::scheduler_algorithm::{
            OptimizedWorkOrder, OptimizedWorkOrders, PriorityQueues, SchedulerAgentAlgorithm,
        },
        models::{
            work_order::{
                functional_location::FunctionalLocation,
                order_dates::OrderDates,
                order_text::OrderText,
                order_type::{WDFPriority, WorkOrderType},
                priority::Priority,
                revision::Revision,
                status_codes::StatusCodes,
                unloading_point::UnloadingPoint,
            },
            WorkOrders,
        },
    };

    use crate::agents::scheduler_agent::scheduler_message::WorkOrderPeriodMapping;
    use shared_messages::strategic::strategic_scheduling_message::{
        ScheduleSingleWorkOrder, WorkOrderStatusInPeriod,
    };
    use shared_messages::strategic::TimePeriod;

    #[test]
    fn test_update_scheduler_state() {
        let period_string: String = "2023-W47-48".to_string();

        let schedule_work_orders = vec![ScheduleSingleWorkOrder {
            work_order_number: 2200002020,
            period_string,
        }];

        let mut work_orders = WorkOrders::new();

        let work_order = WorkOrder::new(
            2200002020,
            false,
            1000,
            Priority::new_int(1),
            100.0,
            HashMap::new(),
            HashMap::new(),
            vec![],
            vec![],
            vec![],
            WorkOrderType::Wdf(WDFPriority::new(1)),
            crate::models::work_order::system_condition::SystemCondition::Unknown,
            StatusCodes::new_default(),
            OrderDates::new_test(),
            Revision::new_default(),
            UnloadingPoint::new_default(),
            FunctionalLocation::new_default(),
            OrderText::new_default(),
            false,
        );

        work_orders.insert(work_order);

        let strategic_scheduling_internal = StrategicSchedulingInternal {
            schedule_work_orders,
        };

        let periods: Vec<Period> = vec![Period::new_from_string("2023-W47-48").unwrap()];

        let mut scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            HashMap::new(),
            HashMap::new(),
            work_orders,
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            periods,
            true,
        );

        scheduler_agent_algorithm.update_scheduling_state(strategic_scheduling_internal);

        assert_eq!(
            scheduler_agent_algorithm
                .get_optimized_work_orders()
                .get(&2200002020)
                .as_ref()
                .unwrap()
                .locked_in_period
                .as_ref()
                .unwrap()
                .get_period_string(),
            "2023-W47-48"
        );
    }

    #[test]
    fn test_input_scheduler_message_from() {
        let work_order_period_mapping = WorkOrderPeriodMapping {
            work_order_number: 2100023841,
            period_status: WorkOrderStatusInPeriod {
                locked_in_period: Some(TimePeriod::new("2023-W49-50".to_string())),
                excluded_from_periods: HashSet::new(),
            },
        };

        let schedule_single_work_order = ScheduleSingleWorkOrder {
            work_order_number: 2100023841,
            period_string: "2023-W49-50".to_string(),
        };

        let strategic_scheduling_message =
            StrategicSchedulingMessage::Schedule(schedule_single_work_order);

        let strategic_scheduling_internal: StrategicSchedulingInternal =
            strategic_scheduling_message.into();

        assert_eq!(
            strategic_scheduling_internal.schedule_work_orders[0].work_order_number,
            2100023841
        );
        assert_eq!(
            strategic_scheduling_internal.schedule_work_orders[0]
                .period_status
                .locked_in_period,
            Some(TimePeriod::new("2023-W49-50".to_string()))
        );

        let mut work_load = HashMap::new();

        work_load.insert(Resources::VenMech, 16.0);

        let work_order = WorkOrder::new(
            2100023841,
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
            crate::models::work_order::system_condition::SystemCondition::Unknown,
            StatusCodes::new_default(),
            OrderDates::new_test(),
            Revision::new_default(),
            UnloadingPoint::new_default(),
            FunctionalLocation::new_default(),
            OrderText::new_default(),
            false,
        );

        let mut work_orders = WorkOrders::new();

        work_orders.inner.insert(2100023841, work_order);

        let periods: Vec<Period> = vec![Period::new_from_string("2023-W49-50").unwrap()];

        let mut scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            HashMap::new(),
            HashMap::new(),
            work_orders,
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            periods,
            true,
        );

        scheduler_agent_algorithm.update_scheduling_state(strategic_scheduling_internal);

        assert_eq!(
            scheduler_agent_algorithm
                .get_optimized_work_order(&2100023841)
                .unwrap()
                .locked_in_period,
            Some(Period::new_from_string("2023-W49-50").unwrap())
        );
        assert_eq!(
            scheduler_agent_algorithm
                .get_optimized_work_order(&2100023841)
                .unwrap()
                .scheduled_period,
            None
        );
        // assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("VEN_MECH".to_string(), "2023-W49-50".to_string()), 16.0);
    }

    #[test]
    fn test_calculate_objective_value() {
        let mut work_order = WorkOrder::new(
            2100023841,
            false,
            1000,
            Priority::new_int(1),
            100.0,
            HashMap::new(),
            HashMap::new(),
            vec![],
            vec![],
            vec![],
            WorkOrderType::Wdf(WDFPriority::new(1)),
            crate::models::work_order::system_condition::SystemCondition::Unknown,
            StatusCodes::new_default(),
            OrderDates::new_test(),
            Revision::new_default(),
            UnloadingPoint::new_default(),
            FunctionalLocation::new_default(),
            OrderText::new_default(),
            false,
        );

        work_order
            .get_mut_order_dates()
            .latest_allowed_finish_period = Period::new_from_string("2023-W47-48").unwrap();

        let mut work_orders = WorkOrders::new();

        work_orders.inner.insert(2100023841, work_order);

        let mut optimized_work_orders = OptimizedWorkOrders::new(HashMap::new());

        let optimized_work_order = OptimizedWorkOrder::new(
            Some(Period::new_from_string("2023-W49-50").unwrap()),
            Some(Period::new_from_string("2023-W49-50").unwrap()),
            HashSet::new(),
        );

        optimized_work_orders.insert_optimized_work_order(2100023841, optimized_work_order);

        let mut scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            HashMap::new(),
            HashMap::new(),
            work_orders,
            PriorityQueues::new(),
            optimized_work_orders,
            vec![],
            true,
        );

        scheduler_agent_algorithm.calculate_objective();
        assert_eq!(scheduler_agent_algorithm.get_objective_value(), 2000.0);
    }

    impl StrategicSchedulingInternal {
        pub fn new_test() -> Self {
            // I am having so much fun with this

            let work_order_period_mapping = WorkOrderPeriodMapping::new_test();

            Self {
                work_order_period_mappings: vec![work_order_period_mapping],
            }
        }
    }

    impl StrategicResourcesInternal {
        pub fn new_test() -> Self {
            let mut manual_resources = HashMap::new();

            let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
            let end_date = start_date + chrono::Duration::days(13);
            let period = Period::new(1, start_date, end_date);

            manual_resources.insert((Resources::MtnMech, period.get_period_string()), 300.0);
            manual_resources.insert((Resources::MtnElec, period.get_period_string()), 300.0);
            manual_resources.insert((Resources::Prodtech, period.get_period_string()), 300.0);

            Self { manual_resources }
        }
    }

    pub struct TestRequest {}

    impl Message for TestRequest {
        type Result = Option<TestResponse>;
    }

    pub struct TestResponse {
        pub objective_value: f64,
        pub manual_resources_capacity: HashMap<(Resources, Period), f64>,
        pub manual_resources_loading: HashMap<(Resources, Period), f64>,
        pub priority_queues: PriorityQueues<u32, u32>,
        pub optimized_work_orders: OptimizedWorkOrders,
        pub periods: Vec<Period>,
    }

    impl Handler<TestRequest> for SchedulerAgent {
        type Result = Option<TestResponse>; // Or relevant part of the state

        fn handle(&mut self, _msg: TestRequest, _: &mut Context<Self>) -> Self::Result {
            // Return the state or part of it
            Some(TestResponse {
                objective_value: self.scheduler_agent_algorithm.get_objective_value(),
                manual_resources_capacity: self
                    .scheduler_agent_algorithm
                    .get_manual_resources_capacities()
                    .clone(),
                manual_resources_loading: self
                    .scheduler_agent_algorithm
                    .get_manual_resources_loadings()
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
