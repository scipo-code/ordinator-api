mod assert_functions;
pub mod tactical_interface;
pub mod tactical_parameters;
pub mod tactical_resources;
pub mod tactical_solution;

use std::cmp::Ordering;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use chrono::TimeDelta;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::algorithm::LoadOperation;
use ordinator_actor_core::traits::AbLNSUtils;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_actor_core::traits::ObjectiveValueType;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::StrategicInterface;
use ordinator_orchestrator_actor_traits::SystemSolutionTrait;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder;
use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use priority_queue::PriorityQueue;
use rand::seq::IndexedRandom;
use tactical_solution::TacticalObjectiveValue;
use tactical_solution::TacticalScheduledOperations;
use tactical_solution::TacticalSolution;
use tracing::Level;
use tracing::event;

use self::assert_functions::TacticalAssertions;
use self::tactical_parameters::TacticalParameters;
use self::tactical_solution::OperationSolution;
use super::TacticalOptions;

// If you had a single crate you should simply call thie
pub struct TacticalAlgorithm<Ss>(
    Algorithm<TacticalSolution, TacticalParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>,
)
where
    TacticalSolution: Solution,
    TacticalParameters: Parameters,
    Ss: SystemSolutionTrait;

// FIX
// Move the `tactical_days` into the parameters.
// QUESTION
// I think that we should delete all these, and turn the nested hashmaps into
// Vec<Vec<Work>> instead. That would be a much better solution.
// TODO [ ]
// Delete all the getters and turn the TacticalResources into a array based
// representation.
// TODO [ ]
// You have to make this thing work.
impl<Ss> TacticalAlgorithm<Ss>
where
    TacticalSolution: Solution,
    TacticalParameters: Parameters,
    Ss: SystemSolutionTrait,
{
    pub fn capacity(&self, resource: &Resources, day: &Day) -> &Work {
        self.parameters
            .tactical_capacity
            .resources
            .get(resource)
            .unwrap()
            .days
            .get(day)
            .unwrap()
    }

    pub fn capacity_mut(&mut self, resource: &Resources, day: &Day) -> &mut Work {
        self.parameters
            .tactical_capacity
            .resources
            .get_mut(resource)
            .unwrap()
            .days
            .get_mut(day)
            .unwrap()
    }

    // This is a horrible way of working with the data. What should be done instead?
    pub fn loading(&self, resource: &Resources, day: &Day) -> &Work {
        self.solution
            .tactical_loadings
            .resources
            .get(resource)
            .unwrap()
            .days
            .get(day)
            .unwrap()
    }

    pub fn loading_mut(&mut self, resource: &Resources, day: &Day) -> &mut Work {
        self.solution
            .tactical_loadings
            .resources
            .get_mut(resource)
            .unwrap()
            .days
            .get_mut(day)
            .unwrap()
    }

    fn determine_aggregate_excess(&self, tactical_objective_value: &mut TacticalObjectiveValue) {
        let mut objective_value_from_excess = 0;
        for resource in self.parameters.tactical_capacity.resources.keys() {
            for day in self.parameters.tactical_days.clone() {
                let excess_capacity = self.loading(resource, &day) - self.capacity(resource, &day);

                if excess_capacity > Work::from(0.0) {
                    objective_value_from_excess += excess_capacity.to_f64() as u64;
                }
            }
        }
        tactical_objective_value.resource_penalty.1 = objective_value_from_excess;
    }

    fn determine_tardiness(&mut self, tactical_objective_value: &mut TacticalObjectiveValue) {
        let mut objective_value_from_tardiness = 0;
        for (work_order_number, _solution) in self
            .solution
            .tactical_work_orders
            .0
            .iter()
            .filter(|(_, ts)| ts.is_tactical())
        {
            let tactical_parameter = self
                .parameters
                .tactical_work_orders
                .get(work_order_number)
                .unwrap();
            // FIX START HERE.

            // What does it mean that the StrategicAgent does not have the work order yet
            // What should we do to give him the correct state
            let period_start_date = match &self
                .loaded_shared_solution
                .strategic()
                .scheduled_task(work_order_number)
                .unwrap_or(&Option::None)
            {
                Some(period) => period.start_date().date_naive(),
                None => tactical_parameter.earliest_allowed_start_date,
            };

            let mut activity_keys: Vec<ActivityNumber> = tactical_parameter
                .tactical_operation_parameters
                .keys()
                .cloned()
                .collect();

            activity_keys.sort_unstable_by(|a, b| b.cmp(a));

            let last_activity = activity_keys.last().unwrap();

            let last_day = self
                .solution
                .tactical_scheduled_days(work_order_number, last_activity)
                .expect("Missing state from the tactical agent when calculating objective value")
                .last()
                .unwrap()
                .0
                .date()
                .date_naive();

            let day_difference = (last_day - period_start_date).max(TimeDelta::zero());

            objective_value_from_tardiness +=
                tactical_parameter.weight * day_difference.num_days() as u64;
        }
        tactical_objective_value.urgency.1 = objective_value_from_tardiness;
    }
}

impl<Ss> ActorBasedLargeNeighborhoodSearch for TacticalAlgorithm<Ss>
where
    Algorithm<TacticalSolution, TacticalParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>:
        AbLNSUtils<SolutionType = TacticalSolution>,
    TacticalSolution: Solution,
    TacticalParameters: Parameters,
    Ss: SystemSolutionTrait<Tactical = TacticalSolution>,
{
    type Algorithm =
        Algorithm<TacticalSolution, TacticalParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>;
    type Options = TacticalOptions;

    fn incorporate_shared_state(&mut self) -> Result<bool> {
        Ok(true)
    }

    fn make_atomic_pointer_swap(&mut self) {
        // Performance enhancements:
        // * COW: #[derive(Clone)] struct SharedSolution<'a> { tactical: Cow<'a,
        //   TacticalSolution>, // other fields... }
        //
        // * Reuse the old SharedSolution, cloning only the fields that are needed. let
        //   shared_solution = Arc::new(SharedSolution { tactical:
        //   self.solution.clone(), // Copy over other fields without cloning
        //   ..(**old).clone() });
        self.arc_swap_shared_solution.rcu(|old| {
            let mut shared_solution = (**old).clone();
            shared_solution.tactical_swap(&self.id, self.solution.clone());
            Arc::new(shared_solution)
        });
    }

    fn calculate_objective_value(
        &mut self,
        options: &Self::Options,
    ) -> Result<
        ObjectiveValueType<
            <<Self::Algorithm as AbLNSUtils>::SolutionType as Solution>::ObjectiveValue,
        >,
    > {
        let mut tactical_objective_value = TacticalObjectiveValue::new(options);

        self.determine_tardiness(&mut tactical_objective_value);

        // Calculate penalty for exceeding the capacity
        self.determine_aggregate_excess(&mut tactical_objective_value);

        tactical_objective_value.aggregate_objectives();

        if tactical_objective_value.objective_value < self.solution.objective_value.objective_value
        {
            event!(Level::INFO, tactical_objective_value_better = ?tactical_objective_value);
            Ok(ObjectiveValueType::Better(tactical_objective_value))
        } else {
            event!(Level::INFO, tactical_objective_value_worse = ?tactical_objective_value);
            Ok(ObjectiveValueType::Worse)
        }
    }

    fn schedule(&mut self) -> Result<()> {
        self.asset_that_loading_matches_scheduled()
            .with_context(|| format!("TESTING_ASSERTION on line: {}", line!()))?;
        for (work_order_number, solution) in &self.solution.tactical_work_orders.0.clone() {
            let tactical_parameter = self
                .parameters
                .tactical_work_orders
                .get(work_order_number)
                .expect("TacticalParameter should ALWAYS be available for a TacticalSolution")
                .clone();

            // All the work orders that does not have a solution gets pushed to the queue.
            if matches!(solution, WhereIsWorkOrder::NotScheduled) {
                self.solution_intermediate
                    .push(*work_order_number, tactical_parameter.weight);
            }
        }

        let mut start_day_index = 0;

        let mut loop_state: LoopState = LoopState::Unscheduled;

        let mut current_work_order_number = match self.solution_intermediate.pop() {
            Some((work_order_number, _)) => work_order_number,
            None => return Ok(()),
        };

        let mut counter = 0;
        // The issue is that the code here is running a lot of iterations. What should
        // we do about this? I am not really sure! I thi
        'back_to_loop_state_handle: loop {
            counter += 1;

            event!(
                Level::DEBUG,
                main_loop_counter = counter,
                start_day_index = start_day_index,
                priority_queue_len = self.solution_intermediate.len(),
            );
            let tactical_parameter = match loop_state {
                LoopState::Unscheduled => {
                    start_day_index += 1;
                    self.parameters
                        .tactical_work_orders
                        .get(&current_work_order_number)
                        .unwrap()
                }
                LoopState::Scheduled => {
                    start_day_index = 0;

                    current_work_order_number = match self.solution_intermediate.pop() {
                        Some((work_order_number, _)) => work_order_number,
                        None => {
                            event!(Level::INFO, "main_loop break");
                            break;
                        }
                    };

                    self.parameters
                        .tactical_work_orders
                        .get(&current_work_order_number)
                        .unwrap()
                }
                LoopState::ReleasedFromTactical => {
                    self.solution
                        .release_from_tactical_solution(&current_work_order_number);

                    start_day_index = 0;

                    current_work_order_number = match self.solution_intermediate.pop() {
                        Some((work_order_number, _)) => work_order_number,
                        None => {
                            event!(Level::INFO, "main_loop break");
                            break;
                        }
                    };

                    self.parameters
                        .tactical_work_orders
                        .get(&current_work_order_number)
                        .unwrap()
                }
            };

            let mut operation_solutions = TacticalScheduledOperations::default();

            let mut all_days = self.parameters.tactical_days.clone();

            let allowed_starting_days: Vec<&Day> = self
                .parameters
                .tactical_days
                .iter()
                .filter(|day| {
                    tactical_parameter.earliest_allowed_start_date <= day.date().date_naive()
                })
                .collect();

            let start_day: Day = match allowed_starting_days.get(start_day_index) {
                Some(start_day) => (*start_day).clone(),
                None => {
                    loop_state = LoopState::ReleasedFromTactical;
                    continue 'back_to_loop_state_handle;
                }
            };

            let allowed_days: Vec<_> = all_days
                .iter_mut()
                .filter(|date| start_day.date() <= date.date())
                .collect();

            let mut current_day = allowed_days.into_iter().peekable();

            let mut sorted_activities = tactical_parameter
                .tactical_operation_parameters
                .keys()
                .clone()
                .collect::<Vec<&ActivityNumber>>();

            sorted_activities.sort();

            for activity in sorted_activities {
                let operation_parameters = tactical_parameter
                    .tactical_operation_parameters
                    .get(activity)
                    .expect("The work order should always have its corresponding parameters");

                let resource = operation_parameters.resource;

                let current_day_peek = match current_day.peek() {
                    Some(day) => day,
                    None => {
                        loop_state = LoopState::ReleasedFromTactical;
                        continue 'back_to_loop_state_handle;
                    }
                };

                let first_day_remaining_capacity =
                    match self.remaining_capacity(&resource, current_day_peek) {
                        Some(remaining_capacity) => remaining_capacity,
                        None => {
                            loop_state = LoopState::Unscheduled;
                            continue 'back_to_loop_state_handle;
                        }
                    };

                let loadings = determine_load(
                    first_day_remaining_capacity,
                    &operation_parameters.operating_time,
                    operation_parameters.work_remaining,
                );

                let mut activity_load = Vec::<(Day, Work)>::new();
                // The breaks here mean that the code might input a partial work order
                // This should not matter for correctness.
                for load in loadings {
                    let day = match current_day.peek() {
                        Some(day) => (*day).clone(),
                        None => {
                            break;
                        }
                    };
                    activity_load.push((day, load));

                    current_day.next();

                    let peek_next_day = current_day.peek();
                    let current_day = match peek_next_day {
                        Some(next_day) => next_day,
                        None => {
                            break;
                        }
                    };

                    if self.remaining_capacity(&resource, current_day).is_none() {
                        loop_state = LoopState::Unscheduled;
                        continue 'back_to_loop_state_handle;
                    };
                }

                let operation_solution = OperationSolution::new(
                    activity_load,
                    resource,
                    operation_parameters.number,
                    operation_parameters.work_remaining,
                    current_work_order_number,
                    *activity,
                );
                operation_solutions.insert_operation_solution(*activity, operation_solution);
            }

            self.update_loadings(&operation_solutions, LoadOperation::Add)?;
            loop_state = LoopState::Scheduled;

            event!(Level::INFO, "{}", operation_solutions);

            self.solution
                .tactical_insert_work_order(current_work_order_number, operation_solutions);
            self.asset_that_loading_matches_scheduled()
                .with_context(|| format!("TESTING_ASSERTION on line: {}", line!()))?;
        }
        Ok(())
    }

    fn unschedule(&mut self) -> Result<()> {
        let work_order_numbers: Vec<WorkOrderNumber> = self
            .solution
            .tactical_work_orders
            .0
            .clone()
            .into_keys()
            .collect();

        let random_work_order_numbers = work_order_numbers.choose_multiple(
            &mut self.parameters.tactical_options.rng.clone(),
            self.parameters
                .tactical_options
                .number_of_removed_work_orders,
        );

        for work_order_number in random_work_order_numbers {
            self.unschedule_specific_work_order(*work_order_number)
                .with_context(|| {
                    format!(
                        "Could not unschedule tactical work order: {:?} on line: {}",
                        work_order_number,
                        line!(),
                    )
                })?;
        }
        Ok(())
    }

    fn algorithm_util_methods(&mut self) -> &mut Self::Algorithm {
        todo!()
    }
}

enum LoopState {
    Unscheduled,
    Scheduled,
    ReleasedFromTactical,
}
impl<Ss> Deref for TacticalAlgorithm<Ss>
where
    Ss: SystemSolutionTrait,
{
    type Target =
        Algorithm<TacticalSolution, TacticalParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<Ss> DerefMut for TacticalAlgorithm<Ss>
where
    Ss: SystemSolutionTrait,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<Ss> TacticalAlgorithm<Ss>
where
    TacticalSolution: Solution,
    TacticalParameters: Parameters,
    Ss: SystemSolutionTrait,
{
    fn update_loadings(
        &mut self,
        operation_solutions: &TacticalScheduledOperations,
        load_operation: LoadOperation,
    ) -> Result<()> {
        for operation in operation_solutions.0.values() {
            let resource = &operation.resource;
            for loadings in &operation.scheduled {
                let day = &loadings.0;
                let load = &loadings.1;
                let resource_loading = self.loading(resource, day);

                let new_load = match load_operation {
                    LoadOperation::Add => resource_loading + load,
                    LoadOperation::Sub => resource_loading - load,
                };
                *self.loading_mut(resource, day) = new_load;
            }
        }
        Ok(())
    }

    pub fn unschedule_specific_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
    ) -> Result<()> {
        let solution = self
            .solution
            .tactical_work_orders
            .0
            .insert(work_order_number, WhereIsWorkOrder::NotScheduled)
            .context("This means that the TacticalAlgorithm has been initialized wrong")?;

        match solution {
            WhereIsWorkOrder::Strategic => Ok(()),
            WhereIsWorkOrder::Tactical(operation_solutions) => {
                self.update_loadings(&operation_solutions.clone(), LoadOperation::Sub)
            }
            WhereIsWorkOrder::NotScheduled => bail!(
                "Unschedule should never be called on the {}. The state slipped through the tactical scheduling process",
                std::any::type_name_of_val(&solution)
            ),
        }
    }

    fn remaining_capacity(&self, resource: &Resources, day: &Day) -> Option<Work> {
        let remaining_capacity = self.capacity(resource, day) - self.loading(resource, day);

        if remaining_capacity <= Work::from(0.0) {
            None
        } else {
            Some(remaining_capacity)
        }
    }
}
fn determine_load(
    remaining_capacity: Work,
    operating_time: &Work,
    mut work_remaining: Work,
) -> Vec<Work> {
    let mut loadings = Vec::new();

    let first_day_load = match remaining_capacity.partial_cmp(operating_time) {
        Some(Ordering::Less) => remaining_capacity,
        Some(Ordering::Equal) => remaining_capacity,
        Some(Ordering::Greater) => *operating_time,
        None => panic!("remaining work and operating_time are not comparable. There is an error in the data initialization"),
    }.min(work_remaining);

    loadings.push(first_day_load);
    work_remaining -= first_day_load;

    while work_remaining > Work::from(0.0) {
        let load = *operating_time.min(&work_remaining);
        loadings.push(load);
        work_remaining -= load;
    }
    loadings
}

#[allow(dead_code)]
enum OperationDifference {
    SameDay,
    DiffDay,
}
impl<Ss>
    From<Algorithm<TacticalSolution, TacticalParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>>
    for TacticalAlgorithm<Ss>
where
    Ss: SystemSolutionTrait,
{
    fn from(
        value: Algorithm<
            TacticalSolution,
            TacticalParameters,
            PriorityQueue<WorkOrderNumber, u64>,
            Ss,
        >,
    ) -> Self {
        TacticalAlgorithm(value)
    }
}

#[cfg(test)]
pub mod tests {
    use ordinator_scheduling_environment::work_order::operation::Work;
    use ordinator_scheduling_environment::worker_environment::resources::Id;

    use crate::algorithm::determine_load;

    #[test]
    fn test_determine_load_1() {
        let remaining_capacity = Work::from(3.0);
        let operating_time = Work::from(5.0);
        let work_remaining = Work::from(10.0);

        // You need a `SharedSolution` to call this function... Does this even
        // make anysense? I think that it does but this is going to be really really
        // difficult. You should strive to make the code here
        // This sure is a horrible trait. I think that there is a way of making this
        // in a really good way
        //
        // You could also simply remove the function from the `Algorithm inplementation`

        let loadings = determine_load(remaining_capacity, &operating_time, work_remaining);

        assert_eq!(
            loadings,
            vec![Work::from(3.0), Work::from(5.0), Work::from(2.0)]
        );
    }

    #[test]
    fn test_determine_load_2() {
        let id = Id::default();

        let remaining_capacity = Work::from(3.0);
        let operating_time = Work::from(3.0);
        let work_remaining = Work::from(10.0);

        let loadings = determine_load(remaining_capacity, &operating_time, work_remaining);

        assert_eq!(
            loadings,
            vec![
                Work::from(3.0),
                Work::from(3.0),
                Work::from(3.0),
                Work::from(1.0)
            ]
        );
    }

    #[test]
    fn test_work_min() {
        let operating_time = Work::from(3.0);
        let work_remaining = Work::from(10.0);

        let min_work = operating_time.min(work_remaining);

        assert_eq!(min_work, Work::from(3.0));

        let operating_time = Work::from(12.0);
        let work_remaining = Work::from(10.0);

        let min_work = operating_time.min(work_remaining);

        assert_eq!(min_work, Work::from(10.0));
    }

    // You should test all this in the right order. I think that...
    // QUESTION
    // Is it correct of you to move this into the integration testing? Yes
    // absolutely. I do not see anyother way, as the `objective value` may
    // always be dependent on the other `Solution`s.
    // GOOD a decision was made here.
}
