use crate::models::order_period::OrderPeriod;
use crate::models::scheduling_environment::WorkOrders;
use crate::agents::scheduler_agent::SchedulerAgent;
use crate::models::period::{Period, PeriodNone};

use tracing::{event};

pub enum QueueType {
    Normal,
    Unloading,
    ShutdownVendor,
    // ... other queue types ...
}

/// This implementation of the SchedulerAgent will do the following. It should take a messgage
/// and then return a scheduler 
impl SchedulerAgent {

    pub fn schedule_work_orders_by_type(&mut self, queue_type: QueueType) -> () {
        let periods = self.scheduler_agent_algorithm.periods.clone();
        let mut inf_wos = WorkOrders::new();
        for period in periods {
            let work_orders_to_schedule: Vec<_> = {

                let current_queue = match queue_type {
                    QueueType::Normal => &mut self.scheduler_agent_algorithm.priority_queues.normal,
                    QueueType::Unloading => &mut self.scheduler_agent_algorithm.priority_queues.unloading,
                    QueueType::ShutdownVendor => &mut self.scheduler_agent_algorithm.priority_queues.shutdown_vendor,
                    // ... other queue types ...
                };
                
                for (inf_key, inf_work_order) in inf_wos.inner.iter() {
                    current_queue.push(*inf_key, inf_work_order.order_weight);
                }
                let mut wos = Vec::new();
                while !current_queue.is_empty() {
                    let (work_order_key, _weight) = match current_queue.pop() {
                        Some(work_order) => work_order,
                        None => panic!("The scheduler priority queue is empty and this should not happen."),
                    };
                    wos.push(work_order_key);
                };
                wos
            };

            // Here we remove an element from the backlog. That is a problem. 
            for work_order_key in work_orders_to_schedule {
                let inf_wo_key = self.schedule_work_order(work_order_key, &period, &queue_type);

                match inf_wo_key {

                    Some(inf_wo_key) => {
                        let cloned_value = self.scheduler_agent_algorithm.backlog.inner.get(&inf_wo_key);

                        match cloned_value {
                            Some(cloned_value) => {
                                inf_wos.inner.insert(inf_wo_key, cloned_value.clone());
                            }
                            None => (),
                        }
                    }
                    None => (),
                }
               
            }
        }
    }

    /// Okay now we will just recieve period strings from the frontend. What does that mean for the 
    /// backend? It mean that the periods should be initialized in the backend.
    /// 
    /// I will initialize all periods once in the backend and I will recieve the period strings from
    /// the frontend. Where does the work order key come from? What is the problem here? The problem
    /// is that we pass a work order key to the backlog for an entry that does not exist. This means
    /// one of two things. The backlog is initialized wrong, the queues are not disjoint, 
    /// 
    /// The problem is that the backlog shrinks even though nothing is scheduled
    /// 
    /// I should be testing this. I should test 
    pub fn schedule_work_order(&mut self, work_order_key: u32, period: &Period, queue_type: &QueueType) -> Option<u32> {
        match queue_type {
            QueueType::Normal => {
                let work_order = self.scheduler_agent_algorithm.backlog.inner.get(&work_order_key).unwrap();
            
                for (work_center, resource_needed) in work_order.work_load.iter() {
                    let resource_capacity: &mut f64 = self.scheduler_agent_algorithm.manual_resources_capacity.entry((work_center.to_string(), period.clone())).or_insert(0.0);
                    let resource_loading: &mut f64 = self.scheduler_agent_algorithm.manual_resources_loading.entry((work_center.to_string(), period.clone())).or_insert(0.0);
                    
                    if *resource_needed > *resource_capacity - *resource_loading {
                        return Some(work_order_key);
                    }
                    if period.get_end_date() < work_order.order_dates.earliest_allowed_start_date {
                        return Some(work_order_key);
                    }
                }
                event!(tracing::Level::INFO , "Work order {} has been scheduled", work_order_key);
                self.scheduler_agent_algorithm.scheduled_work_orders.insert(work_order_key, OrderPeriod::new(period.clone(), work_order_key));

                for (work_center_period, loading) in self.scheduler_agent_algorithm.manual_resources_loading.iter_mut() {
                    if work_center_period.1 == *period {
                        *loading += work_order.work_load.get(&work_center_period.0).unwrap();
                    }
                }
                None
            }
            QueueType::Unloading => {
                let work_order = self.scheduler_agent_algorithm.backlog.inner.get(&work_order_key).unwrap();
                
                match work_order.unloading_point.period.clone() {
                    PeriodNone::Period(unloading_period) => {
                        dbg!(unloading_period.period_string.clone());
                    }
                    PeriodNone::None =>  {
                        panic!{"The unloading point period is None and this should not happen."}
                    }
                }
                for (work_center, resource_needed) in work_order.work_load.iter() {
                    let resource_capacity: &mut f64 = self.scheduler_agent_algorithm.manual_resources_capacity.entry((work_center.to_string(), period.clone())).or_insert(0.0);
                    let resource_loading: &mut f64 = self.scheduler_agent_algorithm.manual_resources_loading.entry((work_center.to_string(), period.clone())).or_insert(0.0);
                    
                    match work_order.unloading_point.period.clone() {
                        PeriodNone::Period(unloading_period) => {
                           
                            if period.period_string != unloading_period.period_string {
                                return Some(work_order_key);
                            }
                        }
                        PeriodNone::None =>  {
                            panic!{"The unloading point period is None and this should not happen."}
                        }
                    }
                }
                 
                self.scheduler_agent_algorithm.scheduled_work_orders.insert(work_order_key, OrderPeriod::new(period.clone(), work_order_key));

                for (work_center_period, loading) in self.scheduler_agent_algorithm.manual_resources_loading.iter_mut() {
                    if work_center_period.1 == *period {
                        let work_load_for_work_center = work_order.work_load.get(&work_center_period.0);
                        match work_load_for_work_center {
                            Some(work_load_for_work_center) => {
                                *loading += work_load_for_work_center;
                            }
                            None => (),
                        }
                    }
                }
                None
            }
            QueueType::ShutdownVendor => {None}
        }
    }
}



