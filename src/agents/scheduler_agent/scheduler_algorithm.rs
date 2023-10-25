use crate::models::order_period::OrderPeriod;
use crate::models::scheduling_environment::WorkOrders;
use crate::agents::scheduler_agent::SchedulerAgent;
use crate::models::period::Period;
use crate::models::work_order;
use priority_queue::PriorityQueue;

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
            // if self.scheduler_agent_algorithm.backlog.inner.get(&work_order_key).unwrap().unloading_point.present {
            for work_order_key in work_orders_to_schedule {
                let inf_wo_key = self.schedule_work_order(work_order_key, &period, &queue_type);
                inf_wos.inner.insert(inf_wo_key, self.scheduler_agent_algorithm.backlog.inner.remove(&inf_wo_key).unwrap());
            }
           
        }
    }

    pub fn schedule_work_order(&mut self, work_order_key: u32, period: &Period, queue_type: &QueueType) -> u32 {
        match queue_type {
            QueueType::Normal => {
                let work_order = self.scheduler_agent_algorithm.backlog.inner.get(&work_order_key).unwrap();
                for (work_center, resource_needed) in work_order.work_load.iter() {
                    let resource_capacity: &mut f64 = self.scheduler_agent_algorithm.manual_resources_capacity.get_mut(&(work_center.to_string(), period.clone())).unwrap();
                    let resource_loading: &mut f64 = self.scheduler_agent_algorithm.manual_resources_loading.get_mut(&(work_center.to_string(), period.clone())).unwrap();
                    
                    if *resource_needed > *resource_capacity - *resource_loading {
                        return work_order_key;
                    }
                    if period.get_end_date() < work_order.order_dates.earliest_allowed_start_date {
                        return work_order_key;
                    }
                }

                self.scheduler_agent_algorithm.scheduled_work_orders.insert(work_order_key, OrderPeriod::new(period.clone(), work_order_key));

                for (work_center_period, loading) in self.scheduler_agent_algorithm.manual_resources_loading.iter_mut() {
                    if work_center_period.1 == *period {
                        *loading += work_order.work_load.get(&work_center_period.0).unwrap();
                    }
                }
                0
            }
            QueueType::Unloading => {0}
            QueueType::ShutdownVendor => {0}
        }
    }
}



