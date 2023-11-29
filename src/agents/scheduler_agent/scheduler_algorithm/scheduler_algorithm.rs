use std::collections::HashSet;
use core::panic;
use tracing::{event};

use crate::agents::scheduler_agent::scheduler_algorithm::QueueType;
use crate::agents::scheduler_agent::scheduler_algorithm::OptimizedWorkOrder;
use crate::models::time_environment::period::Period;
use crate::models::work_order::WorkOrder;

use super::SchedulerAgentAlgorithm;

/// This implementation of the SchedulerAgent will do the following. It should take a messgage
/// and then return a scheduler 
/// 
/// Okay, so the problem is that we never get through to the actual scheduling part for the 
/// normal queue.
impl SchedulerAgentAlgorithm {
    pub fn schedule_normal_work_orders(&mut self, queue_type: QueueType) -> () {
        let periods = self.periods.clone();
        for period in periods {
            let work_orders_to_schedule: Vec<_> = {

                let current_queue = match queue_type {
                    QueueType::Normal => &mut self.priority_queues.normal,
                    QueueType::UnloadingAndManual => &mut self.priority_queues.unloading,
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
                let inf_wo_key = self.schedule_normal_work_order(work_order_key, &period, &queue_type);
                match inf_wo_key {
                    Some(inf_wo_key) => {
                        let work_order = self.backlog.inner.get(&inf_wo_key);
                        match work_order {
                            Some(work_order) => {
                                let current_queue = match queue_type {
                                    QueueType::Normal => &mut self.priority_queues.normal,
                                    QueueType::UnloadingAndManual => &mut self.priority_queues.unloading,
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
    
    /// How should the forced schedule be different? We already know the period here and we already 
    /// know the period. What should be done about this. This depends on where we can find the 
    /// period. We can find the period in the part of the state that is handled by the 
    /// update_scheduler_state function. 
    pub fn schedule_forced_work_orders(&mut self) {
        let mut work_order_keys: Vec<u32> = vec![];
        for (work_order_key, opt_work_order) in self.get_optimized_work_orders().iter() {
            if opt_work_order.locked_in_period.is_some() {
                work_order_keys.push(*work_order_key);
            }
        }

        for work_order_key in work_order_keys {
            self.schedule_forced_work_order(work_order_key);
        }
    }

    /// The queue type here should be changed. The problem is that the unloading point scheduling is
    /// fundamentally different and we should therefore handle it in a different place than where we
    /// initially thought. The schedule_normal_work_orders should simply schedule work orders that 
    /// are not in the schedule yet. I think, but I am not sure, that the there should be no 
    /// rescheduling here. 
    #[tracing::instrument(fields(
        manual_resources_capacity = self.manual_resources_capacity.len(),
        manual_resources_loading = self.manual_resources_loading.len(),
        optimized_work_orders = self.optimized_work_orders.inner.len(),))]
    pub fn schedule_normal_work_order(&mut self, work_order_key: u32, period: &Period, queue_type: &QueueType) -> Option<u32> {

        let work_order = self.backlog.inner.get(&work_order_key).unwrap().clone();
        
        // The if statements found in here are each constraints that has to be upheld.
        for (work_center, resource_needed) in work_order.work_load.iter() {

            let resource_capacity: &mut f64 = self.manual_resources_capacity.entry((work_center.to_string(), period.clone().period_string)).or_insert(0.0);                             
            let resource_loading: &mut f64 = self.manual_resources_loading.entry((work_center.to_string(), period.clone().period_string)).or_insert(0.0);
            
            if *resource_needed > *resource_capacity - *resource_loading {
                return Some(work_order_key);
            }
            
            if period.get_end_date() < work_order.order_dates.earliest_allowed_start_date {
                return Some(work_order_key);
            }

            if let Some(optimized_work_order) = self.optimized_work_orders.inner.get(&work_order_key) {
                if optimized_work_order.excluded_from_periods.contains(&period) {
                    return Some(work_order_key);
                } 
            }
        }

        match self.optimized_work_orders.inner.get_mut(&work_order_key) {
            Some(optimized_work_order) => {
                optimized_work_order.update_scheduled_period(Some(period.clone()));
                self.changed = true;
            },
            None => {
                self.optimized_work_orders.inner.insert(work_order_key, OptimizedWorkOrder::new(Some(period.clone()), None, HashSet::new()));
                self.changed = true;
            }
        }
        
        event!(tracing::Level::INFO , "Work order {} from the normal has been scheduled", work_order_key);

        self.update_loadings(period, &work_order);
        None
    }


    pub fn schedule_forced_work_order(&mut self, work_order_key: u32) {
        match self.is_scheduled(work_order_key) {
            Some(work_order_key) => self.unschedule_work_order(work_order_key),
            None => (),
        }
        
        let period = self.optimized_work_orders.get_locked_in_period(work_order_key);
        
        self.initialize_loading_used_in_work_order(work_order_key, period.clone());
        
        let work_order = self.backlog.inner.get(&work_order_key).unwrap();
        // self.optimized_work_orders.
        event!(tracing::Level::INFO , "Work order {} has been scheduled with unloading point or manual", work_order_key);
        
        self.optimized_work_orders.set_scheduled_period(work_order_key, period.clone());
        self.changed = true;

        // Is this really the place where we should update the loadings? I am not sure about it. 
        // It is either here or in the update_scheduler_state function. Well thank you for that
        for (work_center_period, loading) in self.manual_resources_loading.iter_mut() {
            if work_center_period.1 == *period.period_string {
                *loading += work_order.work_load.get(&work_center_period.0).unwrap_or(&0.0);
            }
        }
    }
}

// This becomes more simple right? if the key exists you simply change the period. Or else you 
// create a new entry. It should not be needed to create a new entry as we already have, be 
// definition received it from the front end. No that is only if we are in the manual queue. 

// There are more problems here. We now have to make sure that the work order is unscheduled 
// correctly. And then updated correctly.

/// Okay here we have a super chance to apply testing. That will be a crucial step towards making
/// this system scale. Now this stop, you cannot keep not testing you code. Okay, so we should 
/// change the implementation so that the work orders that are "manually" scheduled are simply
/// forced into the schedule. There is no reason to loop over every period to fix the problem. 
impl SchedulerAgentAlgorithm {
    fn is_scheduled(&self, work_order_key: u32) -> Option<u32> {
        match self.optimized_work_orders.inner.get(&work_order_key) {
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
        let work_order = self.backlog.inner.get(&work_order_key).unwrap();
        let period = self.optimized_work_orders.inner.get(&work_order_key).as_ref().unwrap().scheduled_period.as_ref().unwrap(); 


        for (work_center_period, loading) in self.manual_resources_loading.iter_mut() {
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
        self.optimized_work_orders.inner.get_mut(&work_order_key).unwrap().update_scheduled_period(None);
    }

    fn update_loadings(&mut self, period: &Period, work_order: &WorkOrder) -> () {
        for (work_center_period, loading) in self.manual_resources_loading.iter_mut() {
            if work_center_period.1 == *period.period_string {
                *loading += work_order.work_load.get(&work_center_period.0).unwrap_or(&0.0);
            }
        }
    }
}

impl SchedulerAgentAlgorithm {
    pub fn set_optimized_work_order(&mut self, work_order_key: u32, optimized_work_order: OptimizedWorkOrder) {
        self.optimized_work_orders.inner.insert(work_order_key, optimized_work_order);
    }
}


/// Test scheduler scheduling logic
/// Make your own trait that you can use
/// 

#[cfg(test)]
mod tests {
    use super::*;
    
    use std::collections::HashMap;
    use chrono::{TimeZone, Utc};

    use crate::{agents::scheduler_agent::scheduler_algorithm::{SchedulerAgentAlgorithm, PriorityQueues, OptimizedWorkOrders}, models::{WorkOrders, work_order::{WorkOrder, priority::Priority, order_type::{WorkOrderType, WDFPriority}, status_codes::StatusCodes, order_dates::OrderDates, revision::Revision, unloading_point::UnloadingPoint, functional_location::FunctionalLocation, order_text::OrderText, system_condition::SystemCondition}}};
    
    #[test]
    fn test_schedule_work_order() {
        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
        let end_date = start_date + chrono::Duration::days(13);

        let period = Period::new(1, start_date, end_date);

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
            WorkOrderType::WDF(WDFPriority::new(1)),
            SystemCondition::new(),
            StatusCodes::new_default(),
            OrderDates::new_default(),
            Revision::new_default(),
            UnloadingPoint::new_default(),
            FunctionalLocation::new_default(),
            OrderText::new_default(),
            false,
        );

        work_orders.insert(work_order);

        let mut scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            HashMap::new(), 
            HashMap::new(), 
            work_orders, 
            PriorityQueues::new(), 
            OptimizedWorkOrders::new(HashMap::new()),
            vec![],
            true
        );     


        scheduler_agent_algorithm.schedule_normal_work_order(2200002020, &period, &QueueType::Normal);

        assert_eq!(scheduler_agent_algorithm.optimized_work_orders.inner.get(&2200002020).unwrap().scheduled_period, Some(period.clone()));      

    }

    #[test]
    fn test_schedule_work_order_with_work_load() {
        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
        let end_date = start_date + chrono::Duration::days(13);

        let period = Period::new(1, start_date, end_date);

        let mut work_orders = WorkOrders::new();

        let mut work_load = HashMap::new();

        work_load.insert("MTN_MECH".to_string(), 100.0);

        let work_order = WorkOrder::new(
            2200002020,
            false,
            1000,
            Priority::new_int(1),
            100.0,
            HashMap::new(),
            work_load,
            vec![],
            vec![],
            vec![],
            WorkOrderType::WDF(WDFPriority::new(1)),
            SystemCondition::new(),
            StatusCodes::new_default(),
            OrderDates::new_default(),
            Revision::new_default(),
            UnloadingPoint::new_default(),
            FunctionalLocation::new_default(),
            OrderText::new_default(),
            false,
        );

        work_orders.insert(work_order);

        let mut scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            HashMap::new(), 
            HashMap::new(), 
            work_orders, 
            PriorityQueues::new(), 
            OptimizedWorkOrders::new(HashMap::new()),
            vec![],
            true
        );     
        scheduler_agent_algorithm.schedule_normal_work_order(2200002020, &period, &QueueType::Normal);

        assert_eq!(scheduler_agent_algorithm.optimized_work_orders.inner.get(&2200002020), None);      
    }

    #[test]
    fn test_update_loadings() {

        let mut work_orders = WorkOrders::new();

        let mut work_load = HashMap::new();

        work_load.insert("MTN_MECH".to_string(), 20.0);
        work_load.insert("MTN_ELEC".to_string(), 40.0);
        work_load.insert("PRODTECH".to_string(), 60.0);

        let work_order = WorkOrder::new(
            2200002020,
            false,
            1000,
            Priority::new_int(1),
            100.0,
            HashMap::new(),
            work_load,
            vec![],
            vec![],
            vec![],
            WorkOrderType::WDF(WDFPriority::new(1)),
            SystemCondition::new(),
            StatusCodes::new_default(),
            OrderDates::new_default(),
            Revision::new_default(),
            UnloadingPoint::new_default(),
            FunctionalLocation::new_default(),
            OrderText::new_default(),
            false,
        );
    
        work_orders.insert(work_order.clone());

        // The structure is quite nested in this case as we have the backlog that is in the 
        // scheduling agent algorithm and then we pull it out and then we schedule it again. 


        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
        let end_date = start_date + chrono::Duration::days(13);

        let period = Period::new(1, start_date, end_date);

        let mut manual_resource_capacity: HashMap<(String, String), f64> = HashMap::new();
        let mut manual_resource_loadings: HashMap<(String, String), f64> = HashMap::new();

        manual_resource_capacity.insert(("MTN_MECH".to_string(), period.period_string.clone()), 150.0);
        manual_resource_capacity.insert(("MTN_ELEC".to_string(), period.period_string.clone()), 150.0);
        manual_resource_capacity.insert(("PRODTECH".to_string(), period.period_string.clone()), 150.0);

        manual_resource_loadings.insert(("MTN_MECH".to_string(), period.period_string.clone()), 0.0);
        manual_resource_loadings.insert(("MTN_ELEC".to_string(), period.period_string.clone()), 0.0);
        manual_resource_loadings.insert(("PRODTECH".to_string(), period.period_string.clone()), 0.0);

        let mut scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            manual_resource_capacity, 
            manual_resource_loadings, 
            work_orders, 
            PriorityQueues::new(), 
            OptimizedWorkOrders::new(HashMap::new()),
            vec![],
            true
        );     

        scheduler_agent_algorithm.update_loadings(&period, &work_order);

        assert_eq!(scheduler_agent_algorithm.manual_resources_loading.get(&("MTN_MECH".to_string(), period.period_string.clone())), Some(20.0).as_ref());
        assert_eq!(scheduler_agent_algorithm.manual_resources_loading.get(&("MTN_ELEC".to_string(), period.period_string.clone())), Some(40.0).as_ref());
        assert_eq!(scheduler_agent_algorithm.manual_resources_loading.get(&("PRODTECH".to_string(), period.period_string.clone())), Some(60.0).as_ref());

        assert_eq!(scheduler_agent_algorithm.manual_resources_loading.get(&("MTN_SCAF".to_string(), period.period_string.clone())), None);
    }



    /// This test fails as we cannot schedule with the unloading point queue if we do not have a
    /// period lock in the OptimizedWorkOrders. What should I do about it? I am not sure about it?
    /// In general I have an issue with the way that the static data is handled in the program and 
    /// the way that the dynamic data is handled in the program. What should I do about it? I am not
    /// sure
    #[test]
    fn test_unschedule_work_order() {

        let mut work_orders = WorkOrders::new();
        let mut work_load = HashMap::new();

        work_load.insert("MTN_MECH".to_string(), 20.0);
        work_load.insert("MTN_ELEC".to_string(), 40.0);
        work_load.insert("PRODTECH".to_string(), 60.0);

        let work_order = WorkOrder::new(
            2200002020,
            false,
            1000,
            Priority::new_int(1),
            100.0,
            HashMap::new(),
            work_load,
            vec![],
            vec![],
            vec![],
            WorkOrderType::WDF(WDFPriority::new(1)),
            SystemCondition::new(),
            StatusCodes::new_default(),
            OrderDates::new_default(),
            Revision::new_default(),
            UnloadingPoint::new_default(),
            FunctionalLocation::new_default(),
            OrderText::new_default(),
            false,
        );
    
        work_orders.insert(work_order.clone());

        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
        let end_date = start_date + chrono::Duration::days(13);

        let period = Period::new(1, start_date, end_date);

        let mut manual_resource_capacity: HashMap<(String, String), f64> = HashMap::new();
        let mut manual_resource_loadings: HashMap<(String, String), f64> = HashMap::new();

        manual_resource_capacity.insert(("MTN_MECH".to_string(), period.period_string.clone()), 150.0);
        manual_resource_capacity.insert(("MTN_ELEC".to_string(), period.period_string.clone()), 150.0);
        manual_resource_capacity.insert(("PRODTECH".to_string(), period.period_string.clone()), 150.0);

        manual_resource_loadings.insert(("MTN_MECH".to_string(), period.period_string.clone()), 0.0);
        manual_resource_loadings.insert(("MTN_ELEC".to_string(), period.period_string.clone()), 0.0);
        manual_resource_loadings.insert(("PRODTECH".to_string(), period.period_string.clone()), 0.0);

        let mut scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            manual_resource_capacity, 
            manual_resource_loadings, 
            work_orders, 
            PriorityQueues::new(), 
            OptimizedWorkOrders::new(HashMap::new()),
            vec![],
            true
        );     

        scheduler_agent_algorithm.schedule_normal_work_order(2200002020, &period, &QueueType::Normal);

        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("MTN_MECH".to_string(), period.period_string.clone()), 20.0);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("MTN_ELEC".to_string(), period.period_string.clone()), 40.0);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("PRODTECH".to_string(), period.period_string.clone()), 60.0);

        scheduler_agent_algorithm.unschedule_work_order(2200002020);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("MTN_MECH".to_string(), period.period_string.clone()), 0.0);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("MTN_ELEC".to_string(), period.period_string.clone()), 0.0);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("PRODTECH".to_string(), period.period_string.clone()), 0.0);
        
        let optimized_work_order = OptimizedWorkOrder::new(None, Some(period.clone()), HashSet::new());

        scheduler_agent_algorithm.set_optimized_work_order(2200002020, optimized_work_order);

        scheduler_agent_algorithm.schedule_forced_work_order(2200002020);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("MTN_MECH".to_string(), period.period_string.clone()), 20.0);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("MTN_ELEC".to_string(), period.period_string.clone()), 40.0);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("PRODTECH".to_string(), period.period_string.clone()), 60.0);

        let start_date_new = start_date + chrono::Duration::days(14);
        let end_date_new = start_date_new + chrono::Duration::days(13);

        let period_new = Period::new(1, start_date_new, end_date_new);

        scheduler_agent_algorithm.optimized_work_orders.set_locked_in_period(2200002020, period_new.clone());

        scheduler_agent_algorithm.schedule_forced_work_order(2200002020);
        
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("MTN_MECH".to_string(), period_new.period_string.clone()), 20.0);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("MTN_ELEC".to_string(), period_new.period_string.clone()), 40.0);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("PRODTECH".to_string(), period_new.period_string.clone()), 60.0);

        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("MTN_MECH".to_string(), period.period_string.clone()), 0.0);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("MTN_ELEC".to_string(), period.period_string.clone()), 0.0);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("PRODTECH".to_string(), period.period_string.clone()), 0.0);

        scheduler_agent_algorithm.unschedule_work_order(2200002020);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("MTN_MECH".to_string(), period.period_string.clone()), 0.0);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("MTN_ELEC".to_string(), period.period_string.clone()), 0.0);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("PRODTECH".to_string(), period.period_string.clone()), 0.0);
        
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("MTN_MECH".to_string(), period_new.period_string.clone()), 0.0);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("MTN_ELEC".to_string(), period_new.period_string.clone()), 0.0);
        assert_eq!(scheduler_agent_algorithm.get_or_initialize_manual_resources_loading("PRODTECH".to_string(), period_new.period_string.clone()), 0.0);
    }
}