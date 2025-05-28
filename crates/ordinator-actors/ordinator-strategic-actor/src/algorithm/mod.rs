pub mod assert_functions;
pub mod strategic_interface;
pub mod strategic_parameters;
pub mod strategic_resources;
pub mod strategic_solution;

use std::any::type_name;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;
use std::ops::DerefMut;
use std::panic::Location;
use std::sync::Arc;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use anyhow::ensure;
use itertools::Itertools;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::algorithm::LoadOperation;
use ordinator_actor_core::traits::AbLNSUtils;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_actor_core::traits::ObjectiveValueType;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::TacticalInterface;
use ordinator_scheduling_environment::time_environment::TimeEnvironment;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::StrategicOptions;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use priority_queue::PriorityQueue;
use rand::distr::weighted::Weight;
use rand::prelude::SliceRandom;
use rand::seq::IndexedRandom;
use strategic_parameters::StrategicClustering;
use strategic_parameters::StrategicParameters;
use strategic_parameters::WorkOrderParameter;
use strategic_resources::OperationalResource;
use strategic_resources::StrategicResources;
use strategic_solution::StrategicObjectiveValue;
use strategic_solution::StrategicSolution;
use strum::IntoEnumIterator;
use tracing::Level;
use tracing::event;
use tracing::instrument;

use crate::messages::requests::StrategicRequestResource;
use crate::messages::requests::StrategicRequestScheduling;
use crate::messages::responses::StrategicResponseResources;
use crate::messages::responses::StrategicResponseScheduling;

// How would it look like here if you made it generic? impl
// Algorithm<StrategicSolution, StrategicParameters, StrategicAssertions> {
//
// }
// One thing is for sure. If you decide to do this over, you should do it
// correctly. The issue is that you will have so many bugs due to the fact that
// many generic things have to be changed in 4 different places, even though you
// want the same behavior. I think that making the behavior generic is the most
// important thing here.


pub struct StrategicAlgorithm<Ss>(
    pub Algorithm<StrategicSolution, StrategicParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>,
)
where
    StrategicSolution: Solution,
    StrategicParameters: Parameters,
    Ss: SystemSolutions,
    Algorithm<StrategicSolution, StrategicParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>:
        AbLNSUtils;

impl<Ss> Deref for StrategicAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    type Target =
        Algorithm<StrategicSolution, StrategicParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<Ss> DerefMut for StrategicAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<Ss> ActorBasedLargeNeighborhoodSearch for StrategicAlgorithm<Ss>
where
    Algorithm<StrategicSolution, StrategicParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>:
        AbLNSUtils<SolutionType = StrategicSolution>,
    StrategicSolution: Solution,
    StrategicParameters: Parameters,
    Ss: SystemSolutions<Strategic = StrategicSolution>,
{
    type Algorithm =
        Algorithm<StrategicSolution, StrategicParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>;
    type Options = StrategicOptions;

    fn incorporate_shared_state(&mut self) -> Result<bool> {
        let mut work_order_numbers: Vec<ForcedWorkOrder> = vec![];
        dbg!();
        let mut state_change = false;

        // This is the problem. What is the best way around it?
        // We should create a method to update the
        for (work_order_number, strategic_parameter) in
            self.parameters.strategic_work_order_parameters.iter()
        {
            let scheduled_period = self
                .solution
                .strategic_scheduled_work_orders
                .get(work_order_number)
                .with_context(|| {
                    format!(
                        "{work_order_number:?}\nis not found in the StrategicAlgorithm"
                    )
                })?;

            if scheduled_period == &strategic_parameter.locked_in_period {
                continue;
            }

            // One of the most important things to understand is the trade off coming from
            // encapsulation. That is the most difficult thing in all this. I think that
            // there are many possible different trade offs that can be made
            // here but the most important is getting this running and quickly.
            //
            // If this value is some. It means that the Tactical Algorithm has scheduled the
            // work order.
            let tactical_scheduled_period = self
                .loaded_shared_solution
                .tactical_actor_solution()
                .ok()
                .and_then(|solution| solution.tactical_period(work_order_number, &self.parameters.strategic_periods));

                
            if strategic_parameter.locked_in_period.is_some() {
                work_order_numbers.push(ForcedWorkOrder::Locked(*work_order_number));
            } else if let Some(tactical_period) = tactical_scheduled_period {
                work_order_numbers.push(ForcedWorkOrder::FromTactical((
                    *work_order_number,
                    tactical_period.clone(),
                )));
            }
        }

        for (count, forced_work_order_numbers) in work_order_numbers.iter().enumerate() {
            state_change = true;
            dbg!("Iteration: {} of {}", count,  work_order_numbers.len());
            self.schedule_forced_work_order(forced_work_order_numbers)
                .with_context(|| {
                    format!(
                        "{forced_work_order_numbers:#?} could not be force scheduled"
                    )
                })?;
        }

        Ok(state_change)
    }

    fn make_atomic_pointer_swap(&mut self) {
        // Performance enhancements:
        // * COW: #[derive(Clone)] struct SharedSolution<'a> { tactical: Cow<'a,
        //   TacticalSolution>, // other fields... }
        //
        // * Reuse the old SharedSolution, cloning only the fields that are needed. let
        //   shared_solution = Arc::new(SharedSolution { tactical:
        //   self.tactical_solution.clone(), // Copy over other fields without cloning
        //   ..(**old).clone() });
        self.arc_swap_shared_solution.rcu(|old| {
            let mut shared_solution = (**old).clone();
            // TODO [ ]
            // Make a swap here.
            // Should you make a swap here? Yes that is the next piece of code that is
            // crucial for this to work.
            shared_solution.strategic_swap(&self.id, self.solution.clone());
            Arc::new(shared_solution)
        });
    }

    fn calculate_objective_value(
        &mut self,
    ) -> Result<
        ObjectiveValueType<
            <<Self::Algorithm as AbLNSUtils>::SolutionType as Solution>::ObjectiveValue,
        >,
    > {
        let mut strategic_objective_value =
            StrategicObjectiveValue::new(&self.parameters.strategic_options);

        self.determine_urgency(&mut strategic_objective_value)
            .context("could not determine strategic urgency")?;

        self.determine_resource_penalty(&mut strategic_objective_value);

        self.determine_clustering(&mut strategic_objective_value).context("Could not determine StrategicObjective value")?;

        strategic_objective_value.aggregate_objectives();

        // This should not happen. We should always work on self and then
        // substitute out the remaining parts.
        // panic!();
        if strategic_objective_value.objective_value < self.solution.objective_value.objective_value
        {
            event!(Level::INFO, strategic_objective_value_better = ?strategic_objective_value);
            Ok(ObjectiveValueType::Better(strategic_objective_value))
        } else {
            event!(Level::INFO, strategic_objective_value_worse = ?strategic_objective_value);
            Ok(ObjectiveValueType::Worse)
        }
    }

    fn schedule(&mut self) -> Result<()> {
        // WARNING
        // I am not sure that this is the correct place of putting this.
        // What should we change here? I think that the best thing would be to make this
        // as Hmm... you need to think hard about this. You were using the
        // schedule_forced to handle the state updates. That is not the correct
        // approach here. I think that we instead should strive to make a clean
        // schedule function. and then we should make a shared_state_update
        // function. Maybe it is also time to clean up in all this code.
        dbg!();
        while !self.solution_intermediate.is_empty() {
            dbg!();
            for period in self.parameters.strategic_periods.clone() {
                let (work_order_number, weight) = match self.solution_intermediate.pop() {
                    Some((work_order_number, weight)) => {
        dbg!();
                        (work_order_number, weight)
                    },

                    None => {
        dbg!();
                        break;
                    }
                };

                // You are a little overloaded! I think that you should forget about this for
                // now, but remember about it.
                let inf_work_order_number = self
                    .schedule_strategic_work_order(work_order_number, &period)
                    .with_context(|| {
                        format!("{work_order_number:?} could not be scheduled normally")
                    })?;

        dbg!();
                if let Some(work_order_number) = inf_work_order_number {
                    if &period != self.parameters.strategic_periods.last().unwrap() {
                        self.solution_intermediate.push(work_order_number, weight);
                    }
                }
            }
        }
        dbg!();
        Ok(())
    }

    fn unschedule(&mut self) -> Result<()> {
        dbg!();
        let mut rng = rand::rng();
        let strategic_work_orders = &self.solution.strategic_scheduled_work_orders;

        let strategic_parameters = &self.parameters.strategic_work_order_parameters;

        let mut filtered_keys: Vec<_> = strategic_work_orders
            .iter()
            .filter(|(won, _)| {
                strategic_parameters
                    .get(won)
                    .unwrap()
                    .locked_in_period
                    .is_none()
            })
            .map(|(&won, _)| won)
            .collect();

        filtered_keys.sort();

        let sampled_work_order_keys = filtered_keys
            .choose_multiple(
                &mut rng,
                self.parameters
                    .strategic_options
                    .number_of_removed_work_orders,
            )
            .collect::<Vec<_>>()
            .clone();

        // assert!(self.solution.scheduled_periods.values().all(|per| per.is_some()));
        for work_order_number in sampled_work_order_keys {
            self.unschedule_specific_work_order(*work_order_number)
                .with_context(|| format!("Could not unschedule: {work_order_number:?}"))?;

            let weight = self
                .parameters
                .strategic_work_order_parameters
                .get(work_order_number)
                .context("Parameters should always be available")?
                .weight;
            self.solution_intermediate.push(*work_order_number, weight);
        }
        dbg!();
        Ok(())
    }

    fn algorithm_util_methods(&mut self) -> &mut Self::Algorithm {
        &mut self.0
    }
}

impl<Ss> StrategicAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    pub fn update_the_locked_in_period(
        &mut self,
        work_order_number: &WorkOrderNumber,
        locked_in_period: &Period,
    ) -> Result<()> {
        self.solution
            .strategic_scheduled_work_orders
            .insert(*work_order_number, Some(locked_in_period.clone()));

        let strategic_parameter = self
            .parameters
            .strategic_work_order_parameters
            .get_mut(work_order_number)
            .with_context(|| {
                format!(
                    "{:?} not found in {}",
                    work_order_number,
                    std::any::type_name::<StrategicParameters>()
                )
            })?;

        strategic_parameter
            .excluded_periods
            .remove(locked_in_period);
        strategic_parameter.locked_in_period = Some(locked_in_period.clone());
        Ok(())
    }

    // pub fn swap_scheduled_work_orders(&mut self, rng: &mut impl rand::Rng) {
    //         let scheduled_work_orders: Vec<_> = self
    //             .strategic_solution
    //             .strategic_periods
    //             .keys()
    //             .cloned()
    //             .collect();
    //         let randomly_chosen = scheduled_work_orders.choose_multiple(rng,
    // 2).collect::<Vec<_>>();         unsafe {
    //             let scheduled_work_order_1 =
    // self.solution.strategic_periods.get_mut(randomly_chosen[0]).unwrap() as *mut
    // Option<Period>;             let scheduled_work_order_2 =
    // self.solution.strategic_periods.get_mut(randomly_chosen[1]).unwrap() as *mut
    // Option<Period>;             std::ptr::swap(scheduled_work_order_1,
    // scheduled_work_order_2);             // You cannot do this anymore
    // either. What is the best remedy for this?
    // self.update_loadings(randomly_chosen[0],
    // (*scheduled_work_order_1).as_ref().unwrap(), LoadOperation::Sub);
    //             self.update_loadings(randomly_chosen[1],
    // (*scheduled_work_order_2).as_ref().unwrap(), LoadOperation::Sub);
    //             self.update_loadings(randomly_chosen[0],
    // (*scheduled_work_order_1).as_ref().unwrap(), LoadOperation::Add);
    //             self.update_loadings(randomly_chosen[1],
    // (*scheduled_work_order_2).as_ref().unwrap(), LoadOperation::Add);
    //         }
    // }

    fn strategic_capacity_by_resource(
        &self,
        resource: &Resources,
        period: &Period,
    ) -> Result<Work> {
        self.parameters
            .strategic_capacity
            .aggregated_capacity_by_period_and_resource(period, resource)
    }

    fn strategic_loading_by_resource(&self, resource: &Resources, period: &Period) -> Result<Work> {
        self.solution
            .strategic_loadings
            .aggregated_capacity_by_period_and_resource(period, resource)
    }

    #[allow(dead_code)]
    pub fn calculate_utilization(&self) -> Result<Vec<(i32, u64)>> {
        let mut utilization_by_period = Vec::new();

        for period in &self.parameters.strategic_periods {
            let mut intermediate_loading: f64 = 0.0;
            let mut intermediate_capacity: f64 = 0.0;
            for resource in Resources::iter() {
                let loading = self.strategic_loading_by_resource(&resource, period)?;
                let capacity = self.strategic_capacity_by_resource(&resource, period)?;

                intermediate_loading += loading.to_f64();
                intermediate_capacity += capacity.to_f64();
            }
            let percentage_loading =
                ((intermediate_loading / intermediate_capacity) * 100.0) as u64;
            utilization_by_period.push((*period.id(), percentage_loading));
        }
        Ok(utilization_by_period)
    }

    fn determine_urgency(
        &mut self,
        strategic_objective_value: &mut StrategicObjectiveValue,
    ) -> Result<()> {
        for (work_order_number, scheduled_period) in &self.solution.strategic_scheduled_work_orders
        {
            let optimized_period = match scheduled_period {
                Some(optimized_period) => optimized_period,
                None => self
                    .parameters
                    .strategic_periods
                    .last()
                    .context("There should always be a last .parameters.eriod")?,
            };

            let work_order_latest_allowed_finish_period = &self
                .parameters
                .strategic_work_order_parameters
                .get(work_order_number)
                .expect("StrategicParameter should always be available for the StrategicSolution")
                .latest_period;

            let non_zero_period_difference = calculate_period_difference(
                optimized_period,
                work_order_latest_allowed_finish_period,
            );

            let work_order_value = self
                    .parameters
                    .strategic_work_order_parameters
                    .get(work_order_number)
                    .unwrap()
                    .weight;

            let period_penalty = non_zero_period_difference
                * work_order_value;

            strategic_objective_value.urgency.1.checked_add_assign(&period_penalty)
                .ok()
                .with_context(|| format!("Overflow on the strategic urgency.\nperiod penalty: {period_penalty}\nperiod difference: {non_zero_period_difference}\nwork_order_value: {work_order_value}"))?;
        }
        Ok(())
    }

    fn determine_clustering(&mut self, strategic_objective_value: &mut StrategicObjectiveValue) -> anyhow::Result<()> {
        for period in &self.parameters.strategic_periods {
            // Precompute scheduled work orders for the current period
            let scheduled_work_orders_by_period: Vec<_> = self
                .solution
                .strategic_scheduled_work_orders
                .iter()
                .filter_map(|(won, opt_per)| {
                    if let Some(per) = opt_per {
                        if per == period {
                            return Some(won);
                        }
                    }
                    None
                })
                .collect();

            // Cache references to clustering inner map
            let clustering_inner = &self.parameters.strategic_clustering.inner;

            for i in 0..scheduled_work_orders_by_period.len() {
                for j in (i + 1)..scheduled_work_orders_by_period.len() {
                    // Retrieve clustering value, handling symmetry
                    let work_order_pair = (
                        *scheduled_work_orders_by_period[i],
                        *scheduled_work_orders_by_period[j],
                    );
                    let reverse_pair = (
                        *scheduled_work_orders_by_period[j],
                        *scheduled_work_orders_by_period[i],
                    );

                    let clustering_value_for_work_order_pair = clustering_inner
                        .get(&work_order_pair)
                        .or_else(|| clustering_inner.get(&reverse_pair))
                        .with_context(|| {
                            format!(
                                "Missing: {} between {:?} and {:?}",
                                std::any::type_name::<StrategicClustering>(),
                                scheduled_work_orders_by_period[i],
                                scheduled_work_orders_by_period[j]
                            )
                        })
                        .context("clustering_value not available. Did you disable it to increase startup times? That is the most likely scenario")?;

                    // Increment the clustering value in the objective
                    strategic_objective_value.clustering_value.1 +=
                        *clustering_value_for_work_order_pair;
                }
            }
        }
        Ok(())
    }

    // The resource penalty should simply be calculated on the total amount of
    // exceeded hours. I do not see a different way of coding it. It could work
    // on the skill_hours, but that would be needlessly complex in this setting.
    fn determine_resource_penalty(
        &mut self,
        strategic_objective_value: &mut StrategicObjectiveValue,
    ) {
        for (resource, periods) in &self.parameters.strategic_capacity.0 {
            let capacity: f64 = periods.iter().map(|ele| ele.1.total_hours.to_f64()).sum();
            let loading: f64 = self
                .solution
                .strategic_loadings
                .0
                .get(resource)
                .unwrap()
                .iter()
                .map(|ele| ele.1.total_hours.to_f64())
                .sum();

            if loading - capacity > 0.0 {
                strategic_objective_value.resource_penalty.1 += (loading - capacity) as u64
            }
        }
    }
}

// This should be in a different place as well. I think that the best approach
// will be to consolidate this together with the other models.
#[derive(Debug)]
pub enum ForcedWorkOrder {
    Locked(WorkOrderNumber),
    FromTactical((WorkOrderNumber, Period)),
}

impl ForcedWorkOrder {
    pub fn work_order_number(&self) -> &WorkOrderNumber {
        match self {
            ForcedWorkOrder::Locked(work_order_number) => work_order_number,
            ForcedWorkOrder::FromTactical((work_order_number, _)) => work_order_number,
        }
    }
}

#[derive(Debug)]
pub enum ScheduleWorkOrder {
    Normal,
    Forced,
    Unschedule,
}

// You cannot define it like this. It is horrible for
// There has to be changed something in here as well.
// TODO [ ]
// This function should be a part of the `TacticalSolution` interface.
// That is the main point for these types of things. The interface is
// what defines the "Metavariables" from the paper and this is what
// should we.
pub trait StrategicUtils {
    fn schedule_strategic_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
        period: &Period,
    ) -> Result<Option<WorkOrderNumber>>;

    fn schedule_forced_work_order(
        &mut self,
        force_schedule_work_order: &ForcedWorkOrder,
    ) -> Result<()>;

    fn is_scheduled(&self, work_order_number: &WorkOrderNumber) -> bool;

    /// This function updates the StrategicResources based on the a provided
    /// loading.
    fn update_loadings(
        &mut self,
        strategic_resources: StrategicResources,
        load_operation: LoadOperation,
    );

    fn determine_best_permutation(
        &self,
        work_load: HashMap<Resources, Work>,
        period: &Period,
        schedule: ScheduleWorkOrder,
    ) -> Result<Option<StrategicResources>>;
}
// This should be exchanged by a binary heap. This is an issue as well. You do
// not need to have all this code in the
impl<Ss> StrategicUtils for StrategicAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    // TODO [ ]
    // This should be changed as well. But now I will go home. I think that your
    // best approach is to make something that will allow us to implement this
    // and create something for our fellow man. You should remove this function
    // from the code. And you should instead rely on the interface. I think that
    // the interface is the most important thing here. This function is simply
    // about taking the first day of a specific tactical work order and determine
    // its associated period. Nothing else is required for this to function.

    fn schedule_strategic_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
        period: &Period,
    ) -> Result<Option<WorkOrderNumber>> {
        let strategic_parameter = self
            .parameters
            .strategic_work_order_parameters
            .get(&work_order_number)
            .unwrap()
            .clone();

        let work_load = &self
            .parameters
            .strategic_work_order_parameters
            .get(&work_order_number)
            .unwrap()
            .work_load;

        // WARN
        // The issue is that it always returns here and it should not be doing
        // that. If it is the last period it should go through but not be added
        // to the priority queue.
        dbg!();
        if strategic_parameter.excluded_periods.contains(period) {
            return Ok(Some(work_order_number));
        }

        dbg!();
        if self.parameters.period_locks.contains(period) {
            return Ok(Some(work_order_number));
        }

        dbg!();
        let resource_use_option = self
            .determine_best_permutation(work_load.clone(), period, ScheduleWorkOrder::Normal)
            .with_context(|| {
                format!(
                    "{:?}\nfor period\n{:#?}\ncould not be {:?}",
                    work_order_number,
                    period,
                    ScheduleWorkOrder::Normal
                )
            })?;

        let resource_use = match resource_use_option {
            Some(resource_use_inner) => resource_use_inner,
            None => return Ok(Some(work_order_number)),
        };

        dbg!();
        let previous_period = self
            .solution
            .strategic_scheduled_work_orders
            .insert(work_order_number, Some(period.clone()));

        dbg!();
        ensure!(
            previous_period.as_ref().unwrap().is_none(),
            "Previous period: {:#?}\nNew period: {:#?}\nStrategicParameter: {:#?}\nfile: {}\nline: {}",
            &previous_period,
            period,
            strategic_parameter,
            file!(),
            line!()
        );

        resource_use.assert_well_shaped_resources()?;

        dbg!();
        self.update_loadings(resource_use, LoadOperation::Add);
        dbg!();
        Ok(None)
    }

    // Where should this code go now? I think that it should go into the
    fn schedule_forced_work_order(
        &mut self,
        force_schedule_work_order: &ForcedWorkOrder,
    ) -> Result<()> {
        if self.is_scheduled(force_schedule_work_order.work_order_number()) {
            self.unschedule_specific_work_order(*force_schedule_work_order.work_order_number())
                .with_context(|| {
                    format!(
                        "{:#?}\nfile: {}\nline: {}",
                        force_schedule_work_order,
                        file!(),
                        line!()
                    )
                })?;
        }
        dbg!();

        let locked_in_period = match &force_schedule_work_order {
            ForcedWorkOrder::Locked(work_order_number) => self
                .parameters
                .get_locked_in_period(work_order_number)
                .clone(),
            ForcedWorkOrder::FromTactical((_, period)) => period.clone(),
        };

        dbg!();
        // Should the update loadings also be included here? I do not think that is a
        // good idea. What other things could we do?
        self.update_the_locked_in_period(
            force_schedule_work_order.work_order_number(),
            &locked_in_period.clone(),
        )
        .with_context(|| {
            format!(
                "Could not fully update {:#?} in {}",
                force_schedule_work_order, &locked_in_period
            )
        })?;

        dbg!();
        let work_load = self
            .parameters
            .strategic_work_order_parameters
            .get(force_schedule_work_order.work_order_number())
            .unwrap()
            .work_load
            .clone();

        dbg!();
        let strategic_resources = self
            .determine_best_permutation(work_load, &locked_in_period, ScheduleWorkOrder::Forced)
            .with_context(|| format!("{:?}\ncould not be\n{:#?}", force_schedule_work_order, ScheduleWorkOrder::Forced))?
            .expect("It should always be possible to determine a resource permutation for a forced work order");

        dbg!();
        strategic_resources.assert_well_shaped_resources()?;

        dbg!();
        self.update_loadings(strategic_resources, LoadOperation::Add);
        dbg!();
        Ok(())
    }

    fn is_scheduled(&self, work_order_number: &WorkOrderNumber) -> bool {
        self.solution
            .strategic_scheduled_work_orders
            .get(work_order_number)
            .expect("This should always be initialized")
            .is_some()
    }

    /// This function updates the StrategicResources based on the a provided
    /// loading.
    fn update_loadings(
        &mut self,
        strategic_resources: StrategicResources,
        load_operation: LoadOperation,
    ) {
        // How should the change be handled in this function? The most important thing
        // here is to make the function work correctly on the new resource type.
        // This will be difficult as we cannot make the FIX This loading
        // function is not correct. How should we change it so that it will be able to
        // function correctly without calling the permutation loop? We cannot,
        // it would not make sense for this kind of function. I believe that the best
        // decision here is to make a function that lets us manually update and change a
        // value.
        for (period, operational_resources) in strategic_resources.0 {
            for (operational_id, loading) in operational_resources {
                match load_operation {
                    LoadOperation::Add => {
                        let strategic_loading = self
                            .solution
                            .strategic_loadings
                            .0
                            .entry(period.clone())
                            .or_default()
                            .entry(operational_id.clone())
                            .or_default();

                        strategic_loading.total_hours += loading.total_hours;
                        for (skill, hours) in loading.skill_hours {
                            strategic_loading
                                .skill_hours
                                .entry(skill)
                                .and_modify(|wor| *wor += hours)
                                .or_insert(hours);
                        }
                    }
                    LoadOperation::Sub => {
                        let strategic_loading = self
                            .solution
                            .strategic_loadings
                            .0
                            .entry(period.clone())
                            .or_default()
                            .entry(operational_id.clone())
                            .or_default();

                        strategic_loading.total_hours -= loading.total_hours;
                        for (skill, hours) in loading.skill_hours {
                            strategic_loading
                                .skill_hours
                                .entry(skill)
                                .and_modify(|wor| *wor -= hours)
                                .or_insert(hours);
                        }
                    }
                }
            }
        }
    }

    /// This was such a stupid direction to take the code in.
    /// This function is created to find the best permutation of all the work
    /// order assignments to all technicians. This function has two
    /// purposes:
    /// * Determine if there is a feasible permutaion
    /// * If true, return the loading that should be put into the
    ///   StrategicSolution::strategic_loadings.
    fn determine_best_permutation(
        &self,
        work_load: HashMap<Resources, Work>,
        period: &Period,
        schedule: ScheduleWorkOrder,
    ) -> Result<Option<StrategicResources>> {
        ensure!(work_load.iter().all(|e| e.1 >= &Work::from(0.0)), "");
        let mut rng = rand::rng();
        let mut best_total_excess = Work::from(-999999999.0);
        let mut best_work_order_resource_loadings = StrategicResources::default();

        let strategic_capacity_resources = self
            .parameters
            .strategic_capacity
            .0
            .get(period)
            .context("There should always be a dummy resource that can soak excess")?;

        if matches!(schedule, ScheduleWorkOrder::Normal)
            && !work_load.keys().collect::<HashSet<&Resources>>().is_subset(
                &strategic_capacity_resources
                    .values()
                    .flat_map(|ele| ele.skill_hours.keys())
                    .collect::<HashSet<&Resources>>(),
            )
        {
            return Ok(None);
        }

        let strategic_loading_resources: HashMap<String, OperationalResource> = self
            .solution
            .strategic_loadings
            .0
            .get(period)
            .cloned()
            .unwrap_or_else(HashMap::new);

        // This is the difference between the capacity and the loading
        // * val < 0: loading is higher than capacity
        // * val > 0: capacity is higher than loading
        let difference_resources =
            determine_difference_resources(strategic_capacity_resources, &strategic_loading_resources);

        // Perform 10 different technician permutations
        let mut error_for_unschedule = vec![];
        let mut store_strategic_resources_options = vec![];
        for _ in 0..10 {
            let mut technician_permutation = difference_resources
                .clone()
                .into_values()
                .collect::<Vec<_>>();
            technician_permutation.shuffle(&mut rng);

            // Perform 10 different work_load permutations
            for _ in 0..10 {
                let mut work_load_permutation = work_load.clone().into_iter().collect::<Vec<_>>();
                work_load_permutation.shuffle(&mut rng);

                error_for_unschedule.push(work_load_permutation.clone());
                let strategic_resource_loadings_option = match schedule {
                    ScheduleWorkOrder::Normal => determine_normal_work_order_resource_loadings(
                        period,
                        &mut technician_permutation,
                        &mut work_load_permutation,
                    ).with_context(|| "An error occured while calculating the normal scheduling loading".to_string())?,
                    ScheduleWorkOrder::Forced => {
                        let strategic_resource_loadings_option =
                            determine_forced_work_order_resource_loadings(
                                period,
                                &mut best_total_excess,
                                &mut best_work_order_resource_loadings,
                                &mut technician_permutation,
                                &mut work_load_permutation,
                            ).context("incorrectly determine forced work order loadings\n{period}")?;

                        assert_work_load_equal_to_strategic_resource(
                            period,
                            &best_work_order_resource_loadings,
                            &work_load,
                            LoadOperation::Add,
                        )
                        .with_context(|| format!("file: {}\nline: {}", file!(), line!()))?;

                        if let Some(ref strategic_resource_loading) =
                            strategic_resource_loadings_option
                        {
                            assert_work_load_equal_to_strategic_resource(
                                period,
                                strategic_resource_loading,
                                &work_load,
                                LoadOperation::Add,
                            )
                            .with_context(|| format!("file: {}\nline: {}", file!(), line!()))?;
                        }

                        strategic_resource_loadings_option
                    }
                    ScheduleWorkOrder::Unschedule => {
                        let order_map: HashMap<_, _> = technician_permutation
                            .clone()
                            .into_iter()
                            .enumerate()
                            .map(|(i, v)| (v.id, i))
                            .collect();

                        let strategic_resources_loading_vec: Vec<_> = strategic_loading_resources
                            .clone()
                            .into_iter()
                            .sorted_by_key(|ele| order_map.get(&ele.0))
                            .map(|ele| ele.1)
                            .collect();

                        let strategic_resources_option =
                            determine_unschedule_work_resource_loadings(
                                period,
                                &strategic_resources_loading_vec,
                                &mut work_load_permutation,
                            ).with_context(||format!("Error in resource calculation\n{period:#?}\nStrategic loadings: {strategic_loading_resources:#?}\nStrategic capacities: {strategic_capacity_resources:#?}\nWork load: {work_load:?}"))?;
                        store_strategic_resources_options.push(strategic_resources_option.clone());

                        if let Some(ref strategic_resources) = strategic_resources_option {
                            assert_work_load_equal_to_strategic_resource(
                                period,
                                strategic_resources,
                                &work_load,
                                LoadOperation::Sub,
                            )
                            .with_context(|| {
                                format!(
                                    "strategic_resources_loading: {:#?}\nfile: {}\nline: {}",
                                    &strategic_loading_resources,
                                    file!(),
                                    line!()
                                )
                            })?;
                        }
                        strategic_resources_option
                    }
                };

                // If the work_order_resource_loadings is none it means
                // that the code was not able completely satisfy the
                // resource_constraint. This means that we should try to work on
                // a new work_load_permutation. And remember that this one will
                // be replenished
                // There are many different cases here. We should work on the one associated
                // with the Unschedule which means that if the unscheduling
                // operations is not possible we should try to make something.
                let strategic_resource_loadings = match strategic_resource_loadings_option {
                    Some(strategic_resource_loadings) => strategic_resource_loadings,
                    None => continue,
                };

                match schedule {
                    ScheduleWorkOrder::Normal => {
                        if work_load_permutation
                            .into_iter()
                            .all(|(_, wor)| wor == Work::from(0.0))
                        {
                            // If the work order assignment is feasible we put in the
                            return Ok(Some(strategic_resource_loadings));
                        }
                    }
                    ScheduleWorkOrder::Forced => {
                        let equal_resources =
                            // Example:
                            // * let a = work_load.keys(): MTN_MECH
                            // * let b = best_work_order_resource_loadings.<LOGIC FROM BELOW>: {MTN_MECH, MTN_ELEC, PRODTECH}
                            // * a.is_subset(&b): true
                            work_load
                                .keys()
                                .collect::<HashSet<&Resources>>() 
                                .is_subset(
                                    &best_work_order_resource_loadings
                                        .0
                                        .get(period)
                                        .with_context(|| format!("{:#?}\nnot found in\n{}\n{}\n{}", period, std::any::type_name::<StrategicResources>(), file!(), line!()))?
                                        .values()
                                        .flat_map(|ele| ele.skill_hours.keys())
                                        .collect::<HashSet<&Resources>>()
                                    );

                        ensure!(
                            equal_resources,
                            format!(
                                "{:#?}\n{:#?}\nfile: {}\nline: {}",
                                best_work_order_resource_loadings,
                                work_load,
                                file!(),
                                line!()
                            )
                        );
                        return Ok(Some(best_work_order_resource_loadings));
                    }
                    // It is always possible to unschedule work, so therefore we can simply
                    ScheduleWorkOrder::Unschedule => {
                        assert_work_load_equal_to_strategic_resource(
                            period,
                            &strategic_resource_loadings,
                            &work_load,
                            LoadOperation::Sub,
                        )
                        .with_context(|| format!("Location: {}", Location::caller()))?;

                        if work_load_permutation
                            .iter()
                            .all(|res_wor| res_wor.1 == Work::from(0.0))
                        {
                            return Ok(Some(strategic_resource_loadings));
                        }
                    }
                }
            }
        }
        match schedule {
            ScheduleWorkOrder::Normal => Ok(None),
            ScheduleWorkOrder::Forced => Ok(Some(best_work_order_resource_loadings)),
            ScheduleWorkOrder::Unschedule => {
                // NOTE
                // This error has a serious issue. The thing is that the Error 
                    let combined_work = work_load.iter().map(|e| (e.0 ,strategic_loading_resources.iter()
                        .filter(|res| res.1.skill_hours.contains_key(e.0))
                        .fold(Work::from(0.0),|acc, res_fil| {
                        
                            acc + res_fil.1.total_hours
                    })));
                bail!("Unscheduling work order should always be possible\n{period:#?}\nStrategic loadings: {:#?}\nStrategic capacities: {:#?}\nWork load: {:?}\nTotal available resources for each trait: {:#?}\nUnsuccessful work_load_permutations: {:#?}",

                    strategic_loading_resources, strategic_capacity_resources, work_load,
                    combined_work,
                error_for_unschedule)
                
            }
        }
    }
}

fn assert_work_load_equal_to_strategic_resource(
    period: &Period,
    strategic_resource_loadings: &StrategicResources,
    work_load: &HashMap<Resources, Work>,
    load_operation: LoadOperation,
) -> Result<()> {
    let aggregate_strategic_resource = strategic_resource_loadings
            .0
            .get(period)
            .with_context(|| format!("{:#?}\nnot present. This probably means that nothing was {:#?}\nfile: {}\nline: {}", period, ScheduleWorkOrder::Unschedule, file!(), line!()))?
            .iter()
            .fold(Work::from(0.0), |acc, or| acc + or.1.total_hours);

    let aggregate_work_load =
        work_load
            .values()
            .fold(Work::from(0.0), |acc, wor| match load_operation {
                LoadOperation::Add => acc + *wor,
                LoadOperation::Sub => acc - *wor,
            });

    let value = aggregate_work_load.equal(aggregate_strategic_resource);

    ensure!(
        value,
        format!(
            "Aggregate Work:\nStrategicResources: {:#?}\nwork_load: {:#?}\n\n{:#?} {:#?}\nfile: {}\nline: {}",
            aggregate_strategic_resource,
            aggregate_work_load,
            work_load,
            strategic_resource_loadings,
            file!(),
            line!()
        )
    );
    Ok(())
}

fn determine_unschedule_work_resource_loadings(
    period: &Period,
    loading_resources: &[OperationalResource],
    work_load_permutation: &mut [(Resources, Work)],
) -> Result<Option<StrategicResources>> {
    let mut strategic_resources = StrategicResources::default();
    let mut loading_resources_cloned = loading_resources.to_vec();
    for (resources, work) in work_load_permutation.iter_mut() {
        debug_assert!(
            loading_resources_cloned
                .iter()
                .flat_map(|or| or.skill_hours.keys())
                .collect::<HashSet<_>>()
                .contains(resources)
        );

        // Here we do not want to go over the operational_resource in random order. We
        // want to determine the best possible way to visit the structure to
        // make a correct scheduling approach.
        //
        // You should use the `Result::Err` to jump out of the function with the required information
        // You are not doing this correctly. You have to test the functions.
        for operational_resource in loading_resources_cloned.iter_mut() {

            if !operational_resource.skill_hours.contains_key(resources) {
                continue;
            }
            ensure!(operational_resource.skill_hours.iter().all(|e| e.1 == &operational_resource.total_hours),
                "Each skill_hours should always be equal to the value of the total_hours\n{}", std::panic::Location::caller());

            if operational_resource.total_hours >= *work {
                operational_resource.total_hours -= *work;
                operational_resource
                    .skill_hours
                    .iter_mut()
                    .for_each(|ele| *ele.1 -= *work);

                strategic_resources.update_load(
                    period,
                    *resources,
                    *work,
                    operational_resource,
                    LoadOperation::Sub,
                );

                *work = Work::from(0.0);
            ensure!(operational_resource.skill_hours.iter().all(|e| e.1 == &operational_resource.total_hours),
                "Each skill_hours should always be equal to the value of the total_hours\noperational_resource: {:#?}\n{}", operational_resource, std::panic::Location::caller());

                break;
            } else {
                *work -= operational_resource.total_hours;
                strategic_resources.update_load(
                    period,
                    *resources,
                    operational_resource.total_hours,
                    operational_resource,
                    LoadOperation::Sub,
                );

                operational_resource.total_hours = Work::from(0.0);
                operational_resource
                    .skill_hours
                    .iter_mut()
                    .for_each(|ele| *ele.1 = Work::from(0.0));

            ensure!(operational_resource.skill_hours.iter().all(|e| e.1 == &operational_resource.total_hours),
                "Each skill_hours should always be equal to the value of the total_hours\noperational_resource: {:#?}\n{}", operational_resource, std::panic::Location::caller());
                continue;
            }
        }
    }

    // So it returns an `Option::None` if it was not possible to determine this correctly.
    if work_load_permutation
        .iter()
        .all(|ele| ele.1 == Work::from(0.0))
    {
        Ok(Some(strategic_resources))
    } else {
        Ok(None)
    }
    // If all work_load_permutation are Work::from(0.0) we can simply return the
    // value.
}

/// This function determines the resource load for when a work order should be
/// forced into the schedule.
fn determine_forced_work_order_resource_loadings(
    period: &Period,
    best_total_excess: &mut Work,
    best_work_order_resource_loadings: &mut StrategicResources,
    technician_permutation: &mut [OperationalResource],
    work_load_permutation: &mut [(Resources, Work)],
) -> Result<Option<StrategicResources>> {
    let mut work_order_resource_loadings = StrategicResources::default();
    // FIX [ ]
    // There may need to be a `Vec` to handle the case of multiple Technicians.
    for (resources, work) in work_load_permutation.iter_mut() {
        let mut qualified_technicians = technician_permutation
            .iter_mut()
            .filter(|or| or.skill_hours.keys().contains(resources))
            .collect::<Vec<_>>();

        // If there are no qualified technicians. All technicians should be assumed to
        // be responsible? Yes let us just do that!
        //
        // You are a pathetic idiot! You are risking everything because you
        // do not know... 
        // 
        let mut resources_store_dummy: Option<OperationalResource> = None;
        if qualified_technicians.is_empty() {
            let operational_resource = OperationalResource::new(&(resources.to_string() + "_dummy"), Work::from(0.0), vec![*resources]);
            resources_store_dummy = Some(operational_resource);
            qualified_technicians.push(resources_store_dummy.as_mut().unwrap());
        };


        let work_load_by_resources_by_technician =
            work.divide_work(Work::from(qualified_technicians.len() as f64));

        for operational_resource in qualified_technicians {
            ensure!(operational_resource.skill_hours.iter().all(|e| e.1 == &operational_resource.total_hours),
                "Each skill_hours should always be equal to the value of the total_hours\n{}", std::panic::Location::caller());
            operational_resource.total_hours -= work_load_by_resources_by_technician;
            operational_resource
                .skill_hours
                .iter_mut()
                .for_each(|(_, wor)| *wor -= work_load_by_resources_by_technician);
            // So here you are inserting the work_load for that particular skill. You have to instead make a dummy
            // here. I do not see what other options that we have?
            //
            // You cannot give vendor work to the 
            work_order_resource_loadings.update_load(

                period,
                *resources,
                work_load_by_resources_by_technician,
                operational_resource,
                LoadOperation::Add,
            );
        }
    }
    let total_excess = technician_permutation
        .iter()
        .map(|or| std::cmp::min(Work::from(0.0), or.total_hours))
        .reduce(|acc, wor| acc + wor)
        .expect("The Technician Permutation here should never be empty");

    if total_excess == Work::from(0.0) {
        *best_work_order_resource_loadings = work_order_resource_loadings.clone();
        return Ok(Some(work_order_resource_loadings));
    } else if total_excess > *best_total_excess {
        *best_work_order_resource_loadings = work_order_resource_loadings.clone();
        *best_total_excess = total_excess;
    }
    Ok(None)
}

fn determine_normal_work_order_resource_loadings(
    period: &Period,
    technician_permutation: &mut [OperationalResource],
    work_load_permutation: &mut Vec<(Resources, Work)>,
) -> Result<Option<StrategicResources>>{
    let mut work_order_resource_loadings = StrategicResources::default();
    for operation_load in work_load_permutation {
        for operational_resource in technician_permutation.iter_mut() {
            if !operational_resource
                .skill_hours
                .keys()
                .contains(&operation_load.0)
            {
                continue;
            }

            if operation_load.1 <= operational_resource.total_hours {
                operational_resource
                    .skill_hours
                    .iter_mut()
                    .for_each(|(_, wor)| *wor -= operation_load.1);
                operational_resource.total_hours -= operation_load.1;
                work_order_resource_loadings.update_load(
                    period,
                    operation_load.0,
                    operation_load.1,
                    operational_resource,
                    LoadOperation::Add,
                );
                operation_load.1 = Work::from(0.0);
            ensure!(operational_resource.skill_hours.iter().all(|e| e.1 == &operational_resource.total_hours),
                "Each skill_hours should always be equal to the value of the total_hours\n{}", std::panic::Location::caller());
                break;
            } else {
                operation_load.1 -= operational_resource.total_hours;
                operational_resource
                    .skill_hours
                    .iter_mut()
                    .for_each(|(_res, wor)| *wor = Work::from(0.0));
                operational_resource.total_hours = Work::from(0.0);
                work_order_resource_loadings.update_load(
                    period,
                    operation_load.0,
                    operational_resource.total_hours,
                    operational_resource,
                    LoadOperation::Add,
                );
            ensure!(operational_resource.skill_hours.iter().all(|e| e.1 == &operational_resource.total_hours),
                "Each skill_hours should always be equal to the value of the total_hours\n{}", std::panic::Location::caller());
            }
        }

        if operation_load.1 != Work::from(0.0) {
            return Ok(None);
        }
    }
    Ok(Some(work_order_resource_loadings))
}

fn determine_difference_resources(
    capacity_resources: &HashMap<String, OperationalResource>,
    loading_resources: &HashMap<String, OperationalResource>,
) -> HashMap<String, OperationalResource> {
    let mut difference_resources = HashMap::new();
    for capacity in capacity_resources {
        let loading = loading_resources
            .get(capacity.0)
            .cloned()
            // What should happen if the resources are not available here?
            // This is where we should create a new entry
            .unwrap_or_default();
        let total_hours = capacity.1.total_hours - loading.total_hours;
        let skills = capacity.1.skill_hours.clone().into_keys().collect();

        let operational_resource = OperationalResource::new(capacity.0, total_hours, skills);

        difference_resources.insert(capacity.0.clone(), operational_resource);
    }
    difference_resources
}

pub fn calculate_period_difference(scheduled_period: &Period, latest_period: &Period) -> u64 {
    let scheduled_period_date = scheduled_period.end_date().to_owned();
    let latest_date = latest_period.end_date();
    let duration = scheduled_period_date.signed_duration_since(latest_date);
    let days = duration.num_days();
    std::cmp::max(days / 7, 0) as u64
}



impl<Ss> StrategicAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    // ISSUE #000
    pub fn update_resources_state(
        &mut self,
        _strategic_resources_request: StrategicRequestResource,
    ) -> Result<StrategicResponseResources> {
        // You should have done this! But you will need to
        // work a lot on this! You have to be comfortable with all
        // this. You will experience a lot of pain from doing this
        // work. But you will need to learn it
        // match strategic_resources_request {

        //     StrategicRequestResource::GetLoadings {
        //         periods_end: _,
        //         select_resources: _,
        //     } => {
        //         let loading = &self.solution.strategic_loadings;

        //         let strategic_response_resources =
        //
        // StrategicResponseResources::LoadingAndCapacities(loading.clone());
        //         Ok(strategic_response_resources)
        //     }
        //     StrategicRequestResource::GetCapacities {
        //         periods_end: _,
        //         select_resources: _,
        //     } => {
        //         let capacities = &self.parameters.strategic_capacity;

        //         let strategic_response_resources =
        //
        // StrategicResponseResources::LoadingAndCapacities(capacities.clone());
        //         Ok(strategic_response_resources)
        //     }
        //     StrategicRequestResource::GetPercentageLoadings {
        //         periods_end: _,
        //         resources: _,
        //     } => {
        //         let capacities = &self.parameters.strategic_capacity;
        //         let loadings = &self.solution.strategic_loadings;

        //         Algorithm::assert_that_capacity_is_respected(loadings, capacities)
        //             .context("Loadings exceed the capacities")?;
        //         Ok(StrategicResponseResources::Percentage(
        //             capacities.clone(),
        //             loadings.clone(),
        //         ))
        //     }
        // }
        todo!()
    }

    #[instrument(level = "info", skip_all)]
    pub fn update_scheduling_state(
        &mut self,
        strategic_scheduling_request: StrategicRequestScheduling,
    ) -> Result<StrategicResponseScheduling> {
        match strategic_scheduling_request {
            StrategicRequestScheduling::Schedule(schedule_work_order) => {
                let period = self
                    .parameters
                    .strategic_periods
                    .iter()
                    .find(|period| period.period_string() == schedule_work_order.period_string())
                    .cloned()
                    .with_context(|| {
                        format!(
                            "period: {:?} does not exist",
                            schedule_work_order.period_string()
                        )
                    })?;

                let mut number_of_work_orders = 0;
                for work_order_number in schedule_work_order.work_order_number {
                    let strategic_parameter = self
                        .parameters
                        .strategic_work_order_parameters
                        .get_mut(&work_order_number)
                        .unwrap();
                    if strategic_parameter.excluded_periods.contains(&period) {
                        strategic_parameter.excluded_periods.remove(&period);
                    }
                    self.parameters
                        .set_locked_in_period(work_order_number, period.clone())
                        .context("could not set locked in period")?;
                    number_of_work_orders += 1;
                }

                Ok(StrategicResponseScheduling::new(
                    number_of_work_orders,
                    period,
                ))
            }
            StrategicRequestScheduling::ExcludeFromPeriod(exclude_from_period) => {
                let period = self
                    .parameters
                    .strategic_periods
                    .iter()
                    .find(|period| {
                        period.period_string() == exclude_from_period.period_string().clone()
                    })
                    .with_context(|| {
                        format!(
                            "{} was not found in the {}",
                            exclude_from_period.period_string,
                            std::any::type_name::<TimeEnvironment>()
                        )
                    })
                    .cloned()?;

                let mut number_of_work_orders = 0;
                for work_order_number in exclude_from_period.work_order_number {
                    let solution = self
                        .solution
                        .strategic_scheduled_work_orders
                        .get(&work_order_number)
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .clone();

                    let strategic_parameter = self
                            .parameters
                            .strategic_work_order_parameters
                            .get_mut(&work_order_number)
                            .with_context(|| format!("The {:?} was not found in the {:#?}. The {:#?} should have been initialized at creation.", work_order_number, type_name::<WorkOrderParameter>(), type_name::<WorkOrderParameter>()))?;

                    assert!(!strategic_parameter.excluded_periods.contains(&solution));

                    strategic_parameter.excluded_periods.insert(period.clone());

                    // assert!(!strategic_parameter.excluded_periods.contains(self.solution.
                    // strategic_periods.get(&work_order_number).as_ref().unwrap().as_ref().
                    // unwrap()));

                    if let Some(locked_in_period) = &strategic_parameter.locked_in_period.clone() {
                        if strategic_parameter
                            .excluded_periods
                            .contains(locked_in_period)
                        {
                            strategic_parameter.locked_in_period = None;
                            event!(
                                Level::INFO,
                                "{:?} has been excluded from period {} and the locked in period has been removed",
                                work_order_number,
                                period.period_string()
                            );
                        }
                    }

                    let last_period = self.parameters.strategic_periods.iter().last().cloned();
                    // This should actually be done by the algorithm and not this procedure. I am
                    // not really sure what I should think or all this.
                    self.solution
                        .strategic_scheduled_work_orders
                        .insert(work_order_number, last_period);

                    // assert!(
                    //     !strategic_parameter.excluded_periods.contains(
                    //         self.solution
                    //             .strategic_scheduled_work_orders
                    //             .get(&work_order_number)
                    //             .as_ref()
                    //             .unwrap()
                    //             .as_ref()
                    //             .unwrap()
                    //     )
                    // );
                    number_of_work_orders += 1;
                }

                Ok(StrategicResponseScheduling::new(
                    number_of_work_orders,
                    period.clone(),
                ))
            }
        }
    }

    fn unschedule_specific_work_order(&mut self, work_order_number: WorkOrderNumber) -> Result<()> {
        let unschedule_from_period = self
            .solution
            .strategic_scheduled_work_orders
            .get_mut(&work_order_number)
            .with_context(|| {
                format!(
                    "{work_order_number:?}: was not present in the strategic periods"
                    
                )
            })?
            .take();

        if let Some(unschedule_from_period) = unschedule_from_period {
            let strategic_parameter = self
                .parameters
                .strategic_work_order_parameters
                .get(&work_order_number)
                .unwrap();

            let work_load = strategic_parameter.work_load.clone();

            let strategic_resources = self
                .determine_best_permutation(work_load, &unschedule_from_period, ScheduleWorkOrder::Unschedule)
                .with_context(|| format!("{:#?}\n{:#?}\nfor {:?}\nfile: {}\nline: {}", strategic_parameter, unschedule_from_period, ScheduleWorkOrder::Unschedule, file!(), line!()))?
                .context("Determining the StrategicResources associated with a unscheduling operation should always be possible")?;

            strategic_resources.assert_well_shaped_resources()?;
            self.update_loadings(strategic_resources, LoadOperation::Sub);
        }
        Ok(())
    }

    // FIX
    // Determine what to do with this
    pub fn populate_priority_queue(&mut self) {
        for work_order_number in self.solution.strategic_scheduled_work_orders.clone().keys() {
            let strategic_parameter = self
                .parameters
                .strategic_work_order_parameters
                .get(work_order_number)
                .expect(
                    "The StrategicParameter should always be available for the StrategicSolution",
                );

            if strategic_parameter.locked_in_period.is_some() {
                continue;
            }

            if self
                .solution
                .strategic_scheduled_work_orders
                .get(work_order_number)
                .unwrap()
                .is_none()
            {
                let strategic_work_order_weight = strategic_parameter.weight;
                self.solution_intermediate
                    .push(*work_order_number, strategic_work_order_weight);
            }
        }
    }
}

impl<Ss>
    From<Algorithm<StrategicSolution, StrategicParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>>
    for StrategicAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    fn from(
        value: Algorithm<
            StrategicSolution,
            StrategicParameters,
            PriorityQueue<WorkOrderNumber, u64>,
            Ss,
        >,
    ) -> Self {
        StrategicAlgorithm(value)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::str::FromStr;

    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use strategic_parameters::WorkOrderParameter;

    use super::*;

    impl WorkOrderParameter {
        pub fn new(
            locked_in_period: Option<Period>,
            excluded_periods: HashSet<Period>,
            latest_period: Period,
            weight: u64,
            work_load: HashMap<Resources, Work>,
        ) -> Self {
            Self {
                locked_in_period,
                excluded_periods,
                latest_period,
                weight,
                work_load,
            }
        }
    }

    #[test]
    fn test_determine_best_permutation() {}

    #[test]
    fn test_update_load_1() {
        let period = Period::from_str("2025-W23-24").unwrap();
        let resource = Resources::MtnMech;
        let load = Work::from(30.0);

        let capacity = Work::from(100.0);
        let operational_id = "OP-TEST";

        let operational_resource = OperationalResource::new(
            operational_id,
            capacity,
            vec![Resources::MtnMech, Resources::MtnElec, Resources::Prodtech],
        );

        let operational_resources_by_period =
            vec![(operational_id.to_string(), operational_resource.clone())]
                .into_iter()
                .collect();

        let strategic_resources_inner: HashMap<Period, HashMap<String, OperationalResource>> =
            vec![(period.clone(), operational_resources_by_period)]
                .into_iter()
                .collect();

        let mut strategic_resources = StrategicResources::new(strategic_resources_inner);

        strategic_resources.update_load(
            &period,
            resource,
            load,
            &operational_resource,
            LoadOperation::Add,
        );

        assert_eq!(
            strategic_resources
                .0
                .get(&period)
                .unwrap()
                .get(operational_id)
                .unwrap()
                .total_hours,
            Work::from(130.0)
        );
        assert!(
            strategic_resources
                .0
                .get(&period)
                .unwrap()
                .get(operational_id)
                .unwrap()
                .skill_hours
                .values()
                .all(|wor| *wor == Work::from(130.0))
        );
    }

    #[test]
    fn test_update_load_2() {
        let period = Period::from_str("2025-W23-24").unwrap();
        let resource = Resources::VenMech;
        let load = Work::from(30.0);

        let capacity = Work::from(100.0);
        let operational_id = "OP-TEST";

        let operational_resource = OperationalResource::new(
            operational_id,
            capacity,
            vec![Resources::MtnMech, Resources::MtnElec, Resources::Prodtech],
        );

        let operational_resources_by_period =
            vec![(operational_id.to_string(), operational_resource.clone())]
                .into_iter()
                .collect();

        let strategic_resources_inner: HashMap<Period, HashMap<String, OperationalResource>> =
            vec![(period.clone(), operational_resources_by_period)]
                .into_iter()
                .collect();

        let mut strategic_resources = StrategicResources::new(strategic_resources_inner);

        strategic_resources.update_load(
            &period,
            resource,
            load,
            &operational_resource,
            LoadOperation::Add,
        );

        assert_eq!(
            strategic_resources
                .0
                .get(&period)
                .unwrap()
                .get(operational_id)
                .unwrap()
                .total_hours,
            Work::from(130.0)
        );
        assert!(
            strategic_resources
                .0
                .get(&period)
                .unwrap()
                .get(operational_id)
                .unwrap()
                .skill_hours
                .values()
                .all(|wor| *wor == Work::from(130.0))
        );
        assert!(
            strategic_resources
                .0
                .get(&period)
                .unwrap()
                .get(operational_id)
                .unwrap()
                .skill_hours
                .contains_key(&resource)
        );
    }
    #[test]
    fn test_update_load_3() {
        let period = Period::from_str("2025-W23-24").unwrap();
        let resource = Resources::VenMech;
        let load = Work::from(30.0);

        let capacity = Work::from(100.0);
        let operational_id = "OP-TEST";

        let operational_resource = OperationalResource::new(
            operational_id,
            capacity,
            vec![Resources::MtnMech, Resources::MtnElec, Resources::Prodtech],
        );

        let operational_resources_by_period =
            vec![(operational_id.to_string(), operational_resource.clone())]
                .into_iter()
                .collect();

        let strategic_resources_inner: HashMap<Period, HashMap<String, OperationalResource>> =
            vec![(period.clone(), operational_resources_by_period)]
                .into_iter()
                .collect();

        let mut strategic_resources = StrategicResources::new(strategic_resources_inner);

        strategic_resources.update_load(
            &period,
            resource,
            load,
            &operational_resource,
            LoadOperation::Sub,
        );

        assert_eq!(
            strategic_resources
                .0
                .get(&period)
                .unwrap()
                .get(operational_id)
                .unwrap()
                .total_hours,
            Work::from(70.0)
        );
        assert!(
            strategic_resources
                .0
                .get(&period)
                .unwrap()
                .get(operational_id)
                .unwrap()
                .skill_hours
                .values()
                .all(|wor| *wor == Work::from(70.0))
        );
        assert!(
            strategic_resources
                .0
                .get(&period)
                .unwrap()
                .get(operational_id)
                .unwrap()
                .skill_hours
                .contains_key(&resource)
        );
    }

    #[test]
    fn test_determine_normal_work_order_resource_loadings_1() {
        let period = Period::from_str("2025-W23-24").unwrap();

        let mut technician_permutation = vec![
            OperationalResource::new(
                "OP_TEST_0",
                Work::from(40.0),
                vec![Resources::MtnMech, Resources::MtnElec],
            ),
            OperationalResource::new(
                "OP_TEST_1",
                Work::from(40.0),
                vec![Resources::MtnScaf, Resources::MtnElec],
            ),
        ];

        let mut work_load_permutation = vec![
            (Resources::MtnMech, Work::from(30.0)),
            (Resources::MtnElec, Work::from(30.0)),
            (Resources::MtnScaf, Work::from(30.0)),
        ];

        let strategic_resource_option = super::determine_normal_work_order_resource_loadings(
            &period,
            &mut technician_permutation,
            &mut work_load_permutation,
        ).unwrap();

        assert!(strategic_resource_option.is_none());
    }

    #[test]
    fn test_determine_normal_work_order_resource_loadings_2() {
        let period = Period::from_str("2025-W23-24").unwrap();

        let mut technician_permutation = vec![
            OperationalResource::new(
                "OP_TEST_0",
                Work::from(40.0),
                vec![Resources::MtnMech, Resources::MtnElec],
            ),
            OperationalResource::new(
                "OP_TEST_1",
                Work::from(40.0),
                vec![Resources::MtnScaf, Resources::MtnElec],
            ),
        ];

        let mut work_load_permutation = vec![
            (Resources::MtnMech, Work::from(20.0)),
            (Resources::MtnElec, Work::from(20.0)),
            (Resources::MtnScaf, Work::from(20.0)),
        ];

        let strategic_resource_option = super::determine_normal_work_order_resource_loadings(
            &period,
            &mut technician_permutation,
            &mut work_load_permutation,
        ).unwrap();

        assert!(strategic_resource_option.is_some());

        let operational_resource_1 = OperationalResource::new(
            "OP_TEST_0",
            Work::from(40.0),
            vec![Resources::MtnMech, Resources::MtnElec],
        );
        let operational_resource_2 = OperationalResource::new(
            "OP_TEST_1",
            Work::from(20.0),
            vec![Resources::MtnScaf, Resources::MtnElec],
        );

        let mut strategic_resource = StrategicResources::default();
        strategic_resource.insert_operational_resource(period.clone(), operational_resource_1);
        strategic_resource.insert_operational_resource(period.clone(), operational_resource_2);

        assert_eq!(strategic_resource, strategic_resource_option.unwrap());
    }

    #[test]
    fn test_determine_forced_work_order_resource_loadings_1() {
        let period = Period::from_str("2025-W23-24").unwrap();

        let mut technician_permutation = vec![
            OperationalResource::new(
                "OP_TEST_0",
                Work::from(40.0),
                vec![Resources::MtnMech, Resources::MtnElec],
            ),
            OperationalResource::new(
                "OP_TEST_1",
                Work::from(40.0),
                vec![Resources::MtnScaf, Resources::MtnElec],
            ),
        ];

        let mut work_load_permutation = vec![
            (Resources::MtnMech, Work::from(30.0)),
            (Resources::MtnElec, Work::from(30.0)),
            (Resources::MtnScaf, Work::from(30.0)),
        ];

        let mut best_strategic_resource = StrategicResources::default();
        let mut best_total_excess = Work::from(-9999999.0);

        let strategic_resources = determine_forced_work_order_resource_loadings(
            &period,
            &mut best_total_excess,
            &mut best_strategic_resource,
            &mut technician_permutation,
            &mut work_load_permutation,
        ).with_context(|| format!("Incorrectly determined the forced work order loadings\n{}", std::panic::Location::caller())).unwrap();

        assert!(strategic_resources.is_none());

        let operational_resource_1 = OperationalResource::new(
            "OP_TEST_0",
            Work::from(45.0),
            vec![Resources::MtnMech, Resources::MtnElec],
        );
        let operational_resource_2 = OperationalResource::new(
            "OP_TEST_1",
            Work::from(45.0),
            vec![Resources::MtnScaf, Resources::MtnElec],
        );
        let mut strategic_resources = StrategicResources::default();

        strategic_resources.insert_operational_resource(period.clone(), operational_resource_1);
        strategic_resources.insert_operational_resource(period.clone(), operational_resource_2);

        assert_eq!(strategic_resources, best_strategic_resource);
    }

    #[test]
    fn test_determine_forced_work_order_resource_loadings_2() {
        let period = Period::from_str("2025-W23-24").unwrap();

        let mut technician_permutation = vec![
            OperationalResource::new(
                "OP_TEST_0",
                Work::from(40.0),
                vec![Resources::MtnMech, Resources::MtnElec],
            ),
            OperationalResource::new(
                "OP_TEST_1",
                Work::from(40.0),
                vec![Resources::MtnScaf, Resources::MtnElec],
            ),
        ];

        let mut work_load_permutation = vec![
            (Resources::MtnMech, Work::from(20.0)),
            (Resources::MtnElec, Work::from(20.0)),
            (Resources::MtnScaf, Work::from(20.0)),
        ];

        let mut best_strategic_resource = StrategicResources::default();
        let mut best_total_excess = Work::from(-9999999.0);

        let strategic_resources_option = determine_forced_work_order_resource_loadings(
            &period,
            &mut best_total_excess,
            &mut best_strategic_resource,
            &mut technician_permutation,
            &mut work_load_permutation,
        ).unwrap();

        assert!(strategic_resources_option.is_some());
        let operational_resource_1 = OperationalResource::new(
            "OP_TEST_0",
            Work::from(30.0),
            vec![Resources::MtnMech, Resources::MtnElec],
        );
        let operational_resource_2 = OperationalResource::new(
            "OP_TEST_1",
            Work::from(30.0),
            vec![Resources::MtnScaf, Resources::MtnElec],
        );
        let mut strategic_resources = StrategicResources::default();

        strategic_resources.insert_operational_resource(period.clone(), operational_resource_1);
        strategic_resources.insert_operational_resource(period.clone(), operational_resource_2);

        assert_eq!(strategic_resources, strategic_resources_option.unwrap());
    }

    #[test]
    fn test_determine_forced_work_order_resource_loadings_3() {
        let period = Period::from_str("2025-W23-24").unwrap();

        let mut technician_permutation = vec![
            OperationalResource::new(
                "OP_TEST_0",
                Work::from(30.0),
                vec![Resources::MtnMech, Resources::MtnElec],
            ),
            OperationalResource::new(
                "OP_TEST_1",
                Work::from(30.0),
                vec![Resources::MtnScaf, Resources::MtnElec],
            ),
        ];

        let mut work_load_permutation = vec![
            (Resources::MtnMech, Work::from(20.0)),
            (Resources::MtnElec, Work::from(20.0)),
            (Resources::MtnScaf, Work::from(20.0)),
            (Resources::VenMech, Work::from(20.0)),
        ];

        let mut best_strategic_resource = StrategicResources::default();
        let mut best_total_excess = Work::from(-9999999.0);

        let strategic_resources_option = determine_forced_work_order_resource_loadings(
            &period,
            &mut best_total_excess,
            &mut best_strategic_resource,
            &mut technician_permutation,
            &mut work_load_permutation,
        ).unwrap();

        assert!(strategic_resources_option.is_some());
        // Is this the right way of doing things? I do not think that it is...
        let operational_resource_1 = OperationalResource::new(
            "OP_TEST_0",
            Work::from(30.0),
            vec![Resources::MtnMech, Resources::MtnElec],
        );
        let operational_resource_2 = OperationalResource::new(
            "OP_TEST_1",
            Work::from(30.0),
            vec![Resources::MtnScaf, Resources::MtnElec],
        );
        let operational_resource_3 = OperationalResource::new(
            "VEN-MECH_dummy",
            Work::from(20.0),
            vec![Resources::VenMech],
        );

        let mut strategic_resources = StrategicResources::default();

        strategic_resources.insert_operational_resource(period.clone(), operational_resource_1);
        strategic_resources.insert_operational_resource(period.clone(), operational_resource_2);
        strategic_resources.insert_operational_resource(period.clone(), operational_resource_3);

        assert_eq!(strategic_resources, strategic_resources_option.unwrap());
    }

    #[test]
    fn test_determine_unschedule_work_order_loadings_1() {
        let period = Period::from_str("2025-W23-24").unwrap();

        let mut work_load_permutation = vec![
            (Resources::MtnMech, Work::from(20.0)),
            (Resources::MtnElec, Work::from(20.0)),
        ];

        let loading_resources = [
            OperationalResource::new(
                "OP_TEST_0",
                Work::from(20.0),
                vec![Resources::MtnMech, Resources::MtnElec],
            ),
            OperationalResource::new(
                "OP_TEST_1",
                Work::from(20.0),
                vec![Resources::MtnScaf, Resources::MtnElec],
            ),
        ];

        let strategic_resources_option_calculated = determine_unschedule_work_resource_loadings(
            &period,
            &loading_resources,
            &mut work_load_permutation,
        ).unwrap();

        assert!(strategic_resources_option_calculated.is_some());

        let operational_resource_1 = OperationalResource::new(
            "OP_TEST_0",
            Work::from(-20.0),
            vec![Resources::MtnMech, Resources::MtnElec],
        );
        let operational_resource_2 = OperationalResource::new(
            "OP_TEST_1",
            Work::from(-20.0),
            vec![Resources::MtnScaf, Resources::MtnElec],
        );
        let mut strategic_resources_manual = StrategicResources::default();

        strategic_resources_manual.insert_operational_resource(period.clone(), operational_resource_1);
        strategic_resources_manual.insert_operational_resource(period.clone(), operational_resource_2);

        assert_eq!(strategic_resources_manual, strategic_resources_option_calculated.unwrap());
    }

    #[test]
    fn test_determine_unschedule_work_order_loadings_2() -> Result<()> {
        let period = Period::from_str("2026-W33-34").unwrap();

        let work_load_permutation = [
            (Resources::MtnMech, Work::from(2.0)),
            (Resources::Prodtech, Work::from(2.0)),
            (Resources::MtnInst, Work::from(2.0)),
            (Resources::MtnElec, Work::from(2.0)),
        ];

        let loading_resources = [
            OperationalResource::new(
                "OP_TEST_1",
                Work::from(6.0),
                vec![Resources::MtnMech, Resources::Prodtech, Resources::MtnElec],
            ),
            OperationalResource::new(
                "OP_TEST_2",
                Work::from(0.0),
                vec![Resources::MtnScaf, Resources::MtnRigg, Resources::MtnLagg],
            ),
            OperationalResource::new(
                "OP_TEST_0",
                Work::from(2.0),
                vec![Resources::MtnInst, Resources::MtnMech, Resources::MtnElec],
            ),
        ];

        // We should readjust this to take in a Vec<T> instead of a HashMap<K, V>. That
        // is a much better approach for dealing with this. And then we reuse
        // the structure that we already have. QUESTION:
        // Can you combine the .permutation() with random access.
        let strategic_resources_option = determine_unschedule_work_resource_loadings(
            &period,
            &loading_resources,
            &mut work_load_permutation.clone(),
        ).unwrap();

        super::assert_work_load_equal_to_strategic_resource(
            &period,
            strategic_resources_option.as_ref().unwrap(),
            &HashMap::from(work_load_permutation),
            LoadOperation::Sub,
        )?;

        assert!(strategic_resources_option.is_some());

        // let operational_resource_1 = OperationalResource::new("OP_TEST_0",
        // Work::from(-20.0), vec![Resources::MtnMech, Resources::MtnElec]); let
        // operational_resource_2 = OperationalResource::new("OP_TEST_1",
        // Work::from(-20.0), vec![Resources::MtnScaf, Resources::MtnElec]); let
        // mut strategic_resources = StrategicResources::default();

        // strategic_resources.insert_operatensureonal_resource(period.clone(),
        // operational_resource_1); strategic_resources.
        // insert_operational_resource(period.clone(), operational_resource_2);

        // assert_eq!(strategic_resources, strategic_resources_option.unwrap());

        //FIX MAKE A PROPTEST
        Ok(())
    }

    // Should this test go into the integration testing instead? I
    // think that is a really good idea. Also you should never s
    #[test]
    fn test_unschedule_random_work_orders() -> Result<()> {
        let periods: Vec<Period> = vec![
            Period::from_str("2023-W47-48").unwrap(),
            Period::from_str("2023-W49-50").unwrap(),
        ];

        let _latest_period = Period::from_str("2023-W49-50").unwrap();

        let mut work_load_1 = HashMap::new();
        let mut work_load_2 = HashMap::new();
        let mut work_load_3 = HashMap::new();

        work_load_1.insert(Resources::MtnMech, Work::from(10.0));
        work_load_1.insert(Resources::MtnElec, Work::from(10.0));
        work_load_1.insert(Resources::Prodtech, Work::from(10.0));

        work_load_2.insert(Resources::MtnMech, Work::from(20.0));
        work_load_2.insert(Resources::MtnElec, Work::from(20.0));
        work_load_2.insert(Resources::Prodtech, Work::from(20.0));

        work_load_3.insert(Resources::MtnMech, Work::from(30.0));
        work_load_3.insert(Resources::MtnElec, Work::from(30.0));
        work_load_3.insert(Resources::Prodtech, Work::from(30.0));

        let mut strategic_resources = StrategicResources::default();

        let operational_resource_0 = OperationalResource::new(
            "OP_TEST_0",
            Work::from(40.0),
            vec![Resources::MtnMech, Resources::MtnElec],
        );
        let operational_resource_1 = OperationalResource::new(
            "OP_TEST_1",
            Work::from(40.0),
            vec![Resources::MtnMech, Resources::MtnElec],
        );

        strategic_resources.insert_operational_resource(periods[0].clone(), operational_resource_0);
        strategic_resources.insert_operational_resource(periods[1].clone(), operational_resource_1);

        // This way of making parameters needs to go away. Is the right call here to
        // simply delete the let scheduling_environment =
        // Arc::new(Mutex::new(SchedulingEnvironment::default()));

        // let id = Id::new("Strategic", vec![], vec![Asset::Unknown]);

        // let mut strategic_parameters = StrategicParameters::new(
        //     &id,
        //     StrategicOptions::default(),
        //     &scheduling_environment.lock().unwrap(),
        // )?;

        // let strategic_parameter_1 = WorkOrderParameter::new(
        //     None,
        //     HashSet::new(),
        //     latest_period.clone(),
        //     1000,
        //     work_load_1,
        // );

        // let strategic_parameter_2 = WorkOrderParameter::new(
        //     None,
        //     HashSet::new(),
        //     latest_period.clone(),
        //     1000,
        //     work_load_2,
        // );

        // let strategic_parameter_3 = WorkOrderParameter::new(
        //     None,
        //     HashSet::new(),
        //     latest_period.clone(),
        //     1000,
        //     work_load_3,
        // );

        // let work_order_number_1 = WorkOrderNumber(2200000001);
        // let work_order_number_2 = WorkOrderNumber(2200000002);
        // let work_order_number_3 = WorkOrderNumber(2200000003);

        // strategic_parameters
        //     .strategic_work_order_parameters
        //     .insert(work_order_number_1, strategic_parameter_1);

        // strategic_parameters
        //     .strategic_work_order_parameters
        //     .insert(work_order_number_2, strategic_parameter_2);

        // strategic_parameters
        //     .strategic_work_order_parameters
        //     .insert(work_order_number_3, strategic_parameter_3);

        // let scheduling_environment =
        // Arc::new(Mutex::new(SchedulingEnvironment::default()));

        // let id = Id::new("Strategic", vec![], vec![Asset::Unknown]);

        // let strategic_parameters = StrategicParameters::new(
        //     &id,
        //     StrategicOptions::default(),
        //     &scheduling_environment.lock().unwrap(),
        // )?;

        // let strategic_solution = StrategicSolution::new(&strategic_parameters);

        // Actor::builder().agent_id(&id).
        // scheduling_environment(scheduling_environment).algorithm(|con|con.id(&id).
        // parameters(options, scheduling_environment)).configurations();

        // let mut strategic_algorithm = Algorithm::new(
        //     &Id::default(),
        //     strategic_solution,
        //     strategic_parameters,
        //     ArcSwapSharedSolution::default().into(),
        // );

        // strategic_algorithm
        //     .solution
        //     .strategic_scheduled_work_orders
        //     .insert(work_order_number_1, Some(periods[0].clone()));
        // strategic_algorithm
        //     .solution
        //     .strategic_scheduled_work_orders
        //     .insert(work_order_number_2, Some(periods[1].clone()));
        // strategic_algorithm
        //     .solution
        //     .strategic_scheduled_work_orders
        //     .insert(work_order_number_3, Some(periods[1].clone()));

        // let operational_resource_0 = OperationalResource::new("OP_TEST_0",
        // Work::from(30.0), vec![     Resources::MtnMech,
        //     Resources::MtnElec,
        //     Resources::Prodtech,
        // ]);
        // let operational_resource_1 =
        //     OperationalResource::new("OP_TEST_1", Work::from(150.0), vec![
        //         Resources::MtnMech,
        //         Resources::MtnElec,
        //         Resources::Prodtech,
        //     ]);

        // strategic_algorithm
        //     .solution
        //     .strategic_loadings
        //     .insert_operational_resource(periods[0].clone(), operational_resource_0);
        // strategic_algorithm
        //     .solution
        //     .strategic_loadings
        //     .insert_operational_resource(periods[1].clone(), operational_resource_1);

        // let seed: [u8; 32] = [
        //     1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        // 21, 22, 23, 24,     25, 26, 27, 28, 29, 30, 31, 32,
        // ];

        // let rng = StdRng::from_seed(seed);

        // let strategic_options = StrategicOptions {
        //     number_of_removed_work_order: 2,
        //     rng,
        //     urgency_weight: 1,
        //     resource_penalty_weight: 1,
        //     clustering_weight: 1,
        //     work_order_configurations: todo!(),
        //     material_to_period: todo!(),
        // };

        // strategic_algorithm.parameters.strategic_options = strategic_options;

        // strategic_algorithm.unschedule().expect(
        //     "It should always be possible to unschedule random work orders in the
        // strategic agent", );

        // assert_eq!(
        //     *strategic_algorithm
        //         .solution
        //         .strategic_scheduled_work_orders
        //         .get(&WorkOrderNumber(2200000001))
        //         .unwrap(),
        //     Some(Period::from_str("2023-W47-48").unwrap())
        // );

        // assert_eq!(
        //     *strategic_algorithm
        //         .solution
        //         .strategic_scheduled_work_orders
        //         .get(&WorkOrderNumber(2200000002))
        //         .unwrap(),
        //     None
        // );

        // assert_eq!(
        //     *strategic_algorithm
        //         .solution
        //         .strategic_scheduled_work_orders
        //         .get(&WorkOrderNumber(2200000003))
        //         .unwrap(),
        //     None
        // );
        Ok(())
    }

    #[test]
    fn test_calculate_period_difference_1() {
        let scheduled_period = Period::from_str("2023-W47-48");
        let latest_period = Period::from_str("2023-W49-50");

        let difference =
            calculate_period_difference(&scheduled_period.unwrap(), &latest_period.unwrap());

        assert_eq!(difference, 0);
    }
    #[test]
    fn test_calculate_period_difference_2() {
        let period_1 = Period::from_str("2023-W47-48");
        let period_2 = Period::from_str("2023-W45-46");

        let difference = calculate_period_difference(&period_1.unwrap(), &period_2.unwrap());

        assert_eq!(difference, 2);
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
    fn test_unschedule_work_order_none_in_scheduled_period() -> Result<()> {
        let _work_order_number = WorkOrderNumber(2100000001);
        let periods = [Period::from_str("2026-W41-42").unwrap()];
        let mut strategic_resources = StrategicResources::default();

        let operational_resource_0 = OperationalResource::new(
            "OP_TEST_0",
            Work::from(40.0),
            vec![Resources::MtnMech, Resources::MtnElec],
        );

        strategic_resources.insert_operational_resource(periods[0].clone(), operational_resource_0);

        // let scheduling_environment =
        // Arc::new(Mutex::new(SchedulingEnvironment::default()));

        // let id = Id::new("Strategic", vec![], vec![Asset::Unknown]);

        // let mut strategic_parameters = StrategicParameters::new(
        //     &id,
        //     StrategicOptions::default(),
        //     &scheduling_environment.lock().unwrap(),
        // )?;

        // let strategic_parameter = WorkOrderParameter::new(
        //     None,
        //     HashSet::new(),
        //     periods[0].clone(),
        //     1000,
        //     HashMap::from([(Resources::MtnMech, Work::from(5.0))]),
        // );

        // strategic_parameters
        //     .strategic_work_order_parameters
        //     .insert(work_order_number, strategic_parameter);

        // let tactical_solution_builder = TacticalSolutionBuilder::new();

        // let mut tactical_days = HashMap::new();
        // tactical_days.insert(work_order_number, WhereIsWorkOrder::NotScheduled);

        // let tactical_solution = tactical_solution_builder
        //     .with_tactical_days(tactical_days)
        //     .build();

        // let shared_solution = SharedSolution {
        //     tactical: tactical_solution,
        //     ..SharedSolution::default()
        // };

        // // This is all a complete mess. I think that we should really think about
        // completing all this code // and then proceed to the next step.
        // let arc_swap_shared_solution =
        //     ArcSwapSharedSolution(ArcSwap::from_pointee(shared_solution));

        // let mut strategic_solution = StrategicSolution::new(&strategic_parameters);

        // strategic_solution
        //     .strategic_scheduled_work_orders
        //     .insert(work_order_number, Some(periods[0].clone()));

        // let mut strategic_algorithm = Algorithm::new(
        //     &Id::default(),
        //     strategic_solution,
        //     strategic_parameters,
        //     arc_swap_shared_solution.into(),
        // );

        // let operational_resource_0 = OperationalResource::new("OP_TEST_0",
        // Work::from(30.0), vec![     Resources::MtnMech,
        //     Resources::MtnElec,
        //     Resources::Prodtech,
        // ]);

        // strategic_algorithm
        //     .solution
        //     .strategic_loadings
        //     .insert_operational_resource(periods[0].clone(), operational_resource_0);

        // strategic_algorithm
        //     .update_based_on_shared_solution()
        //     .unwrap();

        // strategic_algorithm
        //     .unschedule_specific_work_order(work_order_number)
        //     .unwrap();
        // assert_eq!(
        //     *strategic_algorithm
        //         .solution
        //         .strategic_scheduled_work_orders
        //         .get(&work_order_number)
        //         .unwrap(),
        //     None
        // );
        Ok(())
    }

    #[test]
    fn test_period_clone_equality() {
        let period_1 = Period::from_str("2023-W47-48").unwrap();
        let period_2 = Period::from_str("2023-W47-48").unwrap();

        assert_eq!(period_1, period_2);
        assert_eq!(period_1, period_1.clone());
    }
}
