use actix::prelude::*;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};
use tokio::time::{sleep, Duration};
use tracing::{debug, info, span, trace};

use crate::agents::scheduler_agent::scheduler_algorithm::QueueType;
use crate::agents::scheduler_agent::{self, SchedulerAgent};
use crate::api::websocket_agent::SchedulerFrontendLoadingMessage;
use crate::api::websocket_agent::SchedulerFrontendMessage;
use crate::api::websocket_agent::WebSocketAgent;
use crate::models::time_environment::period::Period;

#[derive(Serialize, Deserialize)]
#[serde(tag = "scheduler_message_type")]
#[derive(Debug)]
pub enum SchedulerRequests {
    Input(FrontendInputSchedulerMessage),
    WorkPlanner(WorkPlannerMessage),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InputSchedulerMessage {
    pub name: String,
    pub platform: String,
    pub work_order_period_mappings: Vec<WorkOrderPeriodMapping>, // For each work order only one of these can be true
    pub manual_resources: HashMap<(String, String), f64>,
    pub period_lock: HashMap<String, bool>,
}

impl InputSchedulerMessage {
    pub fn get_manual_resources(&self) -> HashMap<(String, String), f64> {
        self.manual_resources.clone()
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ManualResource {
    pub resource: String,
    pub period: TimePeriod,
    pub capacity: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimePeriod {
    pub period_string: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FrontendInputSchedulerMessage {
    pub name: String,
    pub platform: String,
    pub work_order_period_mappings: Vec<WorkOrderPeriodMapping>,
    pub manual_resources: Vec<ManualResource>,
    pub period_lock: HashMap<String, bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkOrderPeriodMapping {
    pub work_order_number: u32,
    pub period_status: WorkOrderStatusInPeriod,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkOrderStatusInPeriod {
    #[serde(deserialize_with = "deserialize_period_option")]
    pub locked_in_period: Option<Period>,
    #[serde(deserialize_with = "deserialize_period_set")]
    pub excluded_from_periods: HashSet<Period>,
}

struct SchedulerResources<'a>(&'a HashMap<(String, String), f64>);

impl Display for SchedulerResources<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "--------------------------")?;
        for ((resource, period), capacity) in self.0 {
            writeln!(
                f,
                "Resource: {}\nPeriod: {}\nCapacity: {}",
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

impl From<FrontendInputSchedulerMessage> for InputSchedulerMessage {
    fn from(raw: FrontendInputSchedulerMessage) -> Self {
        let mut manual_resources_map: HashMap<(String, String), f64> = HashMap::new();
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
            trace!(message = "change occured in optimized work orders");
            ctx.notify(MessageToFrontend {});
            self.scheduler_agent_algorithm.set_changed(false);
        }

        Box::pin(actix::fut::wrap_future::<_, Self>(fut))
    }
}

struct MessageToFrontend {}

impl Message for MessageToFrontend {
    type Result = ();
}

impl Handler<MessageToFrontend> for SchedulerAgent {
    type Result = ();

    fn handle(&mut self, _msg: MessageToFrontend, _ctx: &mut Self::Context) -> Self::Result {
        let span = span!(
            tracing::Level::TRACE,
            "preparing scheduler message for the frontend",
            self.platform
        );
        let _enter = span.enter();
        let scheduling_overview_data = self.extract_state_to_scheduler_overview().clone();

        dbg!(scheduling_overview_data.clone().len());

        let scheduler_frontend_message = SchedulerFrontendMessage {
            frontend_message_type: "frontend_scheduler_overview".to_string(),
            scheduling_overview_data,
        };

        trace!(
            "scheduler_frontend_message: {:?}",
            scheduler_frontend_message
        );

        let nested_loadings = scheduler_agent::transform_hashmap_to_nested_hashmap(
            self.scheduler_agent_algorithm
                .get_manual_resources_loadings()
                .clone(),
        );

        let scheduler_frontend_loading_message = SchedulerFrontendLoadingMessage {
            frontend_message_type: "frontend_scheduler_loading".to_string(),
            manual_resources_loading: nested_loadings,
        };

        match self.ws_agent_addr.as_ref() {
            Some(ws_agent) => {
                ws_agent.do_send(scheduler_frontend_message);
                ws_agent.do_send(scheduler_frontend_loading_message);
            }
            None => {
                info!("No WebSocketAgentAddr set yet, so no message sent to frontend")
            }
        }
    }
}

impl Handler<SchedulerRequests> for SchedulerAgent {
    type Result = ();

    fn handle(&mut self, msg: SchedulerRequests, _ctx: &mut Self::Context) -> Self::Result {
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

fn deserialize_period_option<'de, D>(deserializer: D) -> Result<Option<Period>, D::Error>
where
    D: Deserializer<'de>,
{
    let option = Option::<TimePeriod>::deserialize(deserializer)?;
    match option {
        Some(time_period_map) => Period::new_from_string(&time_period_map.period_string)
            .map(Some)
            .map_err(Error::custom),
        None => Ok(None),
    }
}

fn deserialize_period_set<'de, D>(deserializer: D) -> Result<HashSet<Period>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec = Vec::<TimePeriod>::deserialize(deserializer)?;
    let mut set = HashSet::new();
    for time_period_map in vec {
        let period =
            Period::new_from_string(&time_period_map.period_string).map_err(Error::custom)?;
        set.insert(period);
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
        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
        let end_date = start_date + chrono::Duration::days(13);

        let work_order_period_mappings = vec![WorkOrderPeriodMapping {
            work_order_number: 2200002020,
            period_status: WorkOrderStatusInPeriod {
                locked_in_period: Some(Period::new(1, start_date, end_date)),
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

        let mut scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            HashMap::new(),
            HashMap::new(),
            work_orders,
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            vec![],
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
                .get_string(),
            "2023-W47-48"
        );
    }

    #[test]
    fn test_input_scheduler_message_from() {
        let work_order_period_mapping = WorkOrderPeriodMapping {
            work_order_number: 2100023841,
            period_status: WorkOrderStatusInPeriod {
                locked_in_period: Some(Period::new_from_string("2023-W49-50").unwrap()),
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
            Some(Period::new_from_string("2023-W49-50").unwrap())
        );

        let mut work_load = HashMap::new();

        work_load.insert("VEN_MECH".to_string(), 16.0);

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

        let mut scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            HashMap::new(),
            HashMap::new(),
            work_orders,
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            vec![],
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

        work_order.order_dates.latest_allowed_finish_period =
            Period::new_from_string("2023-W47-48").unwrap();

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

            manual_resources.insert(
                ("MTN_MECH".to_string(), period.period_string.clone()),
                300.0,
            );
            manual_resources.insert(
                ("MTN_ELEC".to_string(), period.period_string.clone()),
                300.0,
            );
            manual_resources.insert(
                ("PRODTECH".to_string(), period.period_string.clone()),
                300.0,
            );

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
        pub manual_resources_capacity: HashMap<(String, String), f64>,
        pub manual_resources_loading: HashMap<(String, String), f64>,
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
            let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
            let end_date = start_date + chrono::Duration::days(13);
            let period = Period::new(1, start_date, end_date);

            WorkOrderStatusInPeriod {
                locked_in_period: Some(period),
                excluded_from_periods: HashSet::new(),
            }
        }
    }
}
