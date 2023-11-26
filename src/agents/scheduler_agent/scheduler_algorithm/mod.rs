mod scheduler_algorithm;

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::fmt::Display;

use priority_queue::PriorityQueue;
use tracing::{span, event, Level};

use crate::agents::scheduler_agent::InputSchedulerMessage;
use crate::models::scheduling_environment::WorkOrders;
use crate::models::period::Period;

#[derive(Debug)]
pub struct SchedulerAgentAlgorithm {
    objective_value: f64,
    manual_resources_capacity : HashMap<(String, String), f64>,
    manual_resources_loading: HashMap<(String, String), f64>,
    backlog: WorkOrders,
    priority_queues: PriorityQueues<u32, u32>,
    optimized_work_orders: OptimizedWorkOrders,
    periods: Vec<Period>,
    changed: bool,
}

impl SchedulerAgentAlgorithm {
    pub fn get_backlog(&self) -> &WorkOrders {
        &self.backlog
    }

    pub fn get_optimized_work_order(&self, work_order_number: &u32) -> Option<&OptimizedWorkOrder> {
        self.optimized_work_orders.inner.get(work_order_number)
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn set_changed(&mut self, changed: bool) {
        self.changed = changed;
    }
}


#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct OptimizedWorkOrder {
    pub scheduled_period: Option<Period>,
    pub locked_in_period: Option<Period>,
    pub excluded_from_periods: HashSet<Period>,
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

    pub fn get_scheduled_period(&self) -> Option<Period> {
        self.scheduled_period.clone()
    }

    pub fn update_scheduled_period(&mut self, period: Option<Period>) {
        self.scheduled_period = period;
    }
}


impl Display for SchedulerAgentAlgorithm {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "SchedulerAgentAlgorithm: \n
            objective_value: {}, \n
            manual_resources_capacity: {:?}, \n
            manual_resources_loading: {:?}, \n
            backlog: {:?}, \n
            priority_queues: {:?}, \n
            optimized_work_orders: {:?}, \n
            periods: {:?}", 
            self.objective_value, 
            self.manual_resources_capacity,
            self.manual_resources_loading,
            self.backlog,
            self.priority_queues,
            self.optimized_work_orders,
            self.periods)
    }

}

impl SchedulerAgentAlgorithm {
    pub fn log_optimized_work_orders(&self) {
        for (work_order_number, optimized) in &self.optimized_work_orders.inner {
            
            match &optimized.locked_in_period {
                Some(period) => {
                    event!(tracing::Level::TRACE, work_order_number = %work_order_number, period = period.period_string)
                }
                None => event!(tracing::Level::TRACE, work_order_number = %work_order_number,  period = "no locked period")
            }

            match &optimized.scheduled_period {
                Some(period) => {
                    event!(tracing::Level::TRACE, work_order_number = %work_order_number, period = %period.period_string)
                }
                None => event!(tracing::Level::TRACE, work_order_number = %work_order_number,  period = "None")
            }

            for period in &optimized.excluded_from_periods {
                event!(tracing::Level::TRACE, work_order_number = %work_order_number, period = %period)
            }
        }
    }
}

impl SchedulerAgentAlgorithm {
    
    #[tracing::instrument(name = "update_scheduler_state", level = "DEBUG", skip(self, input_message))]
    pub fn update_scheduler_state(&mut self, input_message: InputSchedulerMessage) {

        let _span = span!(Level::INFO, "update_scheduler_state");
        self.manual_resources_capacity = input_message.get_manual_resources();

        for work_order_period_mapping in input_message.work_order_period_mappings {
            let message = match self.optimized_work_orders.inner.get(&work_order_period_mapping.work_order_number) {
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
            let optimized_work_orders = &self.optimized_work_orders.inner;

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
            self.optimized_work_orders.inner.insert(work_order_number, optimized_work_order);
            self.update_priority_queues();
        }
    }
}

impl SchedulerAgentAlgorithm {
    pub fn populate_priority_queues(&mut self) -> () {
        for (key, work_order) in self.backlog.inner.iter() {
            if work_order.unloading_point.present  {
                event!(tracing::Level::DEBUG , "Work order {} has been added to the unloading queue", key);
                self.priority_queues.unloading.push(*key, work_order.order_weight);
            } else if work_order.revision.shutdown || work_order.vendor {
                event!(tracing::Level::DEBUG , "Work order {} has been added to the shutdown/vendor queue", key);
                self.priority_queues.shutdown_vendor.push(*key, work_order.order_weight);
            } else {
                event!(tracing::Level::DEBUG , "Work order {} has been added to the normal queue", key);
                self.priority_queues.normal.push(*key, work_order.order_weight);
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
        for (key, work_order) in &self.optimized_work_orders.inner {
            let work_order_weight = self.backlog.inner.get(&key).unwrap().order_weight;
            match &work_order.locked_in_period {
                Some(_work_order) => {
                    self.priority_queues.unloading.push(*key, work_order_weight);
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

impl SchedulerAgentAlgorithm {
    pub fn new(
        objective_value: f64,
        manual_resources_capacity: HashMap<(String, String), f64>, 
        manual_resources_loading: HashMap<(String, String), f64>, 
        backlog: WorkOrders, 
        priority_queues: PriorityQueues<u32, u32>,
        optimized_work_orders: OptimizedWorkOrders,
        periods: Vec<Period>,
        changed: bool,
    ) -> Self {
        SchedulerAgentAlgorithm {
            objective_value,
            manual_resources_capacity,
            manual_resources_loading,
            backlog,
            priority_queues,
            optimized_work_orders,
            periods,      
            changed
        }
    }

    pub fn get_optimized_work_orders(&self) -> &HashMap<u32, OptimizedWorkOrder> {
        &self.optimized_work_orders.inner
    }

    pub fn get_manual_resources_loading(&self) -> &HashMap<(String, String), f64> {
        &self.manual_resources_loading
    }
}

#[derive(Debug, PartialEq)]
pub enum QueueType {
    Normal,
    UnloadingAndManual,
    ShutdownVendor,
}


#[cfg(test)]
mod tests {

}