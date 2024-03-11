use core::panic;
use rand::prelude::SliceRandom;
use tracing::info;
use tracing::instrument;

use super::StrategicAlgorithm;
use crate::agents::strategic_agent::strategic_algorithm::OptimizedWorkOrder;
use crate::agents::traits::LargeNeighborHoodSearch;
use crate::models::time_environment::period::Period;

/// Okay I think that this impl block should be refactored into something different. It is not BadRequest
/// practice to have so many different
impl StrategicAlgorithm {
    #[instrument(level = "trace", skip_all)]
    pub fn schedule_normal_work_orders(&mut self) {
        while !self.priority_queues.normal.is_empty() {
            for period in self.periods.clone() {
                let (work_order_number, weight) = match self.priority_queues.normal.pop() {
                    Some((work_order_number, weight)) => (work_order_number, weight),
                    None => {
                        break;
                    }
                };
                let inf_work_order_number =
                    self.schedule_normal_work_order(work_order_number, &period);

                if let Some(work_order_number) = inf_work_order_number {
                    self.priority_queues.normal.push(work_order_number, weight);
                }
            }
        }
    }

    #[instrument(level = "trace", skip_all)]
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

    #[instrument(level = "trace", skip_all)]
    pub fn schedule_normal_work_order(
        &mut self,
        work_order_key: u32,
        period: &Period,
    ) -> Option<u32> {
        let optimized_work_order = self
            .optimized_work_orders
            .inner
            .get(&work_order_key)
            .unwrap()
            .clone();

        if period != self.get_periods().last().unwrap() {
            for (resource, resource_needed) in optimized_work_order.get_work_load().clone().iter() {
                let resource_capacity: &f64 = self
                    .resources_capacity
                    .inner
                    .get(&resource.clone())
                    .unwrap()
                    .get(&period.clone())
                    .unwrap();

                let resource_loading: &f64 = self
                    .resources_loading
                    .inner
                    .get(&resource.clone())
                    .unwrap()
                    .get(&period.clone())
                    .unwrap();
                if *resource_needed > *resource_capacity - *resource_loading {
                    return Some(work_order_key);
                }

                if optimized_work_order.get_excluded_periods().contains(period) {
                    return Some(work_order_key);
                }
            }
        }
        match self.optimized_work_orders.inner.get_mut(&work_order_key) {
            Some(optimized_work_order) => {
                optimized_work_order.set_scheduled_period(Some(period.clone()));
                self.changed = true;
            }
            None => {
                panic!(
                    "The work order is not found in the optimized work orders. Should have been
                initialized"
                )
            }
        }

        info!(
            "Work order {} from the normal has been scheduled",
            work_order_key
        );
        self.update_loadings(period.clone(), &optimized_work_order);
        None
    }

    #[instrument(level = "trace", skip_all)]
    pub fn schedule_forced_work_order(&mut self, work_order_key: u32) {
        if let Some(work_order_key) = self.is_scheduled(work_order_key) {
            self.unschedule(work_order_key);
        }

        let period_internal = self
            .optimized_work_orders
            .get_locked_in_period(work_order_key);

        info!(
            "Work order {} has been scheduled with unloading point or manual",
            work_order_key
        );

        self.optimized_work_orders
            .set_scheduled_period(work_order_key, period_internal.clone());
        self.changed = true;

        let work_order = self
            .optimized_work_orders
            .inner
            .get(&work_order_key)
            .unwrap()
            .clone();

        self.update_loadings(period_internal, &work_order);
    }

    #[instrument(level = "trace", skip_all)]
    pub fn unschedule_random_work_orders(
        &mut self,
        number_of_work_orders: usize,
        rng: &mut impl rand::Rng,
    ) {
        let optimized_work_orders = self.get_optimized_work_orders();

        let mut filtered_keys: Vec<_> = optimized_work_orders
            .iter()
            .filter(|(&_key, value)| value.get_locked_in_period().is_none())
            .map(|(&key, _)| key)
            .collect();

        filtered_keys.sort();

        let sampled_work_order_keys = filtered_keys
            .choose_multiple(rng, number_of_work_orders)
            .collect::<Vec<_>>()
            .clone();

        for work_order_key in sampled_work_order_keys {
            self.unschedule(*work_order_key);

            self.populate_priority_queues();
        }

        self.changed = true;
    }

    #[instrument(skip_all)]
    pub fn calculate_objective(&mut self) {
        let mut objective = 0;
        for (work_order_key, optimized_work_order) in &self.optimized_work_orders.inner {
            let optimized_period = match &optimized_work_order.scheduled_period {
                Some(optimized_period) => optimized_period.clone(),
                None => {
                    if let Some(last_period) = self.periods.last() {
                        last_period.clone()
                    } else {
                        panic!("There are no periods in the system")
                    }
                }
            };

            let work_order_latest_allowed_finish_period = self
                .optimized_work_orders
                .inner
                .get(work_order_key)
                .unwrap()
                .get_latest_period()
                .clone();

            let period_difference = calculate_period_difference(
                optimized_period,
                work_order_latest_allowed_finish_period,
            );
            let objective_contribution = if period_difference > 0 {
                period_difference
                    * self
                        .optimized_work_orders
                        .inner
                        .get(work_order_key)
                        .unwrap()
                        .get_weight() as i64
            } else {
                0
            };
            objective += objective_contribution;
        }
        self.objective_value = objective as f32;
    }

    #[instrument(skip_all)]
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

    #[instrument(skip_all)]
    fn update_loadings(&mut self, period_input: Period, work_order: &OptimizedWorkOrder) {
        for (resource, periods) in self.resources_loading.inner.iter_mut() {
            for (period, loading) in periods {
                if *period == period_input {
                    *loading += work_order.get_work_load().get(resource).unwrap_or(&0.0);
                }
            }
        }
    }
}

fn calculate_period_difference(scheduled_period: Period, latest_period: Option<Period>) -> i64 {
    let scheduled_period_date = scheduled_period.get_end_date();
    let latest_period_date = match latest_period.clone() {
        Some(period) => period.get_end_date(),
        None => scheduled_period_date,
    };

    let duration = scheduled_period_date.signed_duration_since(latest_period_date);
    let days = duration.num_days();
    days / 7
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{Duration, TimeZone, Utc};
    use rand::{rngs::StdRng, SeedableRng};
    use std::collections::{HashMap, HashSet};

    use crate::agents::strategic_agent::strategic_algorithm::{
        AlgorithmResources, OptimizedWorkOrders, PriorityQueues, StrategicAlgorithm,
    };
    use shared_messages::resources::Resources;

    #[test]
    fn test_schedule_work_order() {
        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
        let end_date = start_date + chrono::Duration::days(13);

        let period = Period::new(1, start_date, end_date);

        let mut optimized_work_orders = OptimizedWorkOrders::new(HashMap::new());

        let optimized_work_order =
            OptimizedWorkOrder::new(None, None, HashSet::new(), None, 1000, HashMap::new());

        optimized_work_orders.insert_optimized_work_order(2200002020, optimized_work_order);

        let mut resource_capacity = HashMap::new();
        let mut resource_loadings = HashMap::new();

        let mut period_hash_map_150 = HashMap::new();
        let mut period_hash_map_0 = HashMap::new();

        period_hash_map_150.insert(period.clone(), 150.0);
        period_hash_map_0.insert(period.clone(), 0.0);

        resource_capacity.insert(Resources::MtnMech, period_hash_map_150.clone());
        resource_capacity.insert(Resources::MtnElec, period_hash_map_150.clone());
        resource_capacity.insert(Resources::Prodtech, period_hash_map_150.clone());

        resource_loadings.insert(Resources::MtnMech, period_hash_map_0.clone());
        resource_loadings.insert(Resources::MtnElec, period_hash_map_0.clone());
        resource_loadings.insert(Resources::Prodtech, period_hash_map_0.clone());
        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            AlgorithmResources::new(resource_capacity),
            AlgorithmResources::new(resource_loadings),
            PriorityQueues::new(),
            optimized_work_orders,
            vec![period.clone()],
            true,
        );

        scheduler_agent_algorithm.schedule_normal_work_order(2200002020, &period);

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

        let mut work_load = HashMap::new();

        work_load.insert(Resources::MtnMech, 100.0);

        let mut optimized_work_orders = OptimizedWorkOrders::new(HashMap::new());

        let optimized_work_order =
            OptimizedWorkOrder::new(None, None, HashSet::new(), None, 1000, work_load);

        optimized_work_orders
            .inner
            .insert(2200002020, optimized_work_order);

        let mut resource_capacity = HashMap::new();
        let mut resource_loadings = HashMap::new();

        let mut period_hash_map_0 = HashMap::new();

        period_hash_map_0.insert(period.clone(), 0.0);

        resource_capacity.insert(Resources::MtnMech, period_hash_map_0.clone());
        resource_loadings.insert(Resources::MtnMech, period_hash_map_0.clone());

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            AlgorithmResources::new(resource_capacity),
            AlgorithmResources::new(resource_loadings),
            PriorityQueues::new(),
            optimized_work_orders,
            vec![period.clone()],
            true,
        );
        scheduler_agent_algorithm.schedule_normal_work_order(2200002020, &period);

        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200002020)
                .unwrap()
                .get_scheduled_period(),
            scheduler_agent_algorithm.get_periods().last().cloned()
        );
    }

    #[test]
    fn test_update_loadings() {
        let mut work_load = HashMap::new();

        work_load.insert(Resources::MtnMech, 20.0);
        work_load.insert(Resources::MtnElec, 40.0);
        work_load.insert(Resources::Prodtech, 60.0);

        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 7, 0, 0).unwrap();
        let end_date = start_date + chrono::Duration::days(13);

        let period = Period::new(1, start_date, end_date);

        let mut resource_capacity = HashMap::new();
        let mut resource_loadings = HashMap::new();

        let mut period_hash_map_150 = HashMap::new();
        let mut period_hash_map_0 = HashMap::new();

        period_hash_map_150.insert(period.clone(), 150.0);
        period_hash_map_0.insert(period.clone(), 0.0);

        resource_capacity.insert(Resources::MtnMech, period_hash_map_150.clone());
        resource_capacity.insert(Resources::MtnElec, period_hash_map_150.clone());
        resource_capacity.insert(Resources::Prodtech, period_hash_map_150.clone());

        resource_loadings.insert(Resources::MtnMech, period_hash_map_0.clone());
        resource_loadings.insert(Resources::MtnElec, period_hash_map_0.clone());
        resource_loadings.insert(Resources::Prodtech, period_hash_map_0.clone());

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            AlgorithmResources::new(resource_capacity),
            AlgorithmResources::new(resource_loadings),
            PriorityQueues::new(),
            OptimizedWorkOrders::new(HashMap::new()),
            vec![],
            true,
        );

        let work_order = OptimizedWorkOrder::new(
            Some(period.clone()),
            Some(period.clone()),
            HashSet::new(),
            None,
            1000,
            work_load,
        );

        scheduler_agent_algorithm.update_loadings(period.clone(), &work_order);

        assert_eq!(
            scheduler_agent_algorithm
                .resources_loading
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period.clone()),
            Some(20.0).as_ref()
        );
        assert_eq!(
            scheduler_agent_algorithm
                .resources_loading
                .inner
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period.clone()),
            Some(40.0).as_ref()
        );
        assert_eq!(
            scheduler_agent_algorithm
                .resources_loading
                .inner
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period.clone()),
            Some(60.0).as_ref()
        );

        assert_eq!(
            scheduler_agent_algorithm
                .resources_loading
                .inner
                .get(&Resources::MtnScaf),
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
        let mut work_load = HashMap::new();

        work_load.insert(Resources::MtnMech, 20.0);
        work_load.insert(Resources::MtnElec, 40.0);
        work_load.insert(Resources::Prodtech, 60.0);

        let start_date = Utc.with_ymd_and_hms(2023, 11, 20, 0, 0, 0).unwrap();
        let end_date = start_date
            + chrono::Duration::days(13)
            + chrono::Duration::hours(23)
            + chrono::Duration::minutes(59)
            + chrono::Duration::seconds(59);
        let period_1 = Period::new(0, start_date, end_date);
        let period_2 = Period::new(0, start_date, end_date) + Duration::weeks(2);
        let period_3 = Period::new(0, start_date, end_date) + Duration::weeks(4);

        let periods: Vec<Period> = vec![period_1.clone(), period_2.clone(), period_3.clone()];
        let resources = vec![Resources::MtnMech, Resources::MtnElec, Resources::Prodtech];
        // Again, this is not completely correct. There is an invariant here that is not being
        // upheld correctly. What should we do about that?
        let mut resource_capacity: HashMap<Resources, HashMap<Period, f64>> = HashMap::new();
        let mut resource_loadings: HashMap<Resources, HashMap<Period, f64>> = HashMap::new();

        for resource in resources.iter() {
            let capacity_map = resource_capacity.entry(resource.clone()).or_default();
            let loading_map = resource_loadings.entry(resource.clone()).or_default();

            for period in periods.iter() {
                capacity_map.insert(period.clone(), 150.0);
                loading_map.insert(period.clone(), 0.0);
            }
        }

        let mut optimized_work_orders = OptimizedWorkOrders::new(HashMap::new());

        let optimized_work_order = OptimizedWorkOrder::new(
            None,
            Some(period_1.clone()),
            HashSet::new(),
            None,
            1000,
            work_load,
        );

        optimized_work_orders
            .inner
            .insert(2200002020, optimized_work_order);

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            AlgorithmResources::new(resource_capacity),
            AlgorithmResources::new(resource_loadings),
            PriorityQueues::new(),
            optimized_work_orders,
            periods,
            true,
        );

        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        scheduler_agent_algorithm.schedule_normal_work_order(2200002020, &period_1);

        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            20.0
        );

        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            40.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            60.0
        );

        scheduler_agent_algorithm.unschedule(2200002020);
        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );

        scheduler_agent_algorithm.schedule_forced_work_order(2200002020);

        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            20.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            40.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            60.0
        );

        scheduler_agent_algorithm
            .optimized_work_orders
            .set_locked_in_period(2200002020, period_2.clone());
        scheduler_agent_algorithm.schedule_forced_work_order(2200002020);

        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );

        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period_2)
                .unwrap(),
            20.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period_2)
                .unwrap(),
            40.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period_2)
                .unwrap(),
            60.0
        );

        scheduler_agent_algorithm.unschedule(2200002020);
        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );

        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::MtnMech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::MtnElec)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
        assert_eq!(
            *scheduler_agent_algorithm
                .get_resources_loadings()
                .inner
                .get(&Resources::Prodtech)
                .unwrap()
                .get(&period_1)
                .unwrap(),
            0.0
        );
    }

    #[test]
    fn test_unschedule_random_work_orders() {
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

        let mut optimized_work_orders = OptimizedWorkOrders::new(HashMap::new());

        let optimized_work_order_1 = OptimizedWorkOrder::new(
            Some(Period::new_from_string("2023-W47-48").unwrap()),
            None,
            HashSet::new(),
            None,
            1000,
            work_load_1,
        );

        let optimized_work_order_2 = OptimizedWorkOrder::new(
            Some(Period::new_from_string("2023-W47-48").unwrap()),
            None,
            HashSet::new(),
            None,
            1000,
            work_load_2,
        );

        let optimized_work_order_3 = OptimizedWorkOrder::new(
            Some(Period::new_from_string("2023-W49-50").unwrap()),
            None,
            HashSet::new(),
            None,
            1000,
            work_load_3,
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

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            AlgorithmResources::default(),
            AlgorithmResources::default(),
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
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200000001)
                .unwrap()
                .scheduled_period,
            Some(Period::new_from_string("2023-W47-48").unwrap())
        );

        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200000002)
                .unwrap()
                .scheduled_period,
            None
        );

        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2200000003)
                .unwrap()
                .scheduled_period,
            None
        );
    }

    #[test]
    fn test_calculate_period_difference() {
        let period_1 = Period::new_from_string("2023-W47-48");
        let period_2 = Period::new_from_string("2023-W49-50");

        let difference = calculate_period_difference(period_1.unwrap(), Some(period_2.unwrap()));

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
        let mut optimized_work_orders = OptimizedWorkOrders::new(HashMap::new());

        let optimized_work_order = OptimizedWorkOrder::new(
            Period::new_from_string("2023-W47-48").ok(),
            None,
            HashSet::new(),
            None,
            1000,
            HashMap::new(),
        );

        optimized_work_orders
            .inner
            .insert(2100000001, optimized_work_order);

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            0.0,
            AlgorithmResources::default(),
            AlgorithmResources::default(),
            PriorityQueues::new(),
            optimized_work_orders,
            vec![Period::new_from_string("2023-W47-48").unwrap()],
            true,
        );

        scheduler_agent_algorithm.unschedule(2100000001);
        assert_eq!(
            scheduler_agent_algorithm
                .optimized_work_orders
                .inner
                .get(&2100000001)
                .unwrap()
                .scheduled_period,
            None
        );
    }

    #[test]
    fn test_period_clone_equality() {
        let period_1 = Period::new_from_string("2023-W47-48").unwrap();
        let period_2 = Period::new_from_string("2023-W47-48").unwrap();

        assert_eq!(period_1, period_2);
        assert_eq!(period_1, period_1.clone());
    }

    impl StrategicAlgorithm {
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
}
