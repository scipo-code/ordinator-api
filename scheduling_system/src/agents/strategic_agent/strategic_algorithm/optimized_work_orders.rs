use serde::Serialize;
use shared_types::scheduling_environment::work_order::operation::Work;
use shared_types::scheduling_environment::worker_environment::resources::Resources;
use std::str::FromStr;
use std::{collections::HashMap, collections::HashSet, hash::Hash, hash::Hasher};
use tracing::instrument;

use shared_types::scheduling_environment::time_environment::period::Period;
use shared_types::scheduling_environment::work_order::{WorkOrder, WorkOrderNumber};

#[derive(Default, Debug, PartialEq, Clone)]
pub struct StrategicParameters {
    pub inner: HashMap<WorkOrderNumber, StrategicParameter>,
}

impl Hash for StrategicParameters {
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

impl Hash for StrategicParameter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the length of the HashMap to ensure different lengths produce different hashes
        self.locked_in_period.hash(state);
        for period in &self.excluded_periods {
            period.hash(state);
        }
    }
}

impl StrategicParameters {
    pub fn new(inner: HashMap<WorkOrderNumber, StrategicParameter>) -> Self {
        Self { inner }
    }

    pub fn insert_optimized_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
        optimized_work_order: StrategicParameter,
    ) {
        self.inner.insert(work_order_number, optimized_work_order);
    }

    #[instrument(level = "trace", skip_all)]
    pub fn get_locked_in_period<'a>(&'a self, work_order_number: &'a WorkOrderNumber) -> &Period {
        let option_period = match self.inner.get(&work_order_number) {
            Some(strategic_parameter) => &strategic_parameter.locked_in_period,
            None => panic!(
                "Work order number {:?} not found in optimized work orders",
                work_order_number
            ),
        };
        match option_period {
            Some(period) => &period,
            None => panic!("Work order number {:?} does not have a locked in period, but it is being called by the optimized_work_orders.schedule_forced_work_order", work_order_number)
        }
    }

    #[instrument(level = "trace", skip_all)]
    pub fn set_locked_in_period(&mut self, work_order_number: WorkOrderNumber, period: Period) {
        let optimized_work_order = match self.inner.get_mut(&work_order_number) {
            Some(optimized_work_order) => optimized_work_order,
            None => panic!(
                "Work order number {:?} not found in optimized work orders",
                work_order_number
            ),
        };
        optimized_work_order.locked_in_period = Some(period);
    }
}

#[derive(Debug, PartialEq, Clone, Default, Serialize)]
pub struct StrategicParameter {
    pub locked_in_period: Option<Period>,
    pub excluded_periods: HashSet<Period>,
    pub latest_period: Period,
    pub weight: u64,
    pub work_load: HashMap<Resources, Work>,
}

#[derive(Debug)]
pub struct StrategicParametersBuilder(StrategicParameter);

impl StrategicParametersBuilder {
    pub fn new() -> Self {
        Self(StrategicParameter {
            locked_in_period: None,
            excluded_periods: HashSet::new(),
            latest_period: Period::from_str("2024-W01-02").unwrap(),
            weight: 0,
            work_load: HashMap::new(),
        })
    }

    pub fn build_from_work_order(mut self, work_order: &WorkOrder, periods: &[Period]) -> Self {
        self.0.excluded_periods = work_order.find_excluded_periods(periods);

        self.0.weight = work_order.work_order_weight();

        self.0.work_load.clone_from(work_order.work_load());

        self.0.latest_period = work_order
            .work_order_dates
            .latest_allowed_finish_period
            .clone();

        // assert!(
        //     self.latest_period.end_date().date_naive()
        //         >= work_order.work_order_dates.latest_allowed_finish_date
        // );

        let unloading_point_period = work_order.unloading_point().clone();

        if work_order.is_vendor()
            && (unloading_point_period.is_some()
                || work_order.work_order_analytic.user_status_codes.awsc)
        {
            match unloading_point_period {
                Some(unloading_point_period) => {
                    self.0.locked_in_period = Some(unloading_point_period.clone());
                }
                None => {
                    let scheduled_period = periods
                        .iter()
                        .find(|period| {
                            period.contains_date(work_order.order_dates().basic_start_date)
                        })
                        .cloned();

                    if let Some(locked_in_period) = scheduled_period {
                        self.0.locked_in_period = Some(locked_in_period.clone());
                    }
                }
            }
            return self;
        }

        if work_order.is_vendor() {
            self.0.locked_in_period = periods.last().cloned();
            return self;
        };

        if work_order.work_order_analytic.user_status_codes.sch {
            if unloading_point_period.is_some()
                && periods[0..=1].contains(&unloading_point_period.clone().unwrap())
            {
                self.0.locked_in_period.clone_from(&unloading_point_period);
            } else {
                let scheduled_period = periods[0..=1]
                    .iter()
                    .find(|period| period.contains_date(work_order.order_dates().basic_start_date));

                if let Some(locked_in_period) = scheduled_period {
                    self.0.locked_in_period = Some(locked_in_period.clone());
                }
            }
            return self;
        }

        if work_order.work_order_analytic.user_status_codes.awsc {
            let scheduled_period = periods
                .iter()
                .find(|period| period.contains_date(work_order.order_dates().basic_start_date));

            if let Some(locked_in_period) = scheduled_period {
                self.0.locked_in_period = Some(locked_in_period.clone());
            }
            return self;
        }

        if work_order.unloading_point().is_some() {
            let locked_in_period = unloading_point_period.clone().unwrap();
            if !periods[0..=1].contains(unloading_point_period.as_ref().unwrap()) {
                self.0.locked_in_period = Some(locked_in_period.clone());
            }
            return self;
        }
        let period = periods.last().cloned();
        if work_order.main_work_center.is_fmc() {
            self.0.locked_in_period.clone_from(&period);
        }
        self
    }

    pub fn build(self) -> StrategicParameter {
        StrategicParameter {
            locked_in_period: self.0.locked_in_period,
            excluded_periods: self.0.excluded_periods,
            latest_period: self.0.latest_period,
            weight: self.0.weight,
            work_load: self.0.work_load,
        }
    }
}

impl StrategicParameter {
    pub fn excluded_periods(&self) -> &HashSet<Period> {
        &self.excluded_periods
    }
}
