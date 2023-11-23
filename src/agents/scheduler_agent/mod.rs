pub mod scheduler_message;
pub mod scheduler_algorithm;
pub mod display;

use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use actix::prelude::*; 



use priority_queue::PriorityQueue;
use tracing::Level;
use tracing::{event, span};

use crate::models::work_order::priority::Priority;
use crate::models::work_order::order_type::WorkOrderType;
use crate::agents::scheduler_agent::scheduler_message::{InputSchedulerMessage, ScheduleIteration};
use crate::models::scheduling_environment::WorkOrders;
use crate::models::period::Period;
use crate::api::websocket_agent::WebSocketAgent;

use crate::models::work_order::status_codes::MaterialStatus;


#[derive(Debug)]
pub struct SchedulerAgent {
    platform: String,
    scheduler_agent_algorithm: SchedulerAgentAlgorithm,
    ws_agent_addr: Option<Addr<WebSocketAgent>>,
}

#[derive(Debug)]
pub struct SchedulerAgentAlgorithm {
    manual_resources_capacity : HashMap<(String, String), f64>,
    manual_resources_loading: HashMap<(String, String), f64>,
    backlog: WorkOrders,
    priority_queues: PriorityQueues<u32, u32>,
    optimized_work_orders: OptimizedWorkOrders,
    periods: Vec<Period>,
}

#[derive(Debug)]
pub struct OptimizedWorkOrders {
    inner: HashMap<u32, OptimizedWorkOrder>,
}

impl Hash for OptimizedWorkOrders {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the length of the HashMap to ensure different lengths produce different hashes
        self.inner.len().hash(state);

        // Iterate over the HashMap and hash each key-value pair
        for (key, value) in &self.inner {
            key.hash(state);
            value.hash(state);
        }
    }
}

#[derive(Debug)]
pub struct OptimizedWorkOrder {
    scheduled_period: Option<Period>,
    locked_in_period: Option<Period>,
    excluded_from_periods: HashSet<Period>,
}

impl Hash for OptimizedWorkOrder {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the length of the HashMap to ensure different lengths produce different hashes
       
        self.scheduled_period.hash(state);
        self.locked_in_period.hash(state);
        for period in &self.excluded_from_periods {
            period.hash(state);
        }

    }
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

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        println!("SchedulerAgent is stopped");
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
/// 
/// So here we update the state of the application, but what about the queues? I after the work 
/// order has been scheduled in the front end we need to update the queues. As well so that the 
/// work order is scheduled through the process. We should add the work order to the unloading point
/// queue but what will happen when the work order is unscheduled again at a later point? This is 
/// much more difficult to reason about. I think that the best approach is 
/// 
/// All of this should be handled in the update scheduler state function. There can be no other way
/// Remember that if this becomes complex we should refactor the code. 
impl SchedulerAgent {
    
    #[tracing::instrument(name = "update_scheduler_state", level = "DEBUG", skip(self, input_message))]
    pub fn update_scheduler_state(&mut self, input_message: InputSchedulerMessage) {

        let _span = span!(Level::INFO, "update_scheduler_state");
        self.scheduler_agent_algorithm.manual_resources_capacity = input_message.get_manual_resources();


        for work_order_period_mapping in input_message.work_order_period_mappings {
            let message = match self.scheduler_agent_algorithm.optimized_work_orders.inner.get(&work_order_period_mapping.work_order_number) {
                Some(work_order) => {format!(
                    "work_order is suggested in {:?} \n 
                    work_order is scheduled in {:?} \n
                    work_order is excluded {:?} \n",
                    work_order.scheduled_period,
                    work_order.locked_in_period,
                    work_order.excluded_from_periods
                    )
                }
                None => {
                    "work_order is not in optimized work orders".to_string()
                }
            };

            event!(tracing::Level::DEBUG, "scheduler optimized work order state before update{}", message);

            event!(tracing::Level::DEBUG, "The manual resources are: {:?}", work_order_period_mapping);

            let work_order_number: u32 = work_order_period_mapping.work_order_number;
            let optimized_work_orders = &self.scheduler_agent_algorithm.optimized_work_orders.inner;

            let locked_in_period = work_order_period_mapping.period_status.locked_in_period;
            let excluded_from_periods =  work_order_period_mapping.period_status.excluded_from_periods;
            
            let scheduled_period = optimized_work_orders.get(&work_order_number)
                .map(|ow| ow.scheduled_period.clone())
                .unwrap_or(locked_in_period.clone());

            match locked_in_period.clone() {
                Some(period) => {
                    event!(target: "frontend input message debugging", Level::DEBUG, "Locked period: {}", period.period_string.clone());
                }
                None => {
                    event!(target: "frontend input message debugging", Level::DEBUG, "Locked period: None");
                }
            }

            let optimized_work_order = OptimizedWorkOrder {
                scheduled_period,
                locked_in_period: locked_in_period.clone(),
                excluded_from_periods,
            };
            
            let mut excluded_periods = "".to_string();
            for period in &optimized_work_order.excluded_from_periods {
                excluded_periods += &(period.to_string() + &" ".to_string());
            }

            event!(tracing::Level::DEBUG, 
                work_order_number = %work_order_number, 
                info = "Work order updated", 
                suggested_period = match &optimized_work_order.scheduled_period {
                    Some(period) => period.period_string.clone(), 
                    None => "no suggested period".to_string()
                },
                locked_in_period = match &optimized_work_order.locked_in_period {
                    Some(period) => period.period_string.clone(),
                    None => "no lock on period".to_string()
                },
                excluded_periods = %excluded_periods
            );
         
            
            self.scheduler_agent_algorithm.optimized_work_orders.inner.insert(work_order_number, optimized_work_order);

            event!(tracing::Level::TRACE, self.platform );

            self.update_priority_queues();
        }
    }
}

impl SchedulerAgent {
    fn populate_priority_queues(&mut self) -> () {
        for (key, work_order) in self.scheduler_agent_algorithm.backlog.inner.iter() {
            if work_order.unloading_point.present  {
                event!(tracing::Level::DEBUG , "Work order {} has been added to the unloading queue", key);
                self.scheduler_agent_algorithm.priority_queues.unloading.push(*key, work_order.order_weight);
            } else if work_order.revision.shutdown || work_order.vendor {
                event!(tracing::Level::DEBUG , "Work order {} has been added to the shutdown/vendor queue", key);
                self.scheduler_agent_algorithm.priority_queues.shutdown_vendor.push(*key, work_order.order_weight);
            } else {
                event!(tracing::Level::DEBUG , "Work order {} has been added to the normal queue", key);
                self.scheduler_agent_algorithm.priority_queues.normal.push(*key, work_order.order_weight);
            }
        }
    }

    /// So the idea here is that we look through all the optimized_work_orders and then we schedule
    /// them according to the queue type. There are two cases that should be covered. 
    /// 
    /// Inclusion
    ///     Here we have to move a work order to the unloading point queue. If the work order is 
    ///     already scheduled we have the logic in place to handle this. 
    ///    
    /// 
    /// Exclusion
    ///     We need to force this invariant on the data type. 
    /// 
    /// I am doing the wrong thing here. We only care about the 
    /// 
    /// The exclusion is simply a variation of the materials, EASD. In the code we should create
    /// something to handle this issue. Exclusion is already handled in the code.
    /// 
    fn update_priority_queues(&mut self) -> () {
        for (key, work_order) in &self.scheduler_agent_algorithm.optimized_work_orders.inner {
            let work_order_weight = self.scheduler_agent_algorithm.backlog.inner.get(&key).unwrap().order_weight;
            match &work_order.locked_in_period {
                Some(_work_order) => {
                    self.scheduler_agent_algorithm.priority_queues.unloading.push(*key, work_order_weight);
                }
                None => {}
            }
        }
    }
}

#[derive(Debug)]
pub struct PriorityQueues<T, P> 
    where T: Hash + Eq,
          P: Ord
{ 
    unloading: PriorityQueue<T, P>,
    shutdown_vendor: PriorityQueue<T, P>,
    normal: PriorityQueue<T, P>,
}

impl PriorityQueues<u32, u32> {
    pub fn new() -> Self{
        Self {
            unloading: PriorityQueue::<u32, u32>::new(),
            shutdown_vendor: PriorityQueue::<u32, u32>::new(),
            normal: PriorityQueue::<u32, u32>::new(),
        }
    }
}

#[derive(Clone, Debug)]
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
                        WorkOrderType::WDF(_wdf_priority) => "WDF".to_string(),
                        WorkOrderType::WGN(_wgn_priority) => "WGN".to_string(),
                        WorkOrderType::WPM(_wpm_priority) => "WPM".to_string(),
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
    #[allow(dead_code)]
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