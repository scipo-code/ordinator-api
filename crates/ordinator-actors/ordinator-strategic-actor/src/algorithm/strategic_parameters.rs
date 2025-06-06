use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::MaterialToPeriod;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::ClusteringWeights;
use ordinator_scheduling_environment::work_order::WorkOrder;
use ordinator_scheduling_environment::work_order::WorkOrderConfigurations;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::WorkOrders;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::StrategicOptions;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use serde::Serialize;

use super::StrategicResources;

#[derive(Debug)]
pub struct StrategicParameters
{
    pub strategic_work_order_parameters: HashMap<WorkOrderNumber, WorkOrderParameter>,
    pub strategic_capacity: StrategicResources,
    pub strategic_clustering: StrategicClustering,
    pub period_locks: HashSet<Period>,

    // TODO #04 #00 #01
    // enum PeriodState {
    //     Previous(Period),
    //     Frozen(Period),
    //     Draft(Period),
    //     Draft2(Period),
    // }
    // Create this and have it change based on the value
    // of the [`SystemClock`].
    pub strategic_periods: Vec<Period>,
    // Should the options be here? Yes they, no they should not
    pub strategic_options: StrategicOptions,
}

// QUESTION
// Should you make a builder for the `Parameters`?
// I believe that this is a good idea, but I am not really sure
impl Parameters for StrategicParameters
{
    type Key = WorkOrderNumber;

    // That change in the asset, was not complete without downsides.
    fn from_source(
        id: &Id,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self>
    {
        let asset = id.2.first().expect("This should never happen");

        let work_orders = &scheduling_environment.work_orders;

        let strategic_periods = &scheduling_environment.time_environment.periods;

        let actor_specifications = scheduling_environment
            .worker_environment
            .actor_specification
            .get(id.asset())
            .unwrap();

        let strategic_options = &actor_specifications.strategic.strategic_options;
        let work_order_configurations = &actor_specifications.work_order_configurations;
        let material_to_period = &actor_specifications.material_to_period;

        // You need to develop this together with Dall!
        // Okay so you should put the
        //
        let filter = work_orders
            .inner
            .iter()
            .filter(|(_, wo)| wo.functional_location().asset == *asset);

        // ISSUE #000
        // This is crucial to fix correctly now
        let strategic_work_order_parameters = filter
            .map(|(won, wo)| {
                Ok((
                    *won,
                    // TODO #000001 [ ] Move time environment configuraion into
                    // SchedulingEnvironment TODO #000002 [ ] Move work order
                    // parameters from `./configuration` to
                    // `./temp_scheduling_environmen_database`
                    // You shoul
                    WorkOrderParameter::builder()
                        // This should be created with a list of work order numbers instead
                        // of the current implementation.
                        .with_scheduling_environment(
                            wo,
                            strategic_periods,
                            work_order_configurations,
                            material_to_period,
                        )?
                        .build(),
                ))
            })
            .collect::<Result<HashMap<WorkOrderNumber, WorkOrderParameter>>>()?;

        let strategic_clustering = StrategicClustering::calculate_clustering_values(
            asset,
            work_orders,
            &actor_specifications
                .work_order_configurations
                .clustering_weights,
        )?;

        // The `SchedulingEnvironment` should not know about the `StrategicResources`
        // This is wrongly implemented and therefore should be changed.
        let strategic_capacity = StrategicResources::from((scheduling_environment, id));

        Ok(Self {
            strategic_work_order_parameters,
            strategic_capacity,
            strategic_clustering,
            period_locks: HashSet::default(),
            strategic_periods: strategic_periods.clone(),
            strategic_options: strategic_options.clone(),
        })
    }

    // TODO [ ]
    // This should be created as a `Builder` I am not sure that the best decision
    // will be here. You should create this in a functional way.
    // ISSUE #000 create-individual-parameters-for-each-actor
    fn create_and_insert_new_parameter(
        &mut self,
        _key: Self::Key,
        _scheduling_environment: MutexGuard<SchedulingEnvironment>,
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
    // This weight is derived from the ['StrategicOptions`]. This means that the code should
    // work better
    pub work_load: HashMap<Resources, Work>,
}

// This should be reformulated in a different way I think. You
// should strive to make something that will enable us to make
// the most of the procious time here.
//
// This has to be formulated together with the
// I believe that we should experiment here with making the `Type state`
// pattern.
// ISSUE #000 introduce-type-state-pattern-to-handle-complex-business-variants
// ISSUE #000 read-learning-domain-driven-design
#[derive(Debug)]
pub struct WorkOrderParameterBuilder
{
    pub locked_in_period: Option<Period>,
    pub excluded_periods: HashSet<Period>,
    pub latest_period: Option<Period>,
    pub weight: Option<u64>,
    // This weight is derived from the ['StrategicOptions`]. This means that the code should
    // work better
    pub work_load: HashMap<Resources, Work>,
}

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
            None => {
                panic!("Work order number {work_order_number:?} not found in StrategicParameters")
            }
        };
        match option_period {
            Some(period) => period,
            None => panic!(
                "Work order number {work_order_number:?} does not have a locked in period, but it is being called by the optimized_work_orders.schedule_forced_work_order",
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
    //
    pub fn with_scheduling_environment(
        mut self,
        work_order: &WorkOrder,
        periods: &[Period],
        work_order_configurations: &WorkOrderConfigurations,
        material_to_period: &MaterialToPeriod,
    ) -> Result<Self>
    {
        // FIX [ ]
        // This is horribly written and very error prone
        // Use a TypeState pattern if you are in doubt.
        self.excluded_periods = work_order.find_excluded_periods(periods, material_to_period);

        self.weight = Some(
            work_order
                .work_order_value(work_order_configurations)
                .with_context(|| {
                    format!("Could not calculate the work_order_value for: {work_order}")
                })?,
        );

        self.work_load = work_order
            .work_order_load()
            .context("Could not determine the work order load")?;

        self.latest_period = Some(work_order.latest_allowed_finish_period(periods).clone());
        // FIX

        // Ideally we should split the work orders by the operations in the code. I
        // think that is the best approach going forward. For now simply take
        // the first element of the code.
        let unloading_point_period = work_order
            .operations
            .0
            .iter()
            .nth(0)
            .unwrap()
            .1
            .unloading_point(periods);

        if work_order.vendor()
            && (unloading_point_period.is_some()
                || work_order.work_order_analytic.user_status_codes.awsc)
        {
            match unloading_point_period {
                Some(unloading_point_period) => {
                    self.locked_in_period = Some(unloading_point_period.clone());
                    self.excluded_periods.remove(unloading_point_period);
                }
                None => {
                    let scheduled_period = periods
                        .iter()
                        .find(|period| {
                            period.contains_date(work_order.work_order_dates.basic_start_date)
                        })
                        .cloned();

                    if let Some(locked_in_period) = scheduled_period {
                        self.locked_in_period = Some(locked_in_period.clone());
                        self.excluded_periods.remove(&locked_in_period);
                    }
                }
            }
            return Ok(self);
        }

        if work_order.vendor() {
            self.locked_in_period = periods.last().cloned();
            self.excluded_periods
                .remove(self.locked_in_period.as_ref().unwrap());
            return Ok(self);
        };

        if work_order.work_order_analytic.user_status_codes.sch {
            if unloading_point_period.is_some()
                && periods[0..=1].contains(unloading_point_period.unwrap())
            {
                self.locked_in_period
                    .clone_from(&unloading_point_period.cloned());
                self.excluded_periods
                    .remove(self.locked_in_period.as_ref().unwrap());
            } else {
                let scheduled_period = periods[0..=1].iter().find(|period| {
                    period.contains_date(work_order.work_order_dates.basic_start_date)
                });

                if let Some(locked_in_period) = scheduled_period {
                    self.locked_in_period = Some(locked_in_period.clone());
                    self.excluded_periods
                        .remove(self.locked_in_period.as_ref().unwrap());
                }
            }
            return Ok(self);
        }

        if work_order.work_order_analytic.user_status_codes.awsc {
            let scheduled_period = periods
                .iter()
                .find(|period| period.contains_date(work_order.work_order_dates.basic_start_date));

            if let Some(locked_in_period) = scheduled_period {
                self.locked_in_period = Some(locked_in_period.clone());
                self.excluded_periods
                    .remove(self.locked_in_period.as_ref().unwrap());
            }
            return Ok(self);
        }

        if work_order
            .operations
            .0
            .iter()
            .nth(0)
            .unwrap()
            .1
            .unloading_point(periods)
            .is_some()
        {
            let locked_in_period = unloading_point_period.unwrap();
            if !periods[0..=1].contains(unloading_point_period.as_ref().unwrap()) {
                self.locked_in_period = Some(locked_in_period.clone());
                self.excluded_periods
                    .remove(self.locked_in_period.as_ref().unwrap());
            }
            return Ok(self);
        }
        Ok(self)
    }

    pub fn build(self) -> WorkOrderParameter
    {
        if let Some(ref locked_in_period) = self.locked_in_period {
            assert!(!self.excluded_periods.contains(locked_in_period));
        }

        WorkOrderParameter {
            locked_in_period: self.locked_in_period,
            excluded_periods: self.excluded_periods,
            latest_period: self
                .latest_period
                .expect("There should always be a latest period on a StrategicWorkOrder"),
            weight: self
                .weight
                .expect("There should always a weight on a StrategicWorkOrder"),
            work_load: self.work_load,
        }
    }
}

impl WorkOrderParameter
{
    pub fn builder() -> WorkOrderParameterBuilder
    {
        // SHould we accept a
        WorkOrderParameterBuilder {
            locked_in_period: None,
            excluded_periods: HashSet::default(),
            latest_period: None,
            weight: None,
            work_load: HashMap::default(),
        }
    }
}

impl StrategicClustering
{
    pub fn calculate_clustering_values(
        asset: &Asset,
        work_orders: &WorkOrders,
        clustering_weights: &ClusteringWeights,
    ) -> Result<Self>
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
        Ok(StrategicClustering {
            inner: clustering_similarity,
        })
    }
}

pub fn create_strategic_parameters(
    _work_orders: &WorkOrders,
    _periods: &[Period],
    _asset: &Asset,
) -> Result<HashMap<WorkOrderNumber, WorkOrderParameter>>
{
    todo!()
}
