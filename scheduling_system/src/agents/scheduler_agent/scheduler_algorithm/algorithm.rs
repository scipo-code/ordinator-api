use core::panic;
use rand::prelude::SliceRandom;
use std::collections::HashSet;
use tracing::info;

use super::SchedulerAgentAlgorithm;
use crate::agents::scheduler_agent::scheduler_algorithm::OptimizedWorkOrder;
use crate::agents::scheduler_agent::scheduler_algorithm::QueueType;
use crate::models::time_environment::period::Period;
use crate::models::work_order::WorkOrder;

/// This implementation of the SchedulerAgent will do the following. It should take a messgage
/// and then return a scheduler
///
/// Okay, so the problem is that we never get through to the actual scheduling part for the
/// normal queue.
impl SchedulerAgentAlgorithm {
    pub fn schedule_normal_work_orders(&mut self, queue_type: QueueType) {
        let periods = self.periods.clone();
        for period in periods {
            let work_orders_to_schedule: Vec<_> = {
                let mut work_orders_to_schedule = Vec::new();

                while !self.priority_queues.normal.is_empty() {
                    let (work_order_key, _weight) = match self.priority_queues.normal.pop() {
                        Some(work_order) => work_order,
                        None => panic!(
                            "The scheduler priority queue is empty and this should not happen."
                        ),
                    };
                    work_orders_to_schedule.push(work_order_key);
                }
                work_orders_to_schedule
            };

            for work_order_key in work_orders_to_schedule {
                let inf_wo_key =
                    self.schedule_normal_work_order(work_order_key, &period, &queue_type);
                if let Some(wo_key) = inf_wo_key {
                    let work_order = self.backlog.inner.get(&wo_key);
                    if let Some(work_order) = work_order {
                        self.priority_queues
                            .normal
                            .push(wo_key, work_order.get_order_weight());
                    }
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
        manual_resources_capacity = self.resources_capacity.len(),
        manual_resources_loading = self.resources_loading.len(),
        optimized_work_orders = self.optimized_work_orders.inner.len(),))]
    pub fn schedule_normal_work_order(
        &mut self,
        work_order_key: u32,
        period: &Period,
        queue_type: &QueueType,
    ) -> Option<u32> {
        let mut work_order = self.backlog.inner.get(&work_order_key).unwrap().clone();

        // The if statements found in here are each constraints that has to be upheld.
        for (work_center, resource_needed) in work_order.get_work_load().clone().iter() {
            let resource_capacity: &mut f64 = self
                .resources_capacity
                .entry((work_center.clone(), period.clone()))
                .or_insert(0.0);
            let resource_loading: &mut f64 = self
                .resources_loading
                .entry((work_center.clone(), period.clone()))
                .or_insert(0.0);

            if *resource_needed > *resource_capacity - *resource_loading {
                return Some(work_order_key);
            }

            if period.get_end_date() < work_order.get_order_dates().earliest_allowed_start_date {
                return Some(work_order_key);
            }

            if let Some(optimized_work_order) =
                self.optimized_work_orders.inner.get(&work_order_key)
            {
                if optimized_work_order.excluded_from_periods.contains(period) {
                    return Some(work_order_key);
                }
            }
        }

        match self.optimized_work_orders.inner.get_mut(&work_order_key) {
            Some(optimized_work_order) => {
                optimized_work_order.update_scheduled_period(Some(period.clone()));
                self.changed = true;
            }
            None => {
                self.optimized_work_orders.inner.insert(
                    work_order_key,
                    OptimizedWorkOrder::new(Some(period.clone()), None, HashSet::new()),
                );
                self.changed = true;
            }
        }

        info!(
            "Work order {} from the normal has been scheduled",
            work_order_key
        );
        self.update_loadings(period, &work_order);
        None
    }

    pub fn schedule_forced_work_order(&mut self, work_order_key: u32) {
        if let Some(work_order_key) = self.is_scheduled(work_order_key) {
            self.unschedule_work_order(&work_order_key);
        }

        let period = self
            .optimized_work_orders
            .get_locked_in_period(work_order_key);

        self.initialize_loading_used_in_work_order(work_order_key, period.clone());

        let work_order = self.backlog.inner.get(&work_order_key).unwrap();
        // self.optimized_work_orders.
        info!(
            "Work order {} has been scheduled with unloading point or manual",
            work_order_key
        );

        self.optimized_work_orders
            .set_scheduled_period(work_order_key, period.clone());
        self.changed = true;

        // Is this really the place where we should update the loadings? I am not sure about it.
        // It is either here or in the update_scheduler_state function. Well thank you for that
        for (work_center_period, loading) in self.resources_loading.iter_mut() {
            if work_center_period.1 == period {
                *loading += work_order
                    .get_work_load()
                    .get(&work_center_period.0)
                    .unwrap_or(&0.0);
            }
        }
    }

    /// Does it matter that we clone the work orders. I think that it does not matter here as the
    /// code should be able to work as long as the optimized work orders are updated correctly and
    /// not clone anywhere. After sampling the work orders based on the keys I should take care to
    ///
    pub fn unschedule_random_work_orders(
        &mut self,
        number_of_work_orders: usize,
        rng: &mut impl rand::Rng,
    ) {
        let mut work_order_keys: Vec<_> =
            self.get_optimized_work_orders().keys().cloned().collect();

        work_order_keys.sort();
        let sampled_work_order_keys = work_order_keys.choose_multiple(rng, number_of_work_orders);

        for work_order_key in sampled_work_order_keys {
            self.unschedule_work_order(work_order_key);
        }

        self.changed = true;
    }

    /// Here we calculate the objective value. If the work order is has been unscheduled then we
    /// do not have a Period in the scheduled_period. If we do not have a period in the
    ///
    /// So what should be done if a work order in not scheduled? I think that the best approach will
    /// be to "schedule" it outside of the initialized periods. That will make a lot of sense. I
    /// think that it will be the best approach. Actually every work order should be scheduled like
    /// this to make the system consistent. And make the objective value meaningful.
    pub fn calculate_objective(&mut self) {
        let mut objective = 0;
        for (work_order_key, optimized_work_order) in &self.optimized_work_orders.inner {
            let optimized_period = match &optimized_work_order.scheduled_period {
                Some(optimized_period) => optimized_period.clone(),
                None => {
                    if let Some(last_period) = self.periods.last() {
                        last_period.add_one_period()
                    } else {
                        panic!("There are no periods in the system")
                    }
                }
            }
            .clone();

            let work_order_latest_allowed_finish_period = self
                .backlog
                .inner
                .get_mut(work_order_key)
                .unwrap()
                .get_order_dates()
                .latest_allowed_finish_period
                .clone();

            let period_difference = calculate_period_difference(
                optimized_period,
                work_order_latest_allowed_finish_period,
            );
            let objective_contribution = if period_difference > 0 {
                period_difference
                    * self
                        .backlog
                        .inner
                        .get(work_order_key)
                        .unwrap()
                        .get_order_weight() as i64
            } else {
                0
            };
            objective += objective_contribution;
        }
        self.objective_value = objective as f64;
    }
}

fn calculate_period_difference(period_1: Period, period_2: Period) -> i64 {
    let period_1_date = period_1.get_end_date();
    let period_2_date = period_2.get_end_date();

    let duration = period_1_date.signed_duration_since(period_2_date);

    let days = duration.num_days();

    days / 7
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
        self.optimized_work_orders
            .inner
            .get(&work_order_key)
            .and_then(|optimized_work_order| {
                optimized_work_order
                    .scheduled_period
                    .as_ref()
                    .map(|_| work_order_key)
            })
    }

    /// Here we unschedule the work order that means that we should take care to. What is the error
    /// here. What did I actually just implement? We are moving fast at the moment. I can feel that
    /// I get overwhelmed by the amount of work that it will take. I think that we should continue
    /// moving forward and meditate if needed. There is an error here and instead of testing for it
    /// am brute forcing my way through it. I should not do that I should test the code and make a
    /// test that fails. Unschedule work order basically means that we should move the work order
    /// to the first period that is outside of the initialized periods. What is the best appraoch
    /// for doing this? The problem is that we have changed the API for the unschedule work order
    /// and also the whole meaning of what it means to be scheduld. Should I be allowed to
    /// unschedule a work order that is not scheduled? If so the logic should be that we set the
    /// scheduled period to the prospective period. That actually means that we have changed the
    /// problem setup. But it is the way that it should be.
    fn unschedule_work_order(&mut self, work_order_key: &u32) {
        let work_order = self.backlog.inner.get(work_order_key).unwrap();
        let period = match &self.optimized_work_orders.inner.get(work_order_key) {
            Some(optimized_work_order) => match &optimized_work_order.scheduled_period {
                Some(period) => Some(period.clone()),
                None => {
                    if let Some(last_period) = self.periods.last() {
                        Some(last_period.add_one_period())
                    } else {
                        panic!("There are no periods in the system")
                    }
                }
            },
            None => {
                if let Some(last_period) = self.periods.last() {
                    Some(last_period.add_one_period())
                } else {
                    panic!("There are no periods in the system")
                }
            }
        }
        .unwrap();

        // The period is used to make sure that we update the loading correctly. The loading that
        // should be updated is the is the one found in the period that the work order was scheduled
        // in and not the period that the work order is moving to. This is a crucial difference.

        // One of the assumptions of the unschedule work order is that the the work order is
        // actually scheduled. This is not the case here as I am implementing logic to handle cases
        // where the work order is not scheduled.
        // What does it mean when we call the unwrap here?
        for (work_center_period, loading) in self.resources_loading.iter_mut() {
            if work_center_period.1 == period {
                let work_load_for_work_center =
                    work_order.get_work_load().get(&work_center_period.0);
                if let Some(work_load_for_work_center) = work_load_for_work_center {
                    *loading -= work_load_for_work_center;
                }
            }
        }
        let prospective_period = self.periods.last().unwrap().add_one_period();

        match self.optimized_work_orders.inner.get_mut(work_order_key) {
            Some(optimized_work_order) => {
                optimized_work_order.update_scheduled_period(Some(prospective_period));
                self.changed = true;
            }
            None => {
                self.optimized_work_orders.inner.insert(
                    *work_order_key,
                    OptimizedWorkOrder::new(Some(prospective_period), None, HashSet::new()),
                );
                self.changed = true;
            }
        }
    }

    fn update_loadings(&mut self, period: &Period, work_order: &WorkOrder) {
        for (work_center_period, loading) in self.resources_loading.iter_mut() {
            if work_center_period.1 == *period {
                *loading += work_order
                    .get_work_load()
                    .get(&work_center_period.0)
                    .unwrap_or(&0.0);
            }
        }
    }
}

impl SchedulerAgentAlgorithm {
    #[cfg(test)]
    pub fn set_optimized_work_order(
        &mut self,
        work_order_key: u32,
        optimized_work_order: OptimizedWorkOrder,
    ) {
        self.optimized_work_orders
            .inner
            .insert(work_order_key, optimized_work_order);
    }
}

/// Test scheduler scheduling logic
/// Make your own trait that you can use
#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{Duration, TimeZone, Utc};
    use rand::{rngs::StdRng, SeedableRng};
    use std::collections::HashMap;

    use crate::{
        agents::scheduler_agent::scheduler_algorithm::{
            OptimizedWorkOrders, PriorityQueues, SchedulerAgentAlgorithm,
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
                system_condition::SystemCondition,
                unloading_point::UnloadingPoint,
                WorkOrder,
            },
            WorkOrders,
        },
    };
    use shared_messages::resources::Resources;

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
            WorkOrderType::Wdf(WDFPriority::new(1)),
            SystemCondition::new(),
            StatusCodes::new_default(),
            OrderDates::new_test(),
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
            true,
        );

        scheduler_agent_algorithm.schedule_normal_work_order(
            2200002020,
            &period,
            &QueueType::Normal,
        );

        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200002020)
                .unwrap()
                .scheduled_period,
            Some(period.clone())
        );
    }

    #[test]
    fn test_schedule_work_order_with_work_load() {
        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
        let end_date = start_date + chrono::Duration::days(13);

        let period = Period::new(1, start_date, end_date);

        let mut work_orders = WorkOrders::new();

        let mut work_load = HashMap::new();

        work_load.insert(Resources::new_from_string("MTN-MECH".to_string()), 100.0);

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
            WorkOrderType::Wdf(WDFPriority::new(1)),
            SystemCondition::new(),
            StatusCodes::new_default(),
            OrderDates::new_test(),
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
            true,
        );
        scheduler_agent_algorithm.schedule_normal_work_order(
            2200002020,
            &period,
            &QueueType::Normal,
        );

        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200002020),
            None
        );
    }

    #[test]
    fn test_update_loadings() {
        let mut work_orders = WorkOrders::new();

        let mut work_load = HashMap::new();

        work_load.insert(Resources::new_from_string("MTN-MECH".to_string()), 20.0);
        work_load.insert(Resources::new_from_string("MTN-ELEC".to_string()), 40.0);
        work_load.insert(Resources::new_from_string("PRODTECH".to_string()), 60.0);

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
            WorkOrderType::Wdf(WDFPriority::new(1)),
            SystemCondition::new(),
            StatusCodes::new_default(),
            OrderDates::new_test(),
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

        let mut manual_resource_capacity: HashMap<(Resources, Period), f64> = HashMap::new();
        let mut manual_resource_loadings: HashMap<(Resources, Period), f64> = HashMap::new();

        manual_resource_capacity.insert((Resources::MtnMech, period.clone()), 150.0);
        manual_resource_capacity.insert((Resources::MtnElec, period.clone()), 150.0);
        manual_resource_capacity.insert((Resources::Prodtech, period.clone()), 150.0);

        manual_resource_loadings.insert((Resources::MtnMech, period.clone()), 0.0);
        manual_resource_loadings.insert((Resources::MtnElec, period.clone()), 0.0);
        manual_resource_loadings.insert((Resources::Prodtech, period.clone()), 0.0);

        let mut scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            manual_resource_capacity,
            manual_resource_loadings,
            work_orders,
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            vec![],
            true,
        );

        scheduler_agent_algorithm.update_loadings(&period, &work_order);

        assert_eq!(
            scheduler_agent_algorithm
                .resources_loading
                .get(&(Resources::MtnMech, period.clone())),
            Some(20.0).as_ref()
        );
        assert_eq!(
            scheduler_agent_algorithm
                .resources_loading
                .get(&(Resources::MtnElec, period.clone())),
            Some(40.0).as_ref()
        );
        assert_eq!(
            scheduler_agent_algorithm
                .resources_loading
                .get(&(Resources::Prodtech, period.clone())),
            Some(60.0).as_ref()
        );

        assert_eq!(
            scheduler_agent_algorithm
                .resources_loading
                .get(&(Resources::MtnScaf, period.clone())),
            None
        );
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

        work_load.insert(Resources::MtnMech, 20.0);
        work_load.insert(Resources::MtnElec, 40.0);
        work_load.insert(Resources::Prodtech, 60.0);

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
            WorkOrderType::Wdf(WDFPriority::new(1)),
            SystemCondition::new(),
            StatusCodes::new_default(),
            OrderDates::new_test(),
            Revision::new_default(),
            UnloadingPoint::new_default(),
            FunctionalLocation::new_default(),
            OrderText::new_default(),
            false,
        );

        work_orders.insert(work_order.clone());

        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 0, 0, 0).unwrap();
        let end_date = start_date
            + chrono::Duration::days(13)
            + chrono::Duration::hours(23)
            + chrono::Duration::minutes(59)
            + chrono::Duration::seconds(59);
        let period_1 = Period::new(0, start_date, end_date);
        let period_2 = Period::new(0, start_date, end_date) + Duration::weeks(2);
        let period_3 = Period::new(0, start_date, end_date) + Duration::weeks(4);

        // Again, this is not completely correct. There is an invariant here that is not being
        // upheld correctly. What should we do about that?
        let mut manual_resource_capacity: HashMap<(Resources, Period), f64> = HashMap::new();
        let mut manual_resource_loadings: HashMap<(Resources, Period), f64> = HashMap::new();

        manual_resource_capacity.insert((Resources::MtnMech, period_1.clone()), 150.0);
        manual_resource_capacity.insert((Resources::MtnElec, period_1.clone()), 150.0);
        manual_resource_capacity.insert((Resources::Prodtech, period_1.clone()), 150.0);

        manual_resource_loadings.insert((Resources::MtnMech, period_1.clone()), 0.0);
        manual_resource_loadings.insert((Resources::MtnElec, period_1.clone()), 0.0);
        manual_resource_loadings.insert((Resources::Prodtech, period_1.clone()), 0.0);

        manual_resource_capacity.insert((Resources::MtnMech, period_2.clone()), 150.0);
        manual_resource_capacity.insert((Resources::MtnElec, period_2.clone()), 150.0);
        manual_resource_capacity.insert((Resources::Prodtech, period_2.clone()), 150.0);

        manual_resource_loadings.insert((Resources::MtnMech, period_2.clone()), 0.0);
        manual_resource_loadings.insert((Resources::MtnElec, period_2.clone()), 0.0);
        manual_resource_loadings.insert((Resources::Prodtech, period_2.clone()), 0.0);

        manual_resource_capacity.insert((Resources::MtnMech, period_3.clone()), 150.0);
        manual_resource_capacity.insert((Resources::MtnElec, period_3.clone()), 150.0);
        manual_resource_capacity.insert((Resources::Prodtech, period_3.clone()), 150.0);

        manual_resource_loadings.insert((Resources::MtnMech, period_3.clone()), 0.0);
        manual_resource_loadings.insert((Resources::MtnElec, period_3.clone()), 0.0);
        manual_resource_loadings.insert((Resources::Prodtech, period_3.clone()), 0.0);

        let periods: Vec<Period> = vec![period_1.clone(), period_2.clone(), period_3.clone()];

        let mut scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            manual_resource_capacity,
            manual_resource_loadings,
            work_orders.clone(),
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            periods,
            true,
        );

        scheduler_agent_algorithm.schedule_normal_work_order(
            2200002020,
            &period_1,
            &QueueType::Normal,
        );

        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::MtnMech, period_1.clone()),
            20.0
        );

        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::MtnElec, period_1.clone()),
            40.0
        );
        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::Prodtech, period_1.clone()),
            60.0
        );

        scheduler_agent_algorithm.unschedule_work_order(&2200002020);
        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::MtnMech, period_1.clone()),
            0.0
        );
        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::MtnElec, period_1.clone()),
            0.0
        );
        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::Prodtech, period_1.clone()),
            0.0
        );

        let optimized_work_order =
            OptimizedWorkOrder::new(None, Some(period_1.clone()), HashSet::new());

        scheduler_agent_algorithm.set_optimized_work_order(2200002020, optimized_work_order);

        scheduler_agent_algorithm.schedule_forced_work_order(2200002020);

        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::MtnMech, period_1.clone()),
            20.0
        );
        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::MtnElec, period_1.clone()),
            40.0
        );
        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::Prodtech, period_1.clone()),
            60.0
        );

        scheduler_agent_algorithm
            .optimized_work_orders
            .set_locked_in_period(2200002020, period_2.clone());
        scheduler_agent_algorithm.schedule_forced_work_order(2200002020);

        assert_eq!(
            scheduler_agent_algorithm.get_or_initialize_manual_resources_loading(
                Resources::MtnMech.clone(),
                period_2.clone()
            ),
            20.0
        );
        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::MtnElec, period_2.clone()),
            40.0
        );
        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::Prodtech, period_2.clone()),
            60.0
        );

        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::MtnMech, period_1.clone()),
            0.0
        );
        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::MtnElec, period_1.clone()),
            0.0
        );
        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::Prodtech, period_1.clone()),
            0.0
        );

        scheduler_agent_algorithm.unschedule_work_order(&2200002020);
        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::MtnMech, period_1.clone()),
            0.0
        );
        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::MtnElec, period_1.clone()),
            0.0
        );
        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::Prodtech, period_1.clone()),
            0.0
        );

        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::MtnMech, period_2.clone()),
            0.0
        );
        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::MtnElec, period_2.clone()),
            0.0
        );
        assert_eq!(
            scheduler_agent_algorithm
                .get_or_initialize_manual_resources_loading(Resources::Prodtech, period_2.clone()),
            0.0
        );
    }

    #[test]
    fn test_unschedule_random_work_orders() {
        let mut work_orders = WorkOrders::new();
        let mut work_load_1 = HashMap::new();
        let mut work_load_2 = HashMap::new();
        let mut work_load_3 = HashMap::new();

        work_load_1.insert(Resources::MtnMech, 10.0);
        work_load_1.insert(Resources::MtnElec, 10.0);
        work_load_1.insert(Resources::Prodtech, 10.0);

        work_load_2.insert(Resources::MtnMech, 20.0);
        work_load_2.insert(Resources::MtnElec, 20.0);
        work_load_2.insert(Resources::Prodtech, 20.0);

        work_load_3.insert(Resources::MtnMech, 30.0);
        work_load_3.insert(Resources::MtnElec, 30.0);
        work_load_3.insert(Resources::Prodtech, 30.0);

        let work_order_1 = WorkOrder::new(
            2200000001,
            false,
            1000,
            Priority::new_int(1),
            100.0,
            HashMap::new(),
            work_load_1,
            vec![],
            vec![],
            vec![],
            WorkOrderType::Wdf(WDFPriority::new(1)),
            SystemCondition::new(),
            StatusCodes::new_default(),
            OrderDates::new_test(),
            Revision::new_default(),
            UnloadingPoint::new_default(),
            FunctionalLocation::new_default(),
            OrderText::new_default(),
            false,
        );

        let work_order_2 = WorkOrder::new(
            2200000002,
            false,
            1000,
            Priority::new_int(1),
            100.0,
            HashMap::new(),
            work_load_2,
            vec![],
            vec![],
            vec![],
            WorkOrderType::Wdf(WDFPriority::new(1)),
            SystemCondition::new(),
            StatusCodes::new_default(),
            OrderDates::new_test(),
            Revision::new_default(),
            UnloadingPoint::new_default(),
            FunctionalLocation::new_default(),
            OrderText::new_default(),
            false,
        );

        let work_order_3 = WorkOrder::new(
            2200000003,
            false,
            1000,
            Priority::new_int(1),
            100.0,
            HashMap::new(),
            work_load_3,
            vec![],
            vec![],
            vec![],
            WorkOrderType::Wdf(WDFPriority::new(1)),
            SystemCondition::new(),
            StatusCodes::new_default(),
            OrderDates::new_test(),
            Revision::new_default(),
            UnloadingPoint::new_default(),
            FunctionalLocation::new_default(),
            OrderText::new_default(),
            false,
        );

        work_orders.insert(work_order_1);
        work_orders.insert(work_order_2);
        work_orders.insert(work_order_3);

        let mut optimized_work_orders = OptimizedWorkOrders::new(HashMap::new());

        let optimized_work_order_1 = OptimizedWorkOrder::new(
            Some(Period::new_from_string("2023-W47-48").unwrap()),
            None,
            HashSet::new(),
        );

        let optimized_work_order_2 = OptimizedWorkOrder::new(
            Some(Period::new_from_string("2023-W47-48").unwrap()),
            None,
            HashSet::new(),
        );

        let optimized_work_order_3 = OptimizedWorkOrder::new(
            Some(Period::new_from_string("2023-W49-50").unwrap()),
            None,
            HashSet::new(),
        );

        optimized_work_orders
            .inner
            .insert(2200000001, optimized_work_order_1);
        optimized_work_orders
            .inner
            .insert(2200000002, optimized_work_order_2);
        optimized_work_orders
            .inner
            .insert(2200000003, optimized_work_order_3);

        let periods: Vec<Period> = vec![
            Period::new_from_string("2023-W47-48").unwrap(),
            Period::new_from_string("2023-W49-50").unwrap(),
        ];

        let mut scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            HashMap::new(),
            HashMap::new(),
            work_orders,
            PriorityQueues::new(),
            optimized_work_orders,
            periods,
            true,
        );

        let seed: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];

        let mut rng = StdRng::from_seed(seed);

        scheduler_agent_algorithm.unschedule_random_work_orders(2, &mut rng);

        assert_eq!(
            *scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200000001)
                .unwrap()
                .scheduled_period
                .as_ref()
                .unwrap(),
            Period::new_from_string("2023-W47-48").unwrap()
        );

        assert_eq!(
            *scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200000002)
                .unwrap()
                .scheduled_period
                .as_ref()
                .unwrap(),
            Period::new_from_string("2023-W51-52").unwrap()
        );

        assert_eq!(
            *scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200000003)
                .unwrap()
                .scheduled_period
                .as_ref()
                .unwrap(),
            Period::new_from_string("2023-W51-52").unwrap()
        );
    }

    #[test]
    fn test_calculate_period_difference() {
        let period_1 = Period::new_from_string("2023-W47-48");
        let period_2 = Period::new_from_string("2023-W49-50");

        let difference = calculate_period_difference(period_1.unwrap(), period_2.unwrap());

        assert_eq!(difference, -2);
    }

    #[test]
    fn test_choose_multiple() {
        for _ in 0..19 {
            let seed: [u8; 32] = [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ];

            let mut rng = StdRng::from_seed(seed);

            assert_eq!(
                [1, 2, 3].choose_multiple(&mut rng, 2).collect::<Vec<_>>(),
                [&3, &2]
            );
        }
    }

    #[test]
    fn test_unschedule_work_order_none_in_scheduled_period() {
        let work_order = WorkOrder::default();

        let mut work_orders = WorkOrders::new();

        work_orders.insert(work_order);

        let optimized_work_orders = OptimizedWorkOrders::new(HashMap::new());

        let mut scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
            0.0,
            HashMap::new(),
            HashMap::new(),
            work_orders,
            PriorityQueues::new(),
            optimized_work_orders,
            vec![Period::new_from_string("2023-W47-48").unwrap()],
            true,
        );

        scheduler_agent_algorithm.unschedule_work_order(&2100000001);
        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2100000001)
                .unwrap()
                .scheduled_period,
            Some(Period::new_from_string("2023-W49-50").unwrap())
        );
    }

    #[test]
    fn test_period_clone_equality() {
        let period_1 = Period::new_from_string("2023-W47-48").unwrap();
        let period_2 = Period::new_from_string("2023-W47-48").unwrap();

        assert_eq!(period_1, period_2);
        assert_eq!(period_1, period_1.clone());
    }
}
