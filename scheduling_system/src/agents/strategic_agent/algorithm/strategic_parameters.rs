use anyhow::{bail, Context, Result};
use serde::Serialize;
use shared_types::scheduling_environment::work_order::operation::Work;
use shared_types::scheduling_environment::worker_environment::resources::Resources;
use shared_types::scheduling_environment::{SchedulingEnvironment, WorkOrders};
use shared_types::strategic::StrategicResources;
use shared_types::Asset;
use std::str::FromStr;
use std::sync::MutexGuard;
use std::{collections::HashMap, collections::HashSet, hash::Hash, hash::Hasher};

use shared_types::scheduling_environment::time_environment::period::Period;
use shared_types::scheduling_environment::work_order::{WorkOrder, WorkOrderNumber};

#[derive(Default, Debug, PartialEq, Clone)]
pub struct StrategicParameters {
    pub strategic_work_order_parameters: HashMap<WorkOrderNumber, StrategicParameter>,
    pub strategic_capacity: StrategicResources,
    pub strategic_clustering: StrategicClustering,
    pub period_locks: HashSet<Period>,
    pub strategic_periods: Vec<Period>,
}

pub type ClusteringValue = u64;

#[derive(Default, Debug, PartialEq, Clone)]
pub struct StrategicClustering {
    pub inner: HashMap<(WorkOrderNumber, WorkOrderNumber), ClusteringValue>,
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
pub struct StrategicParameterBuilder(StrategicParameter);

// TODO: Use this for testing the scheduling program
// enum StrategicParameterStates {
//     Scheduled,
//     BasicStart,
//     VendorWithUnloadingPoint,
//     FMCMainWorkCenter,
// }

impl StrategicParameters {
    pub fn new(
        asset: &Asset,
        scheduling_environment: MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self> {
        let mut strategic_clustering = StrategicClustering::default();

        let work_orders = &scheduling_environment.work_orders;
        let strategic_periods = &scheduling_environment.time_environment.strategic_periods;

        strategic_clustering.calculate_clustering_values(asset, &work_orders)?;

        let strategic_capacity = scheduling_environment
            .worker_environment
            .generate_strategic_resources(&strategic_periods);

        let strategic_work_order_parameters =
            create_strategic_parameters(&work_orders, &strategic_periods, &asset).with_context(
                || format!("StrategicParameters for {:#?} could not be created", &asset),
            )?;

        Ok(Self {
            strategic_work_order_parameters,
            strategic_capacity,
            strategic_clustering,
            period_locks: HashSet::default(),
            strategic_periods: scheduling_environment
                .time_environment
                .strategic_periods
                .clone(),
        })
    }

    pub fn insert_strategic_parameter(
        &mut self,
        work_order_number: WorkOrderNumber,
        strategic_parameter: StrategicParameter,
    ) -> Option<StrategicParameter> {
        self.strategic_work_order_parameters
            .insert(work_order_number, strategic_parameter)
    }

    pub fn get_locked_in_period<'a>(
        &'a self,
        work_order_number: &'a WorkOrderNumber,
    ) -> &'a Period {
        let option_period = match self.strategic_work_order_parameters.get(work_order_number) {
            Some(strategic_parameter) => &strategic_parameter.locked_in_period,
            None => panic!(
                "Work order number {:?} not found in StrategicParameters",
                work_order_number
            ),
        };
        match option_period {
            Some(period) => period,
            None => panic!("Work order number {:?} does not have a locked in period, but it is being called by the optimized_work_orders.schedule_forced_work_order", work_order_number)
        }
    }

    pub fn set_locked_in_period(
        &mut self,
        work_order_number: WorkOrderNumber,
        period: Period,
    ) -> Result<()> {
        let optimized_work_order = match self
            .strategic_work_order_parameters
            .get_mut(&work_order_number)
        {
            Some(optimized_work_order) => optimized_work_order,
            None => bail!(
                "Work order number {:?} not found in optimized work orders",
                work_order_number
            ),
        };
        optimized_work_order.locked_in_period = Some(period);
        Ok(())
    }
}

impl StrategicParameterBuilder {
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

        let unloading_point_period = work_order.unloading_point().clone();

        if work_order.is_vendor()
            && (unloading_point_period.is_some()
                || work_order.work_order_analytic.user_status_codes.awsc)
        {
            match unloading_point_period {
                Some(unloading_point_period) => {
                    self.0.locked_in_period = Some(unloading_point_period.clone());
                    self.0.excluded_periods.remove(&unloading_point_period);
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
                        self.0.excluded_periods.remove(&locked_in_period);
                    }
                }
            }
            return self;
        }

        if work_order.is_vendor() {
            self.0.locked_in_period = periods.last().cloned();
            self.0
                .excluded_periods
                .remove(self.0.locked_in_period.as_ref().unwrap());
            return self;
        };

        if work_order.work_order_analytic.user_status_codes.sch {
            if unloading_point_period.is_some()
                && periods[0..=1].contains(&unloading_point_period.clone().unwrap())
            {
                self.0.locked_in_period.clone_from(&unloading_point_period);
                self.0
                    .excluded_periods
                    .remove(self.0.locked_in_period.as_ref().unwrap());
            } else {
                let scheduled_period = periods[0..=1]
                    .iter()
                    .find(|period| period.contains_date(work_order.order_dates().basic_start_date));

                if let Some(locked_in_period) = scheduled_period {
                    self.0.locked_in_period = Some(locked_in_period.clone());
                    self.0
                        .excluded_periods
                        .remove(self.0.locked_in_period.as_ref().unwrap());
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
                self.0
                    .excluded_periods
                    .remove(self.0.locked_in_period.as_ref().unwrap());
            }
            return self;
        }

        if work_order.unloading_point().is_some() {
            let locked_in_period = unloading_point_period.clone().unwrap();
            if !periods[0..=1].contains(unloading_point_period.as_ref().unwrap()) {
                self.0.locked_in_period = Some(locked_in_period.clone());
                self.0
                    .excluded_periods
                    .remove(self.0.locked_in_period.as_ref().unwrap());
            }
            return self;
        }
        self
    }

    pub fn build(self) -> StrategicParameter {
        if let Some(ref locked_in_period) = self.0.locked_in_period {
            assert!(!self.0.excluded_periods.contains(locked_in_period));
        }

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

impl StrategicClustering {
    pub fn calculate_clustering_values(
        &mut self,
        asset: &Asset,
        work_orders: &WorkOrders,
    ) -> Result<()> {
        #[derive(serde::Deserialize, Debug)]
        pub struct ClusteringWeights {
            asset: u64,
            sector: u64,
            system: u64,
            subsystem: u64,
            equipment_tag: u64,
        }

        // Load clustering weights from config
        let clustering_weights: ClusteringWeights = {
            let clustering_config_path = dotenvy::var("CLUSTER_WEIGHTINGS")
                .context("CLUSTER_WEIGHTINGS should be defined in the env")?;
            let clustering_config_contents = std::fs::read_to_string(clustering_config_path)
                .context("Could not read config file")?;
            serde_json::from_str(&clustering_config_contents)?
        };

        let mut clustering_similarity: HashMap<
            (WorkOrderNumber, WorkOrderNumber),
            ClusteringValue,
        > = HashMap::new();

        // Precompute functional locations for all work orders
        let work_orders_data: Vec<_> = work_orders
            .inner
            .iter()
            .filter(|(_, wo)| &wo.functional_location().asset == asset)
            .map(|(number, work_order)| {
                let fl = &work_order.work_order_info.functional_location;
                (
                    number,
                    fl.asset.clone(),
                    fl.sector(),
                    fl.system(),
                    fl.subsystem(),
                    fl.equipment_tag(),
                )
            })
            .collect();

        // Calculate similarity for each pair of work orders
        for i in 0..work_orders_data.len() {
            for j in i..work_orders_data.len() {
                let (wo_num1, asset1, sector1, system1, subsystem1, tag1) = &work_orders_data[i];
                let (wo_num2, asset2, sector2, system2, subsystem2, tag2) = &work_orders_data[j];

                let similarity = {
                    let mut score = 0;
                    if asset1 == asset2 {
                        score += clustering_weights.asset;
                    }
                    if sector1 == sector2 && sector2.is_some() {
                        score += clustering_weights.sector;
                    }
                    if system1 == system2 && system2.is_some() {
                        score += clustering_weights.system;
                    }
                    if subsystem1 == subsystem2 && subsystem2.is_some() {
                        score += clustering_weights.subsystem;
                    }
                    if tag1 == tag2 && tag2.is_some() {
                        score += clustering_weights.equipment_tag;
                    }
                    score
                };

                clustering_similarity.insert((**wo_num1, **wo_num2), similarity);
            }
        }
        self.inner = clustering_similarity;

        Ok(())
    }
}
pub fn create_strategic_parameters(
    work_orders: &WorkOrders,
    periods: &[Period],
    asset: &Asset,
) -> Result<HashMap<WorkOrderNumber, StrategicParameter>> {
    let mut strategic_work_order_parameters = HashMap::new();

    for (work_order_number, work_order) in work_orders
        .inner
        .iter()
        .filter(|(_, wo)| wo.functional_location().asset == *asset)
    {
        let strategic_parameter = StrategicParameterBuilder::new()
            .build_from_work_order(work_order, periods)
            .build();

        strategic_work_order_parameters.insert(*work_order_number, strategic_parameter);
    }
    Ok(strategic_work_order_parameters)
}
