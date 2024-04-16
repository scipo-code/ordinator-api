use colored::*;
use serde::Serialize;
use shared_messages::resources::Resources;
use std::fmt::Write;
use std::{collections::HashMap, collections::HashSet, hash::Hash, hash::Hasher};
use tracing::instrument;

use crate::models::time_environment::period::Period;

#[derive(Debug, PartialEq, Clone)]
pub struct OptimizedWorkOrders {
    pub inner: HashMap<u32, OptimizedWorkOrder>,
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

impl Hash for OptimizedWorkOrder {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the length of the HashMap to ensure different lengths produce different hashes
        self.scheduled_period.hash(state);
        self.locked_in_period.hash(state);
        for period in &self.excluded_periods {
            period.hash(state);
        }
    }
}

impl OptimizedWorkOrders {
    pub fn new(inner: HashMap<u32, OptimizedWorkOrder>) -> Self {
        Self { inner }
    }

    #[instrument(level = "trace", skip_all)]
    pub fn set_scheduled_period(&mut self, work_order_number: u32, period: Period) {
        let optimized_work_order = match self.inner.get_mut(&work_order_number) {
            Some(optimized_work_order) => optimized_work_order,
            None => panic!(
                "Work order number {} not found in optimized work orders",
                work_order_number
            ),
        };
        optimized_work_order.scheduled_period = Some(period);
    }
    #[instrument(level = "trace", skip_all)]
    pub fn get_locked_in_period(&self, work_order_number: u32) -> Period {
        let option_period = match self.inner.get(&work_order_number) {
            Some(optimized_work_order) => optimized_work_order.locked_in_period.clone(),
            None => panic!(
                "Work order number {} not found in optimized work orders",
                work_order_number
            ),
        };
        match option_period {
            Some(period) => period,
            None => panic!("Work order number {} does not have a locked in period, but it is being called by the optimized_work_orders.schedule_forced_work_order", work_order_number)
        }
    }

    #[instrument(level = "trace", skip_all)]
    pub fn set_locked_in_period(&mut self, work_order_number: u32, period: Period) {
        let optimized_work_order = match self.inner.get_mut(&work_order_number) {
            Some(optimized_work_order) => optimized_work_order,
            None => panic!(
                "Work order number {} not found in optimized work orders",
                work_order_number
            ),
        };
        optimized_work_order.locked_in_period = Some(period);
    }
}

#[derive(Debug, PartialEq, Clone, Default, Serialize)]
pub struct OptimizedWorkOrder {
    pub scheduled_period: Option<Period>,
    pub locked_in_period: Option<Period>,
    pub excluded_periods: HashSet<Period>,
    pub latest_period: Option<Period>,
    pub weight: u32,
    pub work_load: HashMap<Resources, f64>,
}

impl OptimizedWorkOrder {
    pub fn new(
        scheduled_period: Option<Period>,
        locked_in_period: Option<Period>,
        excluded_periods: HashSet<Period>,
        latest_period: Option<Period>,
        weight: u32,
        work_load: HashMap<Resources, f64>,
    ) -> Self {
        Self {
            scheduled_period,
            locked_in_period,
            excluded_periods,
            latest_period,
            weight,
            work_load,
        }
    }

    pub fn get_scheduled_period(&self) -> Option<Period> {
        self.scheduled_period.clone()
    }

    pub fn get_locked_in_period(&self) -> Option<Period> {
        self.locked_in_period.clone()
    }

    pub fn get_excluded_periods(&self) -> &HashSet<Period> {
        &self.excluded_periods
    }

    pub fn get_latest_period(&self) -> Option<Period> {
        self.latest_period.clone()
    }

    pub fn get_work_load(&self) -> &HashMap<Resources, f64> {
        &self.work_load
    }

    pub fn get_weight(&self) -> u32 {
        self.weight
    }

    /// This is a huge no-no! I think that this will lets us violate the invariant that we have
    /// created between scheduled work and the loadings. We should test for this
    pub fn set_scheduled_period(&mut self, period: Option<Period>) {
        self.scheduled_period = period;
    }
}

#[derive(Debug, Clone)]
pub struct AlgorithmResources {
    pub inner: HashMap<Resources, HashMap<Period, f64>>,
}

impl AlgorithmResources {
    pub fn new(resources: HashMap<Resources, HashMap<Period, f64>>) -> Self {
        Self { inner: resources }
    }

    pub fn to_string(&self, number_of_periods: u32) -> String {
        let mut string = String::new();
        let mut periods = self
            .inner
            .values()
            .flat_map(|inner_map| inner_map.keys())
            .collect::<Vec<_>>();
        periods.sort();
        periods.dedup();

        write!(string, "{:<12}", "Resource").ok();
        for (nr_period, period) in periods.iter().enumerate().take(number_of_periods as usize) {
            if nr_period == 0 {
                write!(string, "{:>12}", period.period_string().red()).ok();
            } else if nr_period == 1 || nr_period == 2 {
                write!(string, "{:>12}", period.period_string().green()).ok();
            } else {
                write!(string, "{:>12}", period.period_string()).ok();
            }
        }
        writeln!(string).ok();

        for (resource, inner_map) in self.inner.iter() {
            write!(string, "{:<12}", resource.variant_name()).unwrap();
            for (nr_period, period) in periods.iter().enumerate().take(number_of_periods as usize) {
                let value = inner_map.get(period).unwrap_or(&0.0);
                if nr_period == 0 {
                    write!(string, "{:>12}", value.round().to_string().red()).ok();
                } else if nr_period == 1 || nr_period == 2 {
                    write!(string, "{:>12}", value.round().to_string().green()).ok();
                } else {
                    write!(string, "{:>12}", value.round()).ok();
                }
            }
            writeln!(string).ok();
        }
        string
    }
}
