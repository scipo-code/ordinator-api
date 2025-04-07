use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::MutexGuard;

use anyhow::Result;
use anyhow::bail;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrder;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use serde::Serialize;

use super::StrategicResources;
use crate::StrategicOptions;

#[derive(Debug, PartialEq, Clone)]
pub struct StrategicParameters
{
    pub strategic_work_order_parameters: HashMap<WorkOrderNumber, WorkOrderParameter>,
    pub strategic_capacity: StrategicResources,
    pub strategic_clustering: StrategicClustering,
    pub period_locks: HashSet<Period>,
    pub strategic_periods: Vec<Period>,
    pub strategic_options: StrategicOptions,
}

// QUESTION
// Should you make a builder for the `Parameters`?
// I believe that this is a good idea, but I am not really sure
impl Parameters for StrategicParameters
{
    type Key = WorkOrderNumber;
    type Options = StrategicOptions;

    // That change in the asset, was not complete without downsides.
    fn from_source(
        id: &Id,
        options: Self::Options,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self>
    {
        let asset = id.2.first().expect("This should never happen");

        let work_orders = &scheduling_environment.work_orders;

        let strategic_periods = &scheduling_environment.time_environment.strategic_periods;

        let strategic_work_order_parameters = work_orders
            .inner
            .iter()
            .filter(|(won, wo)| wo.functional_location().asset == *asset)
            .map(|(won, wo)| {
                (
                    won,
                    // TODO [ ]
                    // Okay parameters are needed here and you are stuck because you do not know
                    // what to do about it. I think that the best course of action is to make the
                    // whole system work with dependency injection. The primary goal is to
                    // centralize the configurations.
                    // TODO
                    // This name confuses you, the type is really important but you should not be
                    // immediately worried about the name.
                    // You could fix this now, but the configuration policy is much more important.
                    WorkOrderParameter::builder()
                        .with_scheduling_environment(wo, strategic_periods, &options)
                        .build(),
                )
            })
            .collect();

        let strategic_clustering =
            StrategicClustering::calculate_clustering_values(asset, work_orders, options)?;

        let strategic_capacity = scheduling_environment
            .worker_environment
            .generate_strategic_resources(strategic_periods);

        Ok(Self {
            strategic_work_order_parameters,
            strategic_capacity,
            strategic_clustering,
            period_locks: HashSet::default(),
            strategic_periods: strategic_periods.clone(),
            // How should these be defined? The best approach is to create something that
            // will allow us to make something that will scale.
            strategic_options: options,
        })
    }

    // TODO [ ]
    // This should be created as a `Builder` I am not sure that the best decision
    // will be here.
    fn create_and_insert_new_parameter(
        &mut self,
        key: Self::Key,
        scheduling_environment: MutexGuard<SchedulingEnvironment>,
    )
    {
        todo!()
    }
}

pub type ClusteringValue = u64;

#[derive(Debug, PartialEq, Clone)]
pub struct StrategicClustering
{
    pub inner: HashMap<(WorkOrderNumber, WorkOrderNumber), ClusteringValue>,
}

/// WARNING
/// There is a good change that there should be a generic parameter in this
/// type as there are so many different ways that a `StrategicParameter`
/// can be handled.
#[derive(Debug, PartialEq, Clone, Default, Serialize)]
pub struct WorkOrderParameter
{
    pub locked_in_period: Option<Period>,
    pub excluded_periods: HashSet<Period>,
    pub latest_period: Period,
    pub weight: u64,
    pub work_load: HashMap<Resources, Work>,
}

#[derive(Debug)]
pub struct WorkOrderParameterBuilder(WorkOrderParameter);

// TODO: Use this for testing the scheduling program
// enum StrategicParameterStates {
//     Scheduled,
//     BasicStart,
//     VendorWithUnloadingPoint,
//     FMCMainWorkCenter,
// }

impl StrategicParameters
{
    pub fn get_locked_in_period<'a>(&'a self, work_order_number: &'a WorkOrderNumber)
    -> &'a Period
    {
        let option_period = match self.strategic_work_order_parameters.get(work_order_number) {
            Some(strategic_parameter) => &strategic_parameter.locked_in_period,
            None => panic!(
                "Work order number {:?} not found in StrategicParameters",
                work_order_number
            ),
        };
        match option_period {
            Some(period) => period,
            None => panic!(
                "Work order number {:?} does not have a locked in period, but it is being called by the optimized_work_orders.schedule_forced_work_order",
                work_order_number
            ),
        }
    }

    pub fn set_locked_in_period(
        &mut self,
        work_order_number: WorkOrderNumber,
        period: Period,
    ) -> Result<()>
    {
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

impl WorkOrderParameterBuilder
{
    // WARN
    // This builder is crucial for the whole business logic of things. I am not sure
    // what the best approach is for continuing this.
    // TODO [ ]
    // You need a function for each field here, and you have to understand what each
    // of them means.
    // QUESTION
    // _Where should the configs come from?_
    // The higher level Parameters implementation includes the
    // `SchedulingEnvironment` that means that it should be possible to include
    // the `WorkOrderConfigurations` here.
    pub fn with_scheduling_environment(
        &mut self,
        work_order: &WorkOrder,
        periods: &[Period],
        strategic_options: &StrategicOptions,
    ) -> &mut Self
    {
        // FIX [ ]
        // This is horribly written and very error prone
        self.0.excluded_periods =
            work_order.find_excluded_periods(periods, &strategic_options.material_to_period);

        self.0.weight = work_order.work_order_value(&strategic_options.work_order_configurations);

        self.0.work_load = work_order.work_load();

        self.0.latest_period = work_order.latest_allowed_finish_period(periods).clone();
        // FIX

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

    pub fn build(self) -> WorkOrderParameter
    {
        if let Some(ref locked_in_period) = self.0.locked_in_period {
            assert!(!self.0.excluded_periods.contains(locked_in_period));
        }

        WorkOrderParameter {
            locked_in_period: self.0.locked_in_period,
            excluded_periods: self.0.excluded_periods,
            latest_period: self.0.latest_period,
            weight: self.0.weight,
            work_load: self.0.work_load,
        }
    }
}

impl WorkOrderParameter
{
    fn builder() -> WorkOrderParameterBuilder
    {
        WorkOrderParameterBuilder(WorkOrderParameter {
            locked_in_period: todo!(),
            excluded_periods: todo!(),
            latest_period: todo!(),
            weight: todo!(),
            work_load: todo!(),
        })
    }
}

impl StrategicClustering
{
    pub fn calculate_clustering_values(
        asset: &Asset,
        work_orders: &WorkOrders,
        clustering_weights: ClusteringWeights,
    ) -> Result<HashMap<(WorkOrderNumber, WorkOrderNumber), ClusteringValue>>
    {
        let mut clustering_similarity = HashMap::new();
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
        Ok(clustering_similarity)
    }
}

pub fn create_strategic_parameters(
    work_orders: &WorkOrders,
    periods: &[Period],
    asset: &Asset,
) -> Result<HashMap<WorkOrderNumber, WorkOrderParameter>>
{
}
