pub mod assert_functions;
pub mod tactical_parameters;
pub mod tactical_solution;

use anyhow::{bail, Context, Result};
use arc_swap::Guard;
use assert_functions::TacticalAssertions;
use chrono::TimeDelta;
use priority_queue::PriorityQueue;
use rand::seq::IndexedRandom;
use shared_types::{
    scheduling_environment::{
        time_environment::day::Day,
        work_order::{
            operation::{ActivityNumber, Work},
            WorkOrderNumber,
        },
        worker_environment::resources::Resources,
        SchedulingEnvironment,
    },
    tactical::{
        TacticalObjectiveValue, TacticalRequestMessage, TacticalResources, TacticalResponseMessage,
    },
    LoadOperation,
};
use std::collections::HashMap;
use std::{
    cmp::Ordering,
    sync::{Arc, MutexGuard},
};
use tactical_parameters::{OperationParameter, TacticalParameter, TacticalParameters};
use tactical_solution::OperationSolution;
use tracing::{event, Level};

use crate::agents::{
    traits::{ActorBasedLargeNeighborhoodSearch, ObjectiveValueType},
    Algorithm, TacticalScheduledOperations, TacticalSolution, WhereIsWorkOrder,
};

use shared_types::scheduling_environment::work_order::WorkOrder;

use super::TacticalOptions;

// FIX
// Move the `tactical_days` into the parameters.
// QUESTION
// Does that feel correct to say? Yes it does!

impl Algorithm<TacticalSolution, TacticalParameters, PriorityQueue<WorkOrderNumber, u64>> {
    pub fn capacity(&self, resource: &Resources, day: &Day) -> &Work {
        self.parameters
            .tactical_capacity
            .resources
            .get(resource)
            .unwrap()
            .get(day)
    }

    pub fn capacity_mut(&mut self, resource: &Resources, day: &Day) -> &mut Work {
        self.parameters
            .tactical_capacity
            .resources
            .get_mut(resource)
            .unwrap()
            .day_mut(day)
    }

    pub fn loading(&self, resource: &Resources, day: &Day) -> &Work {
        self.solution
            .tactical_loadings
            .resources
            .get(resource)
            .unwrap()
            .get(day)
    }

    pub fn loading_mut(&mut self, resource: &Resources, day: &Day) -> &mut Work {
        self.solution
            .tactical_loadings
            .resources
            .get_mut(resource)
            .unwrap()
            .day_mut(day)
    }

    pub fn create_and_insert_tactical_parameter_and_initialize_solution(
        &mut self,
        work_order: &WorkOrder,
    ) {
        let mut tactical_parameter = TacticalParameter::new(
            work_order.main_work_center,
            HashMap::new(),
            work_order.work_order_weight(),
            work_order.relations().clone(),
            work_order.work_order_dates.earliest_allowed_start_date,
        );

        for (activity, operation) in work_order.operations() {
            let optimized_operation = OperationParameter::new(
                *work_order.work_order_number(),
                operation.number(),
                operation.duration().unwrap(),
                operation.operating_time().unwrap(),
                operation.work_remaining().unwrap(),
                *operation.resource(),
            );
            tactical_parameter
                .tactical_operation_parameters
                .insert(*activity, optimized_operation);
        }
        self.parameters
            .tactical_work_orders
            .insert(*work_order.work_order_number(), tactical_parameter);
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

    pub(crate) fn create_parameters(
        &mut self,
        scheduling_environment_guard: &MutexGuard<SchedulingEnvironment>,
        asset: &shared_types::Asset,
    ) {
        let work_orders = scheduling_environment_guard
            .work_orders
            .work_orders_by_asset(asset);

        self.load_shared_solution();

        for (work_order_number, work_order) in work_orders {
            self.create_and_insert_tactical_parameter_and_initialize_solution(work_order);
            self.solution
                .tactical_scheduled_work_orders
                .0
                .insert(*work_order_number, WhereIsWorkOrder::NotScheduled);
        }
        self.make_atomic_pointer_swap();
    }

    fn determine_tardiness(&mut self, tactical_objective_value: &mut TacticalObjectiveValue) {
        let mut objective_value_from_tardiness = 0;
        for (work_order_number, _solution) in self
            .solution
            .tactical_scheduled_work_orders
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
                .strategic
                .strategic_scheduled_work_orders
                .get(work_order_number)
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

impl ActorBasedLargeNeighborhoodSearch for TacticalAlgorithm {
    type MessageRequest = TacticalRequestMessage;
    type MessageResponse = TacticalResponseMessage;
    type Solution = TacticalSolution;
    type ObjectiveValue = TacticalObjectiveValue;
    type Options = TacticalOptions;

    fn clone_algorithm_solution(&self) -> Self::Solution {
        self.solution.clone()
    }

    fn incorporate_shared_state(&mut self) -> Result<bool> {
        Ok(true)
    }

    fn swap_solution(&mut self, solution: Self::Solution) {
        self.solution = solution;
    }

    fn make_atomic_pointer_swap(&self) {
        // Performance enhancements:
        // * COW:
        //      #[derive(Clone)]
        //      struct SharedSolution<'a> {
        //          tactical: Cow<'a, TacticalSolution>,
        //          // other fields...
        //      }
        //
        // * Reuse the old SharedSolution, cloning only the fields that are needed.
        //     let shared_solution = Arc::new(SharedSolution {
        //             tactical: self.solution.clone(),
        //             // Copy over other fields without cloning
        //             ..(**old).clone()
        //         });
        self.arc_swap_shared_solution.0.rcu(|old| {
            let mut shared_solution = (**old).clone();
            shared_solution.tactical = self.solution.clone();
            Arc::new(shared_solution)
        });
    }

    fn update_objective_value(&mut self, objective_value: Self::ObjectiveValue) {
        self.solution.objective_value = objective_value;
    }

    fn calculate_objective_value(&mut self) -> Result<ObjectiveValueType<Self::ObjectiveValue>> {
        let mut tactical_objective_value =
            TacticalObjectiveValue::new(0, (1, u64::MAX), (1000000000, u64::MAX));

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
        for (work_order_number, solution) in &self.solution.tactical_scheduled_work_orders.0 {
            let tactical_parameter = self
                .parameters()
                .get(work_order_number)
                .expect("TacticalParameter should ALWAYS be available for a TacticalSolution");

            // All the work orders that does not have a solution gets pushed to the queue.
            if matches!(solution, WhereIsWorkOrder::NotScheduled) {
                self.priority_queue
                    .push(*work_order_number, tactical_parameter.weight);
            }
        }

        let mut start_day_index = 0;

        let mut loop_state: LoopState = LoopState::Unscheduled;

        let mut current_work_order_number = match self.priority_queue.pop() {
            Some((work_order_number, _)) => work_order_number,
            None => return Ok(()),
        };

        let mut counter = 0;
        // The issue is that the code here is running a lot of iterations. What should we
        // do about this? I am not really sure! I thi
        'back_to_loop_state_handle: loop {
            counter += 1;

            event!(
                Level::DEBUG,
                main_loop_counter = counter,
                start_day_index = start_day_index,
                priority_queue_len = self.priority_queue.len(),
            );
            let tactical_parameter = match loop_state {
                LoopState::Unscheduled => {
                    start_day_index += 1;
                    self.parameters().get(&current_work_order_number).unwrap()
                }
                LoopState::Scheduled => {
                    start_day_index = 0;

                    current_work_order_number = match self.priority_queue.pop() {
                        Some((work_order_number, _)) => work_order_number,
                        None => {
                            event!(Level::INFO, "main_loop break");
                            break;
                        }
                    };

                    self.parameters().get(&current_work_order_number).unwrap()
                }
                LoopState::ReleasedFromTactical => {
                    self.solution
                        .release_from_solution(&current_work_order_number);

                    start_day_index = 0;

                    current_work_order_number = match self.priority_queue.pop() {
                        Some((work_order_number, _)) => work_order_number,
                        None => {
                            event!(Level::INFO, "main_loop break");
                            break;
                        }
                    };

                    self.parameters().get(&current_work_order_number).unwrap()
                }
            };

            let mut operation_solutions = TacticalScheduledOperations::default();

            let mut all_days = self.tactical_days.clone();

            let allowed_starting_days: Vec<&Day> = self
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

                let loadings = self.determine_load(
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
    fn unschedule(&mut self, options: &mut Self::Options) -> Result<()> {
        let work_order_numbers: Vec<WorkOrderNumber> = self
            .solution
            .tactical_scheduled_work_orders
            .0
            .clone()
            .into_keys()
            .collect();

        let random_work_order_numbers = work_order_numbers
            .choose_multiple(&mut options.rng, options.number_of_removed_work_orders);

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
}

enum LoopState {
    Unscheduled,
    Scheduled,
    ReleasedFromTactical,
}

impl Algorithm<TacticalSolution, TacticalParameters, PriorityQueue<WorkOrderNumber, u64>> {
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
            .tactical_scheduled_work_orders
            .0
            .insert(work_order_number, WhereIsWorkOrder::NotScheduled)
            .context("This means that the TacticalAlgorithm has been initialized wrong")?;

        match solution {
            WhereIsWorkOrder::Strategic => {
                Ok(())

            }
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

    fn determine_load(
        &self,
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

    pub fn parameters_mut(&mut self) -> &mut HashMap<WorkOrderNumber, TacticalParameter> {
        &mut self.parameters.tactical_work_orders
    }

    pub fn parameters(&self) -> &HashMap<WorkOrderNumber, TacticalParameter> {
        &self.parameters.tactical_work_orders
    }
}

#[allow(dead_code)]
enum OperationDifference {
    SameDay,
    DiffDay,
}

#[cfg(test)]
pub mod tests {
    use std::{collections::HashMap, str::FromStr};

    use chrono::{Days, NaiveDate};
    use shared_types::{
        scheduling_environment::{
            work_order::{
                operation::{ActivityNumber, Work},
                WorkOrderNumber,
            },
            worker_environment::resources::Resources,
        },
        tactical::TacticalResources,
    };
    use strum::IntoEnumIterator;

    use crate::agents::{
        tactical_agent::algorithm::OperationSolution, traits::ActorBasedLargeNeighborhoodSearch,
        ArcSwapSharedSolution, TacticalScheduledOperations, WhereIsWorkOrder,
    };

    use super::{Day, OperationParameter, TacticalParameter};
    use shared_types::scheduling_environment::time_environment::period::Period;

    #[test]
    fn test_determine_load_1() {
        let tactical_algorithm = super::TacticalAlgorithm::new(
            vec![],
            super::TacticalResources::new(HashMap::new()),
            super::TacticalResources::new(HashMap::new()),
            ArcSwapSharedSolution::default().into(),
        );

        let remaining_capacity = Work::from(3.0);
        let operating_time = Work::from(5.0);
        let work_remaining = Work::from(10.0);

        let loadings =
            tactical_algorithm.determine_load(remaining_capacity, &operating_time, work_remaining);

        assert_eq!(
            loadings,
            vec![Work::from(3.0), Work::from(5.0), Work::from(2.0)]
        );
    }

    #[test]
    fn test_determine_load_2() {
        let tactical_algorithm = super::TacticalAlgorithm::new(
            vec![],
            super::TacticalResources::new(HashMap::new()),
            super::TacticalResources::new(HashMap::new()),
            ArcSwapSharedSolution::default().into(),
        );

        let remaining_capacity = Work::from(3.0);
        let operating_time = Work::from(3.0);
        let work_remaining = Work::from(10.0);

        let loadings =
            tactical_algorithm.determine_load(remaining_capacity, &operating_time, work_remaining);

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

    #[test]
    fn test_calculate_objective_value() {
        let work_order_number = WorkOrderNumber(2100000001);
        let activity_number = ActivityNumber(1);
        let first_period = Period::from_str("2024-W13-14").unwrap();

        let tactical_days = |number_of_days: u32| -> Vec<Day> {
            let mut days: Vec<Day> = Vec::new();
            let mut date = first_period.start_date().to_owned();
            for day_index in 0..number_of_days {
                days.push(Day::new(day_index as usize, date.to_owned()));
                date = date.checked_add_days(Days::new(1)).unwrap();
            }
            days
        };

        let mut tactical_algorithm = super::TacticalAlgorithm::new(
            tactical_days(56),
            super::TacticalResources::new(HashMap::new()),
            super::TacticalResources::new(HashMap::new()),
            ArcSwapSharedSolution::default().into(),
        );

        let operation_parameter = OperationParameter::new(
            work_order_number,
            1,
            Work::from(1.0),
            Work::from(1.0),
            Work::from(1.0),
            Resources::MtnMech,
        );

        let operation_solution = OperationSolution::new(
            vec![(
                tactical_algorithm.tactical_days[27].clone(),
                Work::from(1.0),
            )],
            Resources::MtnMech,
            operation_parameter.number,
            operation_parameter.work_remaining,
            work_order_number,
            activity_number,
        );

        let mut operation_parameters = HashMap::new();
        operation_parameters.insert(activity_number, operation_parameter);

        let mut operation_solutions = HashMap::new();
        operation_solutions.insert(ActivityNumber(1), operation_solution);

        let optimized_tactical_work_order = TacticalParameter::new(
            Resources::MtnMech,
            operation_parameters,
            10,
            vec![],
            NaiveDate::from_ymd_opt(2024, 10, 10).unwrap(),
        );

        tactical_algorithm
            .parameters_mut()
            .insert(work_order_number, optimized_tactical_work_order);

        tactical_algorithm.calculate_objective_value().unwrap();

        // assert_eq!(tactical_algorithm.objective_value().0, 270);
    }

    #[test]
    fn test_schedule_1() {
        let work_order_number = WorkOrderNumber(2100000001);
        let first_period = Period::from_str("2024-W13-14").unwrap();

        let tactical_days = |number_of_days: u32| -> Vec<Day> {
            let mut days: Vec<Day> = Vec::new();
            let mut date = first_period.start_date().to_owned();
            for day_index in 0..number_of_days {
                days.push(Day::new(day_index as usize, date.to_owned()));
                date = date.checked_add_days(Days::new(1)).unwrap();
            }
            days
        };

        let mut tactical_algorithm = super::TacticalAlgorithm::new(
            tactical_days(56),
            TacticalResources::new_from_data(
                Resources::iter().collect(),
                tactical_days(56),
                Work::from(0.0),
            ),
            TacticalResources::new_from_data(
                Resources::iter().collect(),
                tactical_days(56),
                Work::from(0.0),
            ),
            ArcSwapSharedSolution::default().into(),
        );

        let operation_parameter = OperationParameter::new(
            work_order_number,
            1,
            Work::from(1.0),
            Work::from(1.0),
            Work::from(1.0),
            Resources::MtnMech,
        );

        let mut tactical_operation_parameters = HashMap::new();
        tactical_operation_parameters.insert(ActivityNumber(1), operation_parameter);

        let tactical_work_order_parameter = TacticalParameter::new(
            Resources::MtnMech,
            tactical_operation_parameters,
            10,
            vec![],
            NaiveDate::from_ymd_opt(2024, 10, 10).unwrap(),
        );

        tactical_algorithm
            .parameters_mut()
            .insert(work_order_number, tactical_work_order_parameter);

        let activity_number = ActivityNumber(0);

        let mut tactical_activities = TacticalScheduledOperations::default();

        tactical_activities.0.insert(
            activity_number,
            OperationSolution::new(
                vec![],
                Resources::MtnMech,
                1,
                Work::from(0.0),
                work_order_number,
                activity_number,
            ),
        );

        tactical_algorithm
            .solution
            .tactical_scheduled_work_orders
            .0
            .insert(
                work_order_number,
                WhereIsWorkOrder::Tactical(tactical_activities),
            );

        tactical_algorithm.schedule().unwrap();

        let scheduled_date = tactical_algorithm
            .solution
            .tactical_scheduled_days(&work_order_number, &ActivityNumber(0));

        assert!(scheduled_date.is_ok());
    }

    #[test]
    fn test_schedule_2() {
        let work_order_number = WorkOrderNumber(2100000010);
        let activity_number = ActivityNumber(1);
        let first_period = Period::from_str("2024-W13-14").unwrap();

        let tactical_days = |number_of_days: u32| -> Vec<Day> {
            let mut days: Vec<Day> = Vec::new();
            let mut date = first_period.start_date().to_owned();
            for day_index in 0..number_of_days {
                days.push(Day::new(day_index as usize, date.to_owned()));
                date = date.checked_add_days(Days::new(1)).unwrap();
            }
            days
        };

        let mut tactical_algorithm = super::TacticalAlgorithm::new(
            tactical_days(56),
            super::TacticalResources::new_from_data(
                Resources::iter().collect(),
                tactical_days(56),
                Work::from(100.0),
            ),
            super::TacticalResources::new_from_data(
                Resources::iter().collect(),
                tactical_days(56),
                Work::from(0.0),
            ),
            ArcSwapSharedSolution::default().into(),
        );

        let mut tactical_activities = TacticalScheduledOperations::default();

        tactical_activities.0.insert(
            activity_number,
            OperationSolution::new(
                vec![],
                Resources::MtnMech,
                1,
                Work::from(0.0),
                work_order_number,
                activity_number,
            ),
        );

        tactical_algorithm
            .solution
            .tactical_scheduled_work_orders
            .0
            .insert(
                work_order_number,
                WhereIsWorkOrder::Tactical(tactical_activities),
            );

        let operation_parameter = OperationParameter::new(
            work_order_number,
            1,
            Work::from(1.0),
            Work::from(1.0),
            Work::from(1.0),
            Resources::MtnMech,
        );

        let mut operation_parameters = HashMap::new();
        operation_parameters.insert(ActivityNumber(1), operation_parameter);

        let optimized_tactical_work_order = TacticalParameter::new(
            Resources::MtnMech,
            operation_parameters,
            10,
            vec![],
            NaiveDate::from_ymd_opt(2024, 10, 10).unwrap(),
        );

        tactical_algorithm
            .parameters_mut()
            .insert(work_order_number, optimized_tactical_work_order);

        tactical_algorithm.schedule().unwrap();

        let scheduled_date = tactical_algorithm
            .solution
            .tactical_scheduled_days(&work_order_number, &ActivityNumber(1));

        assert!(scheduled_date.is_ok());
    }
}
