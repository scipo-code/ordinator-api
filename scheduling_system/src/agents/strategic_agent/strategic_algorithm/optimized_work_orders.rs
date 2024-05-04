use colored::*;
use serde::Serialize;
use shared_messages::models::worker_environment::resources::Resources;
use std::collections::hash_map::Entry;
use std::fmt::Write;
use std::{collections::HashMap, collections::HashSet, hash::Hash, hash::Hasher};
use tracing::instrument;

use crate::agents::LoadOperation;
use shared_messages::models::time_environment::period::Period;
use shared_messages::models::work_order::WorkOrder;

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

    pub fn insert_optimized_work_order(
        &mut self,
        work_order_number: u32,
        optimized_work_order: OptimizedWorkOrder,
    ) {
        self.inner.insert(work_order_number, optimized_work_order);
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

#[derive(Debug)]
pub struct OptimizedWorkOrderBuilder {
    pub scheduled_period: Option<Period>,
    pub locked_in_period: Option<Period>,
    pub excluded_periods: HashSet<Period>,
    pub latest_period: Option<Period>,
    pub weight: u32,
    pub work_load: HashMap<Resources, f64>,
}

impl OptimizedWorkOrderBuilder {
    pub fn new() -> Self {
        Self {
            scheduled_period: None,
            locked_in_period: None,
            excluded_periods: HashSet::new(),
            latest_period: None,
            weight: 0,
            work_load: HashMap::new(),
        }
    }

    pub fn build_from_work_order(mut self, work_order: &WorkOrder, periods: &[Period]) -> Self {
        self.scheduled_period = periods.last().cloned();

        self.excluded_periods = work_order.find_excluded_periods(periods);

        self.weight = work_order.work_order_weight();

        self.work_load = work_order.work_load().clone();

        self.latest_period = Some(work_order.order_dates.latest_allowed_finish_period.clone());

        let unloading_point_period = work_order.unloading_point().period.clone();

        if work_order.is_vendor()
            && (unloading_point_period.is_some() || work_order.status_codes().awsc)
        {
            match unloading_point_period {
                Some(unloading_point_period) => {
                    self.locked_in_period = Some(unloading_point_period.clone());
                    self.scheduled_period = Some(unloading_point_period.clone());
                }
                None => {
                    let scheduled_period = periods
                        .iter()
                        .find(|period| {
                            period.contains_date(work_order.order_dates().basic_start_date)
                        })
                        .cloned();

                    match scheduled_period {
                        Some(scheduled_period) => {
                            self.locked_in_period = Some(scheduled_period.clone());
                            self.scheduled_period = Some(scheduled_period.clone());
                        }
                        None => {
                            self.scheduled_period = periods.last().cloned();
                        }
                    }
                }
            }
            return self;
        }

        if work_order.is_vendor() {
            self.locked_in_period = periods.last().cloned();
            self.scheduled_period = periods.last().cloned();
            return self;
        };

        if work_order.status_codes().sch {
            if unloading_point_period.is_some()
                && periods[0..=1].contains(&unloading_point_period.clone().unwrap())
            {
                self.locked_in_period = unloading_point_period.clone();
                self.scheduled_period = unloading_point_period.clone();
            } else {
                let scheduled_period = periods[0..=1]
                    .iter()
                    .find(|period| period.contains_date(work_order.order_dates().basic_start_date));
                match scheduled_period {
                    Some(scheduled_period) => {
                        self.locked_in_period = Some(scheduled_period.clone());
                        self.scheduled_period = Some(scheduled_period.clone());
                    }
                    None => {
                        self.scheduled_period = periods.last().cloned();
                    }
                }
            }
            return self;
        }

        if work_order.status_codes().awsc {
            let scheduled_period = periods
                .iter()
                .find(|period| period.contains_date(work_order.order_dates().basic_start_date));

            match scheduled_period {
                Some(locked_in_period) => {
                    self.locked_in_period = Some(locked_in_period.clone());
                    self.scheduled_period = Some(locked_in_period.clone());
                }
                None => {
                    self.scheduled_period = periods.last().cloned();
                }
            }
            return self;
        }

        if work_order.unloading_point().period.is_some() {
            let locked_in_period = unloading_point_period.clone().unwrap();
            if !periods[0..=1].contains(unloading_point_period.as_ref().unwrap()) {
                self.locked_in_period = Some(locked_in_period.clone());
                self.scheduled_period = Some(locked_in_period.clone());
            }
            return self;
        }

        if work_order.main_work_center.is_fmc() {
            self.locked_in_period = periods.last().cloned();
            self.scheduled_period = periods.last().cloned();
        }
        self.scheduled_period = periods.last().cloned();
        self
    }

    pub fn build(self) -> OptimizedWorkOrder {
        OptimizedWorkOrder {
            scheduled_period: self.scheduled_period,
            locked_in_period: self.locked_in_period,
            excluded_periods: self.excluded_periods,
            latest_period: self.latest_period,
            weight: self.weight,
            work_load: self.work_load,
        }
    }
}

impl OptimizedWorkOrder {
    pub fn builder() -> OptimizedWorkOrderBuilder {
        OptimizedWorkOrderBuilder::new()
    }

    pub fn scheduled_period_mut(&mut self) -> &mut Option<Period> {
        &mut self.scheduled_period
    }

    pub fn excluded_periods(&self) -> &HashSet<Period> {
        &self.excluded_periods
    }

    pub fn set_scheduled_period(&mut self, period: Option<Period>) {
        self.scheduled_period = period;
    }
}

#[derive(Debug, Clone)]
pub struct StrategicResources {
    pub inner: HashMap<Resources, HashMap<Period, f64>>,
}

impl StrategicResources {
    pub fn new(resources: HashMap<Resources, HashMap<Period, f64>>) -> Self {
        Self { inner: resources }
    }

    pub fn update_load(
        &mut self,
        resource: &Resources,
        period: &Period,
        load: f64,
        load_operation: LoadOperation,
    ) {
        let resource_entry = self.inner.entry(resource.clone());
        let periods = match resource_entry {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(HashMap::new()),
        };

        match periods.entry(period.clone()) {
            Entry::Occupied(mut entry) => match load_operation {
                LoadOperation::Add => *entry.get_mut() += load,
                LoadOperation::Sub => *entry.get_mut() -= load,
            },
            Entry::Vacant(entry) => match load_operation {
                LoadOperation::Add => {
                    entry.insert(load);
                }
                LoadOperation::Sub => {
                    entry.insert(load);
                }
            },
        };
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

        let mut sorted_resources: Vec<&Resources> = self.inner.keys().collect();

        sorted_resources
            .sort_by(|resource_a, resource_b| resource_a.to_string().cmp(&resource_b.to_string()));
        for resource in sorted_resources {
            let inner_map = self.inner.get(resource).unwrap();
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
