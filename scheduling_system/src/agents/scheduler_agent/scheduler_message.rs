use actix::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, span, trace};

use crate::agents::scheduler_agent::scheduler_algorithm::QueueType;
use crate::agents::scheduler_agent::{self, SchedulerAgent};
use crate::api::websocket_agent::WebSocketAgent;
use crate::models::time_environment::period::Period;
use crate::models::worker_environment::resources::Resources;

#[derive(Deserialize, Debug)]
#[serde(tag = "scheduler_message_type")]
pub enum SchedulerRequests {
    Input(FrontendInputSchedulerMessage),
    Period(FrontendUpdatePeriod),
    WorkPlanner(WorkPlannerMessage),
}

#[derive(Debug)]
pub struct InputSchedulerMessage {
    name: String,
    platform: String,
    work_order_period_mappings: Vec<WorkOrderPeriodMapping>, // For each work order only one of these can be true
    manual_resources: HashMap<(Resources, String), f64>,
    period_lock: HashMap<String, bool>,
}

impl InputSchedulerMessage {
    pub fn get_manual_resources(&self) -> HashMap<(Resources, String), f64> {
        self.manual_resources.clone()
    }

    pub fn get_work_order_period_mappings(&self) -> &Vec<WorkOrderPeriodMapping> {
        &self.work_order_period_mappings
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetAgentAddrMessage<T: actix::Actor> {
    pub addr: Addr<T>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct WorkPlannerMessage {
    pub cannot_schedule: Vec<u32>,
    under_loaded_work_centers: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct FrontendInputSchedulerMessage {
    pub name: String,
    pub platform: String,
    pub work_order_period_mappings: Vec<WorkOrderPeriodMapping>,
    pub manual_resources: Vec<ManualResource>,
    pub period_lock: HashMap<String, bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ManualResource {
    pub resource: Resources,
    pub period: TimePeriod,
    pub capacity: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TimePeriod {
    pub period_string: String,
}

impl TimePeriod {
    pub fn get_period_string(&self) -> String {
        self.period_string.clone()
    }
}

/// This is a message that is sent from the scheduler frontend to the scheduler agent requesting an
/// update of the periods that we schedule over. What is it actually that we are getting from the
/// frontend here? We will only get a single period from the frontend. Or maybe multiple. The
/// important thing to remember is that we cannot simply set the periods in the scheduler agent
/// we have to update them. This is crucial. Also, should we use a hashmap or a vector for the
/// periods? I think that a vector is the better approach for now. A hashmap would make sense if we
/// were to have overlapping periods or non-continuous periods. Which is not the case for now. This
/// means that we will skip that part.
#[derive(Deserialize, Debug)]
pub struct UpdatePeriod {
    pub periods: Vec<Period>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkOrderPeriodMapping {
    pub work_order_number: u32,
    pub period_status: WorkOrderStatusInPeriod,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkOrderStatusInPeriod {
    pub locked_in_period: Option<TimePeriod>,
    #[serde(deserialize_with = "deserialize_period_set")]
    pub excluded_from_periods: HashSet<String>,
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

impl Message for SchedulerRequests {
    type Result = ();
}

impl Display for InputSchedulerMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let manual_resources_pretty = SchedulerResources(&self.manual_resources);
        write!(
            f,
            "Name: {}, 
            \nPlatform: {}, 
            \nSchedule Work Order: {:?}, 
            \nManual Resource: {},
            \nPeriod Lock: {:?}",
            self.name,
            self.platform,
            self.work_order_period_mappings,
            manual_resources_pretty,
            self.period_lock
        )
    }
}

// The problem with using the period ID is the it is not static and changes every period. There are
// multiple approaches to be chosen here. One is to use the period ID and the other is to use the
// I think that using the period ID is the best approch but it will require some changes to the
// structure of the scheduler agent as we will have to force additional structure on the application
// Hmm... I feel like this is one of the moments where I know that it is the right thing to do but
// it also feels like a distraction from the main task at hand. This is a very tricky situation but
// handling it correctly is crucial for the future of the application. I think that I will go with
// Hmm... Should I just send all the periods to the backend again? I think that I will do that. I
#[derive(Deserialize, Debug)]
#[serde(tag = "scheduler_message_type")]
pub struct FrontendUpdatePeriod {
    periods: Vec<u32>,
}

/// There is something fundamentally wrong here. We should not be using strings as keys and we
/// should not be hashing Period or Resources. In the algorithm we should be using an array or
/// arrays, but that is not the goal for this part of the program, here understanding the logic is
/// more important. That means we should be using Resource and Period as keys
///
/// I think that I should just
impl From<FrontendInputSchedulerMessage> for InputSchedulerMessage {
    fn from(raw: FrontendInputSchedulerMessage) -> Self {
        let mut manual_resources_map: HashMap<(Resources, String), f64> = HashMap::new();
        for res in raw.manual_resources {
            manual_resources_map.insert((res.resource, res.period.period_string), res.capacity);
        }
        println!("{:?}", manual_resources_map);
        InputSchedulerMessage {
            name: raw.name,
            platform: raw.platform,
            work_order_period_mappings: raw.work_order_period_mappings,
            manual_resources: manual_resources_map,
            period_lock: raw.period_lock,
        }
    }
}

impl Display for FrontendInputSchedulerMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Name: {}, 
            \nPlatform: {}, 
            \nWorkorder period mappings: {}, 
            \nManual Resource: {},
            \nPeriod Lock: {:?}",
            self.name,
            self.platform,
            self.work_order_period_mappings.len(),
            self.manual_resources.len(),
            self.period_lock
        )
    }
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

            ctx.notify(MessageToFrontend::Overview);
            ctx.notify(MessageToFrontend::Loading);

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

impl Handler<MessageToFrontend> for SchedulerAgent {
    type Result = ();

    fn handle(&mut self, msg: MessageToFrontend, _ctx: &mut Self::Context) -> Self::Result {
        let span = span!(
            tracing::Level::TRACE,
            "preparing scheduler message for the frontend",
            self.platform
        );
        let _enter = span.enter();

        match msg {
            MessageToFrontend::Overview => {
                let scheduling_overview_data = self.extract_state_to_scheduler_overview().clone();
                let scheduler_frontend_overview_message = OverviewMessage {
                    frontend_message_type: "frontend_scheduler_overview".to_string(),
                    scheduling_overview_data,
                };
                trace!(
                    scheduler_message = "scheduler overview message",
                    "scheduler_frontend_overview_message: {:?}",
                    scheduler_frontend_overview_message
                );
                match self.ws_agent_addr.as_ref() {
                    Some(ws_agent) => {
                        ws_agent.do_send(scheduler_frontend_overview_message);
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

impl Handler<SchedulerRequests> for SchedulerAgent {
    type Result = ();

    fn handle(&mut self, msg: SchedulerRequests, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SchedulerRequests::Input(msg) => {
                let input_message: InputSchedulerMessage = msg.into();
                info!(
                    target: "SchedulerRequest::Input",
                    message = %input_message,
                    "received a message from the frontend"
                );
                self.scheduler_agent_algorithm.log_optimized_work_orders();
                self.scheduler_agent_algorithm
                    .update_scheduler_algorithm_state(input_message);
                self.scheduler_agent_algorithm.log_optimized_work_orders();

                if self
                    .scheduler_agent_algorithm
                    .get_optimized_work_order(&2100023393)
                    .is_some()
                {
                    info!("2100023393 is in the optimized work orders");
                } else {
                    info!("2100023393 is not in the optimized work orders");
                }
            }
            SchedulerRequests::WorkPlanner(msg) => {
                info!(
                    "SchedulerAgentReceived a WorkPlannerMessage message: {:?}",
                    msg
                );
            }
            // We should handle the logic of the period update here. Here we have the information
            // available to update the state based on the period update. We should also send the
            // Okay what is happening here? Let us go through an example where two messages hit the
            // scheduler agent at the same time. Both messages clone the same state. No the problem
            // comes when you clone the state in the first message. Okay the error is that multiple
            // lock are are lying in queue with the same state. So the first message is handled and
            // is incrementing the state and then when the second message is handled it is, it is
            // handled using the old state again, so of course it is not going to work. And have to
            // be handled in a different way.
            SchedulerRequests::Period(msg) => {
                info!("SchedulerAgentReceived a PeriodMessage message: {:?}", msg);

                let mut scheduling_env_lock = self.scheduling_environment.lock().unwrap();

                let periods = scheduling_env_lock.get_mut_periods();

                for period_id in msg.periods.iter() {
                    if periods.last().unwrap().get_id() + 1 == *period_id {
                        let new_period =
                            periods.last().unwrap().clone() + chrono::Duration::weeks(2);
                        periods.push(new_period);
                    } else {
                        error!("periods not handled correctly");
                    }
                }
                self.scheduler_agent_algorithm.set_periods(periods.to_vec());
                ctx.notify(MessageToFrontend::Period);
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

fn deserialize_period_set<'de, D>(deserializer: D) -> Result<HashSet<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec = Vec::<TimePeriod>::deserialize(deserializer)?;
    let mut set = HashSet::new();
    for time_period_map in vec {
        set.insert(time_period_map.period_string);
    }
    Ok(set)
}

#[cfg(test)]
pub mod tests {
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

    #[test]
    fn test_update_scheduler_state() {
        let period_string: String = "2023-W47-48".to_string();

        let work_order_period_mappings = vec![WorkOrderPeriodMapping {
            work_order_number: 2200002020,
            period_status: WorkOrderStatusInPeriod {
                locked_in_period: Some(TimePeriod::new(period_string)),
                excluded_from_periods: HashSet::new(),
            },
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

        let input_message = InputSchedulerMessage {
            name: "test".to_string(),
            platform: "test".to_string(),
            work_order_period_mappings,
            manual_resources: HashMap::new(),
            period_lock: HashMap::new(),
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

        scheduler_agent_algorithm.update_scheduler_algorithm_state(input_message);

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
        // let input_scheduler_message: = work_order_period_mapping.into();

        let frontend_input_scheduler_message = FrontendInputSchedulerMessage {
            name: "test".to_string(),
            platform: "test".to_string(),
            work_order_period_mappings: vec![work_order_period_mapping],
            manual_resources: vec![],
            period_lock: HashMap::new(),
        };

        let input_scheduler_message: InputSchedulerMessage =
            frontend_input_scheduler_message.into();

        assert_eq!(
            input_scheduler_message.work_order_period_mappings[0].work_order_number,
            2100023841
        );
        assert_eq!(
            input_scheduler_message.work_order_period_mappings[0]
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

        scheduler_agent_algorithm.update_scheduler_algorithm_state(input_scheduler_message);

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

    impl InputSchedulerMessage {
        pub fn new_test() -> Self {
            // I am having so much fun with this

            let work_order_period_mapping = WorkOrderPeriodMapping::new_test();

            let mut manual_resources = HashMap::new();

            let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
            let end_date = start_date + chrono::Duration::days(13);
            let period = Period::new(1, start_date, end_date);

            manual_resources.insert((Resources::MtnMech, period.get_period_string()), 300.0);
            manual_resources.insert((Resources::MtnElec, period.get_period_string()), 300.0);
            manual_resources.insert((Resources::Prodtech, period.get_period_string()), 300.0);

            Self {
                name: "test".to_string(),
                platform: "test".to_string(),
                work_order_period_mappings: vec![work_order_period_mapping],
                manual_resources,
                period_lock: HashMap::new(),
            }
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

    impl WorkOrderPeriodMapping {
        pub fn new_test() -> Self {
            WorkOrderPeriodMapping {
                work_order_number: 2200002020,
                period_status: WorkOrderStatusInPeriod::new_test(),
            }
        }
    }

    impl WorkOrderStatusInPeriod {
        pub fn new_test() -> Self {
            let period_string = "2023-W47-48".to_string();
            WorkOrderStatusInPeriod {
                locked_in_period: Some(TimePeriod::new(period_string)),
                excluded_from_periods: HashSet::new(),
            }
        }
    }

    impl TimePeriod {
        pub fn new(period_string: String) -> Self {
            Self { period_string }
        }
    }
}
