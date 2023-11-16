pub mod scheduler_message;
pub mod scheduler_algorithm;
pub mod display;

use std::collections::HashMap;
use std::collections::HashSet;
use actix::prelude::*; 
use actix::Message;
use priority_queue::PriorityQueue;
use std::hash::Hash;
use tracing::{event};
use tokio::time::{sleep, Duration};

use crate::models::work_order::priority::Priority;
use crate::models::work_order::order_type::WorkOrderType;
use crate::agents::scheduler_agent::scheduler_message::{SetAgentAddrMessage, SchedulerRequests, InputSchedulerMessage};
use crate::models::scheduling_environment::WorkOrders;
use crate::models::period::Period;
use crate::api::websocket_agent::WebSocketAgent;
use crate::agents::scheduler_agent::scheduler_algorithm::QueueType;
use crate::models::work_order::status_codes::MaterialStatus;
use crate::api::websocket_agent::SchedulerFrontendMessage;
use crate::api::websocket_agent::SchedulerFrontendLoadingMessage;

pub struct SchedulerAgent {
    platform: String,
    scheduler_agent_algorithm: SchedulerAgentAlgorithm,
    ws_agent_addr: Option<Addr<WebSocketAgent>>,
}

pub struct SchedulerAgentAlgorithm {
    manual_resources_capacity : HashMap<(String, String), f64>,
    manual_resources_loading: HashMap<(String, String), f64>,
    backlog: WorkOrders,
    priority_queues: PriorityQueues<u32, u32>,
    optimized_work_orders: OptimizedWorkOrders,
    periods: Vec<Period>,
}

pub struct OptimizedWorkOrder {
    scheduled_period: Option<Period>,
    locked_in_period: Option<Period>,
    excluded_from_periods: HashSet<Period>,
}

pub struct OptimizedWorkOrders {
    inner: HashMap<u32, OptimizedWorkOrder>,
}

impl OptimizedWorkOrders {
    pub fn new(inner: HashMap<u32, OptimizedWorkOrder>) -> Self {
        Self {
            inner: inner,
        }
    }
}

impl SchedulerAgent {
    pub fn set_ws_agent_addr(&mut self, ws_agent_addr: Addr<WebSocketAgent>) {
        self.ws_agent_addr = Some(ws_agent_addr);
    }

    // TODO: Here the other Agents Addr messages will also be handled.
}

impl Actor for SchedulerAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        self.populate_priority_queues();
        ctx.notify(ScheduleIteration {})
    }

    fn stopped(&mut self, ctx: &mut Context<Self>) {
        println!("SchedulerAgent is stopped");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct ScheduleIteration {}

impl Handler<ScheduleIteration> for SchedulerAgent {

    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: ScheduleIteration, ctx: &mut Self::Context) -> Self::Result {
        event!(tracing::Level::INFO , "A round of scheduling has been triggered");
        self.schedule_work_orders_by_type(QueueType::Normal);
        self.schedule_work_orders_by_type(QueueType::UnloadingAndManual);

        // let display_manual_resources = display::DisplayableManualResource(self.scheduler_agent_algorithm.manual_resources_capacity.clone());
        // let display_scheduled_work_orders = display::DisplayableScheduledWorkOrders(self.scheduler_agent_algorithm.optimized_work_orders.scheduled_work_orders.clone());

        // println!("manual resources {}", display_manual_resources);
        // println!("Scheduled work orders {}", display_scheduled_work_orders);
        let actor_addr = ctx.address().clone();

        let fut = async move {
            sleep(Duration::from_secs(1)).await;
            actor_addr.do_send(ScheduleIteration {});
        };

        ctx.notify(MessageToFrontend {});

        Box::pin(actix::fut::wrap_future::<_, Self>(fut))
    }
}

struct MessageToFrontend {}

impl Message for MessageToFrontend {
    type Result = ();
}

impl Handler<MessageToFrontend> for SchedulerAgent {
    type Result = ();

    fn handle(&mut self, msg: MessageToFrontend, ctx: &mut Self::Context) -> Self::Result {
        let scheduling_overview_data = self.extract_state_to_scheduler_overview().clone();

        let scheduler_frontend_message = SchedulerFrontendMessage {
            frontend_message_type: "frontend_scheduler_overview".to_string(),
            scheduling_overview_data: scheduling_overview_data,
        };

        let nested_loadings = transform_hashmap_to_nested_hashmap(self.scheduler_agent_algorithm.manual_resources_loading.clone());
        
        let scheduler_frontend_loading_message = SchedulerFrontendLoadingMessage {
            frontend_message_type: "frontend_scheduler_loading".to_string(),
            manual_resources_loading: nested_loadings,
        };
        
        match self.ws_agent_addr.as_ref() {
            Some(ws_agent) => {
                ws_agent.do_send(scheduler_frontend_message);
                ws_agent.do_send(scheduler_frontend_loading_message);
            }
            None => {println!("The websocket agent address is not set")}
        }
    }
}

impl Handler<SchedulerRequests> for SchedulerAgent {
    type Result = ();
    fn handle(&mut self, msg: SchedulerRequests, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SchedulerRequests::Input(msg) => {
                println!("SchedulerAgentReceived a FrontEnd message");
                let input_message: InputSchedulerMessage = msg.into();
                self.update_scheduler_state(input_message);
            }   
            SchedulerRequests::WorkPlanner(msg) => {
               println!("SchedulerAgentReceived a WorkPlannerMessage message");
            },
            SchedulerRequests::ExecuteIteration => {
                self.execute_iteration(ctx);
            }
        }
    }
}

impl Handler<SetAgentAddrMessage<WebSocketAgent>> for SchedulerAgent {
    type Result = ();

    fn handle(&mut self, msg: SetAgentAddrMessage<WebSocketAgent>, ctx: &mut Self::Context) -> Self::Result {
        self.set_ws_agent_addr(msg.addr);
    }
}

impl SchedulerAgent {
    pub fn execute_iteration(&mut self, ctx: &mut <SchedulerAgent as Actor>::Context) {
        println!("I am running a single iteration");  
        ctx.notify(SchedulerRequests::ExecuteIteration)
    }
}

impl SchedulerAgent {
    pub fn new(
        platform: String, 
        scheduler_agent_algorithm: SchedulerAgentAlgorithm,
        ws_agent_addr: Option<Addr<WebSocketAgent>>) 
            -> Self {
  
        Self {
            platform,
            scheduler_agent_algorithm,
            ws_agent_addr,
        }
    }
}

impl SchedulerAgentAlgorithm {
    pub fn new(
        manual_resources_capacity: HashMap<(String, String), f64>, 
        manual_resources_loading: HashMap<(String, String), f64>, 
        backlog: WorkOrders, 
        priority_queues: PriorityQueues<u32, u32>,
        optimized_work_orders: OptimizedWorkOrders,
        periods: Vec<Period>,
    ) -> Self {
        SchedulerAgentAlgorithm {
            manual_resources_capacity,
            manual_resources_loading,
            backlog,
            priority_queues,
            optimized_work_orders,
            periods            
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
impl SchedulerAgent {
    pub fn update_scheduler_state(&mut self, input_message: InputSchedulerMessage) {
        self.scheduler_agent_algorithm.manual_resources_capacity = input_message.get_manual_resources();

        for work_order_period_mapping in input_message.work_order_period_mappings {
            let work_order_number: u32 = work_order_period_mapping.work_order_number;
            let optimized_work_orders = &self.scheduler_agent_algorithm.optimized_work_orders.inner;

            let locked_in_period = work_order_period_mapping.period_status.locked_in_period;
            let excluded_from_periods =  work_order_period_mapping.period_status.excluded_from_periods;
            
            let scheduled_period = optimized_work_orders.get(&work_order_number)
                .map(|ow| ow.scheduled_period.clone())
                .unwrap_or(locked_in_period.clone());

            let optimized_work_order = OptimizedWorkOrder {
                    scheduled_period,
                    locked_in_period: locked_in_period.clone(),
                    excluded_from_periods,
            };
            self.scheduler_agent_algorithm.optimized_work_orders.inner.insert(work_order_number, optimized_work_order);
        }
    }
}

impl SchedulerAgent {
    fn populate_priority_queues(&mut self) -> () {
        for (key, work_order) in self.scheduler_agent_algorithm.backlog.inner.iter() {
            if work_order.unloading_point.present {
                event!(tracing::Level::INFO , "Work order {} has been added to the unloading queue", key);
                self.scheduler_agent_algorithm.priority_queues.unloading.push(*key, work_order.order_weight);
            } else if work_order.revision.shutdown || work_order.vendor {
                event!(tracing::Level::INFO , "Work order {} has been added to the shutdown/vendor queue", key);
                self.scheduler_agent_algorithm.priority_queues.shutdown_vendor.push(*key, work_order.order_weight);
            } else {
                event!(tracing::Level::INFO , "Work order {} has been added to the normal queue", key);
                self.scheduler_agent_algorithm.priority_queues.normal.push(*key, work_order.order_weight);
            }
        }
    }
}

pub struct PriorityQueues<T, P> 
    where T: Hash + Eq,
          P: Ord
{ 
    unloading: PriorityQueue<T, P>,
    shutdown_vendor: PriorityQueue<T, P>,
    normal: PriorityQueue<T, P>,
    manual_schedule: PriorityQueue<T, P>,
}

impl PriorityQueues<u32, u32> {
    pub fn new() -> Self{
        Self {
            unloading: PriorityQueue::<u32, u32>::new(),
            shutdown_vendor: PriorityQueue::<u32, u32>::new(),
            normal: PriorityQueue::<u32, u32>::new(),
            manual_schedule: PriorityQueue::<u32, u32>::new(),
        }
    }
}

#[derive(Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
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

// Now the problem is that the many work orders may not even get a status, in this approach.
// This is an issue. Now when we get the work_order_number the entry could be non-existent. 
// 
impl SchedulerAgent {
    fn extract_state_to_scheduler_overview(&self) -> Vec<SchedulingOverviewData> {
        let mut scheduling_overview_data: Vec<SchedulingOverviewData> = Vec::new();
        for (work_order_number, work_order) in self.scheduler_agent_algorithm.backlog.inner.iter() {
            for (operation_number, operation) in work_order.operations.clone() {
                let scheduling_overview_data_item = SchedulingOverviewData {
                    scheduled_period: match self.scheduler_agent_algorithm.optimized_work_orders.inner.get(work_order_number) {
                        Some(order_period) => {
                            match order_period.scheduled_period.as_ref() { 
                                Some(scheduled_period) => scheduled_period.period_string.clone(),
                                None => "not scheduled".to_string(),
                            }
                        },
                        None => "not scheduled".to_string(),
                    },
                    scheduled_start: work_order.order_dates.basic_start_date.to_string(),
                    unloading_point: work_order.unloading_point.clone().string, 

                    material_date: match work_order.status_codes.material_status {
                        MaterialStatus::Smat => "SMAT".to_string(),
                        MaterialStatus::Nmat => "NMAT".to_string(),
                        MaterialStatus::Cmat => "CMAT".to_string(),
                        MaterialStatus::Wmat => "WMAT".to_string(),
                        MaterialStatus::Pmat => "PMAT".to_string(),
                        MaterialStatus::Unknown => "Implement control tower".to_string(),
                    },
                    
                    work_order_number: work_order_number.clone(),
                    activity: operation_number.clone().to_string(),
                    work_center: operation.work_center.clone(),
                    work_remaining: operation.work_remaining.to_string(),
                    number: operation.number,
                    notes_1: work_order.order_text.notes_1.clone(),
                    notes_2: work_order.order_text.notes_2.clone().to_string(),
                    order_description: work_order.order_text.order_description.clone(),
                    object_description: work_order.order_text.object_description.clone(),
                    order_user_status: work_order.order_text.order_user_status.clone(),
                    order_system_status: work_order.order_text.order_system_status.clone(),
                    functional_location: work_order.functional_location.clone().string,
                    revision: work_order.revision.clone().string,
                    earliest_start_datetime: operation.earliest_start_datetime.to_string(),
                    earliest_finish_datetime: operation.earliest_finish_datetime.to_string(),
                    earliest_allowed_starting_date: work_order.order_dates.earliest_allowed_start_date.to_string(),
                    latest_allowed_finish_date: work_order.order_dates.latest_allowed_finish_date.to_string(),
                    order_type: match work_order.order_type.clone() {
                        WorkOrderType::WDF(wdf_priority) => "WDF".to_string(),
                        WorkOrderType::WGN(wgn_priority) => "WGN".to_string(),
                        WorkOrderType::WPM(wpm_priority) => "WPM".to_string(),
                        WorkOrderType::Other => "Missing Work Order Type".to_string(),
                    },
                    priority: match work_order.priority.clone() {
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

impl OptimizedWorkOrder {
    pub fn new(
        scheduled_period: Option<Period>, 
        locked_in_period: Option<Period>, 
        excluded_from_periods: HashSet<Period>) -> Self {
        
        Self {
            scheduled_period,
            locked_in_period,
            excluded_from_periods,
        }
    }

    pub fn with_new_schedule(&mut self, scheduled_period: Option<Period>) -> Self {
        Self {
            scheduled_period: scheduled_period,
            locked_in_period: self.locked_in_period.clone(),
            excluded_from_periods: self.excluded_from_periods.clone(),
        }
    }

    pub fn update_scheduled_period(&mut self, period: Option<Period>) {
        self.scheduled_period = period;
    }
}

/// This function should be reformulated? I think that we should make sure to create in such a way
/// that. We need an inner hashmap for each of the different 
fn transform_hashmap_to_nested_hashmap(hash_map: HashMap<(String, String), f64>) -> HashMap<String, HashMap<String, f64>> {
    let mut nested_hash_map: HashMap<String, HashMap<String, f64>> = HashMap::new();
    
    for ((work_center, period), value) in hash_map {
        nested_hash_map.entry(work_center)
            .or_insert_with(HashMap::new)
            .insert(period, value);
    }
    nested_hash_map
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_scheduler_agent_initialization() {

    }


}