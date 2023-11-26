use std::collections::HashSet;
use core::panic;
use tracing::{event};

use crate::agents::scheduler_agent::scheduler_algorithm::QueueType;
use crate::agents::scheduler_agent::scheduler_algorithm::OptimizedWorkOrder;
use crate::agents::scheduler_agent::SchedulerAgent;
use crate::models::period::Period;

/// This implementation of the SchedulerAgent will do the following. It should take a messgage
/// and then return a scheduler 
/// 
/// Okay, so the problem is that we never get through to the actual scheduling part for the 
/// normal queue.
impl SchedulerAgent {
    pub fn schedule_work_orders_by_type(&mut self, queue_type: QueueType) -> () {
        let periods = self.scheduler_agent_algorithm.periods.clone();
        for period in periods {
            let work_orders_to_schedule: Vec<_> = {

                let current_queue = match queue_type {
                    QueueType::Normal => &mut self.scheduler_agent_algorithm.priority_queues.normal,
                    QueueType::UnloadingAndManual => &mut self.scheduler_agent_algorithm.priority_queues.unloading,
                    QueueType::ShutdownVendor => &mut self.scheduler_agent_algorithm.priority_queues.shutdown_vendor,
                };
                
                let mut work_orders_to_schedule = Vec::new();

                while !current_queue.is_empty() {
                    let (work_order_key, _weight) = match current_queue.pop() {
                        Some(work_order) => work_order,
                        None => panic!("The scheduler priority queue is empty and this should not happen."),
                    };
                    work_orders_to_schedule.push(work_order_key);
                };
                work_orders_to_schedule
            };

            for work_order_key in work_orders_to_schedule {
                let inf_wo_key = self.schedule_work_order(work_order_key, &period, &queue_type);
                match inf_wo_key {
                    Some(inf_wo_key) => {
                        let work_order = self.scheduler_agent_algorithm.backlog.inner.get(&inf_wo_key);
                        match work_order {
                            Some(work_order) => {
                                let current_queue = match queue_type {
                                    QueueType::Normal => &mut self.scheduler_agent_algorithm.priority_queues.normal,
                                    QueueType::UnloadingAndManual => &mut self.scheduler_agent_algorithm.priority_queues.unloading,
                                    QueueType::ShutdownVendor => &mut self.scheduler_agent_algorithm.priority_queues.shutdown_vendor,
                                };
                                current_queue.push(inf_wo_key, work_order.order_weight);
                            }
                            None => (),
                        }
                    }
                    None => (),
                }
            }
        }
    }
    
    #[tracing::instrument(fields(
        manual_resources_capacity = self.scheduler_agent_algorithm.manual_resources_capacity.len(),
        manual_resources_loading = self.scheduler_agent_algorithm.manual_resources_loading.len(),
        optimized_work_orders = self.scheduler_agent_algorithm.optimized_work_orders.inner.len(),))]
    pub fn schedule_work_order(&mut self, work_order_key: u32, period: &Period, queue_type: &QueueType) -> Option<u32> {
        match queue_type {
            QueueType::Normal => {

                let work_order = self.scheduler_agent_algorithm.backlog.inner.get(&work_order_key).unwrap();
                
                // The if statements found in here are each constraints that has to be upheld.
                for (work_center, resource_needed) in work_order.work_load.iter() {

                    let resource_capacity: &mut f64 = self.scheduler_agent_algorithm.manual_resources_capacity.entry((work_center.to_string(), period.clone().period_string)).or_insert(0.0);                             
                    let resource_loading: &mut f64 = self.scheduler_agent_algorithm.manual_resources_loading.entry((work_center.to_string(), period.clone().period_string)).or_insert(0.0);
                    
                    if *resource_needed > *resource_capacity - *resource_loading {
                        return Some(work_order_key);
                    }
                    
                    if period.get_end_date() < work_order.order_dates.earliest_allowed_start_date {
                        return Some(work_order_key);
                    }

                    if let Some(optimized_work_order) = self.scheduler_agent_algorithm.optimized_work_orders.inner.get(&work_order_key) {
                        if optimized_work_order.excluded_from_periods.contains(&period) {
                            return Some(work_order_key);
                        } 
                    }
                }

                match self.scheduler_agent_algorithm.optimized_work_orders.inner.get_mut(&work_order_key) {
                    Some(optimized_work_order) => {
                        optimized_work_order.update_scheduled_period(Some(period.clone()));
                    },
                    None => {
                        self.scheduler_agent_algorithm.optimized_work_orders.inner.insert(work_order_key, OptimizedWorkOrder::new(Some(period.clone()), None, HashSet::new()));
                    }
                }
              
                event!(tracing::Level::INFO , "Work order {} from the normal has been scheduled", work_order_key);
                for (work_center_period, loading) in self.scheduler_agent_algorithm.manual_resources_loading.iter_mut() {
                    if work_center_period.1 == *period.period_string {
                        *loading += work_order.work_load.get(&work_center_period.0).unwrap_or(&0.0);
                    }
                }
                None
            }

            QueueType::UnloadingAndManual => {
                match self.is_scheduled(work_order_key) {
                    Some(work_order_key) => self.unschedule_work_order(work_order_key),
                    None => (),
                }

                let work_order = self.scheduler_agent_algorithm.backlog.inner.get(&work_order_key).unwrap();

                // ! The error is here
                match self.scheduler_agent_algorithm.optimized_work_orders.inner.get(&work_order_key) {
                    Some(optimized_work_order) => {
                        match optimized_work_order.locked_in_period.clone() {
                            Some(locked_period) => {
                                // event!(target: "frontend input message debugging", Level::INFO, "Period loop: {} Locked period: {} on work order {}", &period, locked_period.period_string.clone(), &work_order_key);
                        
                                if period.period_string != locked_period.period_string {
                                    return Some(work_order_key);
                                }
                            }
                            None => panic!("The locked period should not be None"),
                        }
                    }
                    None => panic!("The optimized work order should not be None"),
                }
                
                event!(tracing::Level::INFO , "Work order {} has been scheduled with unloading point or manual", work_order_key);

                self.scheduler_agent_algorithm.optimized_work_orders.inner.get_mut(&work_order_key).unwrap().scheduled_period = Some(period.clone());
                
                for (work_center_period, loading) in self.scheduler_agent_algorithm.manual_resources_loading.iter_mut() {
                    if work_center_period.1 == *period.period_string {
                        *loading += work_order.work_load.get(&work_center_period.0).unwrap_or(&0.0);
                    }
                }
                None
            }
            QueueType::ShutdownVendor => {None}
        }
    }
}

// This becomes more simple right? if the key exists you simply change the period. Or else you 
// create a new entry. It should not be needed to create a new entry as we already have, be 
// definition received it from the front end. No that is only if we are in the manual queue. 

// There are more problems here. We now have to make sure that the work order is unscheduled 
// correctly. And then updated correctly.

/// Okay here we have a super chance to apply testing. That will be a crucial step towards making
/// this system scale. Now this stop, you cannot keep not testing you code. 
impl SchedulerAgent {
    fn is_scheduled(&self, work_order_key: u32) -> Option<u32> {
        match self.scheduler_agent_algorithm.optimized_work_orders.inner.get(&work_order_key) {
            Some(optimized_work_order) => {
                match optimized_work_order.scheduled_period {
                    Some(_) => return Some(work_order_key),
                    None => return None,
                }
            }
            None => return None,
        }
    }

    fn unschedule_work_order(&mut self, work_order_key: u32) {
        let work_order = self.scheduler_agent_algorithm.backlog.inner.get(&work_order_key).unwrap();
        let period = self.scheduler_agent_algorithm.optimized_work_orders.inner.get(&work_order_key).as_ref().unwrap().scheduled_period.as_ref().unwrap(); 

        for (work_center_period, loading) in self.scheduler_agent_algorithm.manual_resources_loading.iter_mut() {
            if work_center_period.1 == period.period_string {
                let work_load_for_work_center = work_order.work_load.get(&work_center_period.0);
                match work_load_for_work_center {
                    Some(work_load_for_work_center) => {
                        *loading -= work_load_for_work_center;
                    }
                    None => (),
                }
            }
        }
        self.scheduler_agent_algorithm.optimized_work_orders.inner.get_mut(&work_order_key).unwrap().update_scheduled_period(None);
    }
}


/// Test scheduler scheduling logic
/// Make your own trait that you can use
/// 

#[cfg(test)]
mod tests {

    use actix::Addr;

    use crate::agents::scheduler_agent::scheduler_algorithm::SchedulerAgentAlgorithm;

    use super::*;
    
    // #[test]
    // fn test_scheduler_scheduling_logic() {

    //     let mock_web_socket = Addr::new();

    //     let scheduler_agent = SchedulerAgent::new(
    //         "Test Platform".to_string(),
    //         SchedulerAgentAlgorithm::new(),
    //         mock_web_socket

    // );

    // }



}