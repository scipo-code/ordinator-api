pub mod assert_functions;
pub mod tactical_parameters;
pub mod tactical_solution;

use anyhow::{bail, Context, Result};
use arc_swap::Guard;
use chrono::{NaiveDate, TimeDelta};
use priority_queue::PriorityQueue;
use rand::seq::SliceRandom;
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
        tactical_resources_message::TacticalResourceRequest,
        tactical_response_resources::TacticalResponseResources,
        tactical_response_scheduling::TacticalResponseScheduling,
        tactical_response_time::TacticalResponseTime,
        tactical_scheduling_message::TacticalSchedulingRequest,
        tactical_time_message::TacticalTimeRequest, TacticalObjectiveValue, TacticalResources,
    },
    LoadOperation,
};
use std::collections::HashMap;
use std::{
    cmp::Ordering,
    sync::{Arc, MutexGuard},
};
use tactical_parameters::{OperationParameter, TacticalParameter, TacticalParameters};
use tactical_solution::TacticalOperation;
use tracing::{event, instrument, Level};

use crate::agents::{
    traits::LargeNeighborHoodSearch, ArcSwapSharedSolution, SharedSolution, TacticalSolution,
};

use shared_types::scheduling_environment::work_order::WorkOrder;

pub struct TacticalAlgorithm {
    pub arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
    pub loaded_shared_solution: Guard<Arc<SharedSolution>>,
    pub tactical_solution: TacticalSolution,
    pub tactical_parameters: TacticalParameters,
    pub priority_queue: PriorityQueue<WorkOrderNumber, u64>,
    pub tactical_days: Vec<Day>,
}

impl TacticalAlgorithm {
    pub fn new(
        tactical_days: Vec<Day>,
        capacity: TacticalResources,
        loading: TacticalResources,
        strategic_tactical_solution_arc_swap: Arc<ArcSwapSharedSolution>,
    ) -> Self {
        let loaded_shared_solution = strategic_tactical_solution_arc_swap.0.load();
        let mut tactical_algorithm = TacticalAlgorithm {
            arc_swap_shared_solution: strategic_tactical_solution_arc_swap,
            loaded_shared_solution,
            tactical_solution: TacticalSolution::default(),
            tactical_parameters: TacticalParameters::default(),
            priority_queue: PriorityQueue::new(),
            tactical_days,
        };

        tactical_algorithm.tactical_solution.tactical_loadings = loading;
        tactical_algorithm.tactical_parameters.tactical_capacity = capacity;
        tactical_algorithm
    }

    pub fn capacity(&self, resource: &Resources, day: &Day) -> &Work {
        self.tactical_parameters
            .tactical_capacity
            .resources
            .get(resource)
            .unwrap()
            .get(day)
    }

    pub fn capacity_mut(&mut self, resource: &Resources, day: &Day) -> &mut Work {
        self.tactical_parameters
            .tactical_capacity
            .resources
            .get_mut(resource)
            .unwrap()
            .get_mut(day)
    }

    pub fn loading(&self, resource: &Resources, day: &Day) -> &Work {
        self.tactical_solution
            .tactical_loadings
            .resources
            .get(resource)
            .unwrap()
            .get(day)
    }

    pub fn loading_mut(&mut self, resource: &Resources, day: &Day) -> &mut Work {
        self.tactical_solution
            .tactical_loadings
            .resources
            .get_mut(resource)
            .unwrap()
            .get_mut(day)
    }

    pub fn create_tactical_parameter(
        &mut self,
        work_order: &WorkOrder,
        earliest_allowed_start_date: NaiveDate,
    ) {
        let mut tactical_parameter = TacticalParameter::new(
            work_order.main_work_center.clone(),
            HashMap::new(),
            work_order.work_order_weight(),
            work_order.relations().clone(),
            earliest_allowed_start_date,
        );

        for (activity, operation) in work_order.operations() {
            let optimized_operation = OperationParameter::new(
                *work_order.work_order_number(),
                operation.number(),
                operation.duration().clone().unwrap(),
                operation.operating_time().clone().unwrap(),
                operation.work_remaining().clone().unwrap(),
                operation.resource().clone(),
            );
            tactical_parameter
                .tactical_operation_parameters
                .insert(*activity, optimized_operation);
        }
        self.tactical_parameters_mut()
            .insert(*work_order.work_order_number(), tactical_parameter);
    }

    pub fn unschedule_random_work_orders(
        &mut self,
        rng: &mut impl rand::Rng,
        number_of_work_orders: u32,
    ) -> Result<()> {
        let work_order_numbers: Vec<WorkOrderNumber> = self
            .tactical_solution
            .tactical_days
            .clone()
            .into_keys()
            .collect();

        let random_work_order_numbers =
            work_order_numbers.choose_multiple(rng, number_of_work_orders as usize);

        for work_order_number in random_work_order_numbers {
            self.unschedule(*work_order_number).with_context(|| {
                format!(
                    "Could not unschedule tactical work order: {:?} on line: {}",
                    work_order_number,
                    line!(),
                )
            })?;
        }
        Ok(())
    }

    fn determine_aggregate_excess(&mut self) -> u64 {
        let mut objective_value_from_excess = 0;
        for resource in self.tactical_parameters.tactical_capacity.resources.keys() {
            for day in self.tactical_days.clone() {
                let excess_capacity = self.loading(resource, &day) - self.capacity(resource, &day);

                if excess_capacity > Work::from(0.0) {
                    objective_value_from_excess += excess_capacity.to_f64() as u64;
                }
            }
        }

        objective_value_from_excess
    }

    pub(crate) fn create_tactical_parameters(
        &mut self,
        scheduling_environment_guard: &MutexGuard<SchedulingEnvironment>,
        asset: &shared_types::Asset,
    ) {
        let work_orders = scheduling_environment_guard
            .work_orders
            .work_orders_by_asset(asset);

        self.load_shared_solution();

        for (work_order_number, work_order) in work_orders {
            self.create_tactical_parameter(
                work_order,
                work_order.work_order_dates.earliest_allowed_start_date,
            );
            self.tactical_solution
                .tactical_days
                .insert(*work_order_number, None);
        }
        self.make_atomic_pointer_swap();
    }

    pub(crate) fn make_atomic_pointer_swap(&self) {
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
        //             tactical: self.tactical_solution.clone(),
        //             // Copy over other fields without cloning
        //             ..(**old).clone()
        //         });
        self.arc_swap_shared_solution.0.rcu(|old| {
            let mut shared_solution = (**old).clone();
            shared_solution.tactical = self.tactical_solution.clone();
            Arc::new(shared_solution)
        });
    }

    pub fn load_shared_solution(&mut self) {
        self.loaded_shared_solution = self.arc_swap_shared_solution.0.load();
    }

    pub(crate) fn objective_value(&self) -> TacticalObjectiveValue {
        self.tactical_solution.objective_value.clone()
    }
}

impl LargeNeighborHoodSearch for TacticalAlgorithm {
    type BetterSolution = TacticalObjectiveValue;
    type SchedulingRequest = TacticalSchedulingRequest;
    type SchedulingResponse = TacticalResponseScheduling;
    type ResourceRequest = TacticalResourceRequest;
    type ResourceResponse = TacticalResponseResources;
    type TimeRequest = TacticalTimeRequest;
    type TimeResponse = TacticalResponseTime;

    type SchedulingUnit = WorkOrderNumber;

    fn calculate_objective_value(&mut self) -> Self::BetterSolution {
        let mut objective_value_from_tardiness = 0;
        for (work_order_number, _tactical_solution) in self
            .tactical_solution
            .tactical_days
            .iter()
            .filter(|(_, ts)| ts.is_some())
        {
            let tactical_parameter = self.tactical_parameters().get(work_order_number).unwrap();
            let period_start_date = match &self
                .loaded_shared_solution
                .strategic
                .strategic_periods
                .get(work_order_number)
                .expect("The strategic and tactical solution should have the same WorkOrderNumber's available at all times")

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
                .tactical_solution
                .tactical_day(work_order_number, last_activity)
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

        // Calculate penalty for exceeding the capacity
        let objective_value_from_excess = 1000000000 * self.determine_aggregate_excess();
        self.tactical_solution.objective_value.0 =
            objective_value_from_tardiness + objective_value_from_excess;
        event!(
            Level::WARN,
            objective_value_from_excess = ?objective_value_from_excess,
            objective_value_from_tardiness = ?objective_value_from_tardiness,
            tactical_objective_value = ?self.tactical_solution.objective_value
        );

        self.tactical_solution.objective_value.clone()
    }

    fn schedule(&mut self) -> Result<()> {
        for (work_order_number, tactical_solution) in &self.tactical_solution.tactical_days {
            let tactical_parameter = self
                .tactical_parameters()
                .get(work_order_number)
                .expect("TacticalParameter should ALWAYS be available for a TacticalSolution");
            match tactical_solution {
                None => {
                    self.priority_queue
                        .push(*work_order_number, tactical_parameter.weight);
                }
                Some(_) => (),
            }
        }

        let mut start_day_index = 0;

        let mut loop_state: LoopState = LoopState::Unscheduled;

        event!(
            Level::INFO,
            scheduled_work_orders = self
                .tactical_solution
                .tactical_days
                .iter()
                .filter(|ele| ele.1.is_some())
                .count(),
            all_work_orders = self.tactical_solution.tactical_days.len()
        );
        let mut current_work_order_number = match self.priority_queue.pop() {
            Some((work_order_number, _)) => work_order_number,
            None => return Ok(()),
        };

        'main: loop {
            let mut operation_solutions = HashMap::<ActivityNumber, TacticalOperation>::new();

            let tactical_parameter = match loop_state {
                LoopState::Unscheduled => self
                    .tactical_parameters()
                    .get(&current_work_order_number)
                    .unwrap(),
                LoopState::Scheduled => {
                    start_day_index = 0;

                    current_work_order_number = match self.priority_queue.pop() {
                        Some((work_order_number, _)) => work_order_number,
                        None => {
                            event!(
                                Level::INFO,
                                scheduled_work_orders = self
                                    .tactical_solution
                                    .tactical_days
                                    .iter()
                                    .filter(|ele| ele.1.is_some())
                                    .count(),
                                all_work_orders = self.tactical_solution.tactical_days.len()
                            );

                            break;
                        }
                    };

                    self.tactical_parameters()
                        .get(&current_work_order_number)
                        .unwrap()
                }
                LoopState::ReleasedFromTactical => {
                    // What should happen in here? I think that we need to create something that
                    // will allow us to make the code run with
                    self.tactical_solution
                        .tactical_remove_work_order(&current_work_order_number);
                    // So what should happen when this occurs? I think that the best approach here...
                    // There are two things that can happen! Either we set the SchedulePeriod to None
                    // or the scheduled_days to None, or both.. I think that there should always be
                    // complete correspondence between the tactical and the Strategic here. What is
                    // the best way of ensuring this? I think that it is to make the

                    // When it is released it should go to the strategic agent! Yes! But is there
                    // a way that we can reuse the state of the Period so that the Tactical and
                    // Strategic does not both have a Period? Also, the tactical should always be
                    // able to pull in work orders from the strategic and schedule them as well as
                    // they possible can.
                    start_day_index = 0;

                    current_work_order_number = match self.priority_queue.pop() {
                        Some((work_order_number, _)) => work_order_number,
                        None => {
                            break;
                        }
                    };

                    self.tactical_parameters()
                        .get(&current_work_order_number)
                        .unwrap()
                }
            };

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
                    continue 'main;
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
                let mut activity_load = Vec::<(Day, Work)>::new();

                let operation_parameters = tactical_parameter
                    .tactical_operation_parameters
                    .get(activity)
                    .expect("The work order should always have its corresponding parameters");

                let resource = operation_parameters.resource.clone();

                let current_day_peek = match current_day.peek() {
                    Some(day) => day,
                    None => {
                        event!(Level::DEBUG,
                            current_work_order_number = ?current_work_order_number,
                            operation_parameters = ?operation_parameters,
                            operation_solutions = ?operation_solutions,
                            "Work order did not fit in the tactical schedule"
                        );
                        // Why do we break here? Because we ran out of days!

                        // loop_state
                        loop_state = LoopState::ReleasedFromTactical;
                        continue 'main;
                    }
                };

                let first_day_remaining_capacity =
                    match self.remaining_capacity(&resource, current_day_peek) {
                        Some(remaining_capacity) => remaining_capacity,
                        None => {
                            start_day_index += 1;
                            loop_state = LoopState::Unscheduled;
                            continue 'main;
                        }
                    };

                let loadings = self.determine_load(
                    first_day_remaining_capacity,
                    &operation_parameters.operating_time,
                    operation_parameters.work_remaining.clone(),
                );

                for load in loadings {
                    let day = match current_day.peek() {
                        Some(day) => (*day).clone(),
                        None => {
                            event!(Level::DEBUG,
                                current_work_order_number = ?current_work_order_number,
                                operation_parameters = ?operation_parameters,
                                operation_solutions = ?operation_solutions,
                                "Work order did not fit in the tactical schedule"
                            );
                            break;
                        }
                    };
                    activity_load.push((day, load));

                    current_day.next();

                    let peek_next_day = current_day.peek();
                    let current_day = match peek_next_day {
                        Some(next_day) => next_day,
                        None => {
                            event!(Level::DEBUG,
                                current_work_order_number = ?current_work_order_number,
                                operation_parameters = ?operation_parameters,
                                operation_solutions = ?operation_solutions,
                                "Work order did not fit in the tactical schedule"
                            );
                            // brake should schedule what is possible and cut the rest out.
                            break;
                        }
                    };

                    // This is the problem! It says that if the start day index is the last day of the schedule
                    // then we should just go ahead, act like nothing and schedule it
                    if self.remaining_capacity(&resource, current_day).is_none() {
                        start_day_index += 1;
                        loop_state = LoopState::Unscheduled;
                        continue 'main;
                    };
                }

                let operation_solution = TacticalOperation::new(
                    activity_load,
                    resource,
                    operation_parameters.number,
                    operation_parameters.work_remaining.clone(),
                    current_work_order_number,
                    *activity,
                );
                operation_solutions.insert(*activity, operation_solution);
            }
            event!(
                Level::DEBUG,
                "Tactical {:?} has been scheduled starting on day {}",
                current_work_order_number,
                start_day.day_index()
            );
            self.update_loadings(&operation_solutions, LoadOperation::Add);
            // panic!();
            loop_state = LoopState::Scheduled;

            // This is always called. For each work order... As long as the days are inside of the
            // period.
            self.tactical_solution
                .tactical_insert_work_order(current_work_order_number, operation_solutions);

            // TODO: This should be removed. The parameters are not guiding.
            // the solutions are! That is important to remember!
            if self
                .tactical_parameters_mut()
                .get_mut(&current_work_order_number)
                .is_none()
            {
                event!(Level::ERROR, unscheduled_work_order = ?current_work_order_number);
                panic!("Unscheduled work order got through the schedule function");
            }
        }
        Ok(())
    }

    fn unschedule(&mut self, work_order_number: Self::SchedulingUnit) -> Result<()> {
        let tactical_solution = self
            .tactical_solution
            .tactical_days
            .get_mut(&work_order_number)
            .context(
                "This means that the TacticalAlgorithm has been initialized wrong".to_string(),
            )?;

        match tactical_solution.take() {
            Some(operation_solutions) => {
                self.update_loadings(&operation_solutions, LoadOperation::Sub);
                Ok(())
            }
            None => {
                event!(
                    Level::DEBUG,
                    "Work order {:?} was not scheduled before leaving the tactical schedule",
                    work_order_number
                );
                Ok(())
            }
        }
    }

    fn update_scheduling_state(
        &mut self,
        _scheduling_message: Self::SchedulingRequest,
    ) -> Result<Self::SchedulingResponse> {
        Ok(TacticalResponseScheduling {})
        // This is where the algorithm will update the scheduling state.
    }

    fn update_time_state(
        &mut self,
        _time_message: Self::TimeRequest,
    ) -> Result<Self::TimeResponse> {
        // This is where the algorithm will update the time state.
        Ok(TacticalResponseTime {})
    }

    #[instrument(level = "info", skip(self))]
    fn update_resources_state(
        &mut self,
        resource_message: Self::ResourceRequest,
    ) -> Result<Self::ResourceResponse> {
        match resource_message {
            TacticalResourceRequest::SetResources(resources) => {
                // The resources should be initialized together with the Agent itself
                let mut count = 0;
                for (resource, days) in resources.resources {
                    for (day, capacity) in days.days {
                        let day: Day = match self.tactical_days.iter().find(|d| **d == day) {
                            Some(day) => {
                                count += 1;
                                day.clone()
                            }
                            None => {
                                bail!("Day not found in the tactical days".to_string(),);
                            }
                        };

                        *self.capacity_mut(&resource, &day) = capacity;
                    }
                }
                Ok(TacticalResponseResources::UpdatedResources(count))
            }
            TacticalResourceRequest::GetLoadings {
                days_end: _,
                select_resources: _,
            } => {
                let loadings = self.tactical_solution.tactical_loadings.clone();

                event!(Level::DEBUG,loadings = ?loadings);
                let tactical_response_resources = TacticalResponseResources::Loading(loadings);
                Ok(tactical_response_resources)
            }
            TacticalResourceRequest::GetCapacities {
                days_end: _,
                select_resources: _,
            } => {
                let capacities = self.tactical_parameters.tactical_capacity.clone();

                let tactical_response_resources = TacticalResponseResources::Capacity(capacities);

                Ok(tactical_response_resources)
            }
            TacticalResourceRequest::GetPercentageLoadings {
                days_end: _,
                resources: _,
            } => {
                let capacities = &self.tactical_parameters.tactical_capacity;
                let loadings = &self.tactical_solution.tactical_loadings;

                let tactical_response_resources =
                    TacticalResponseResources::Percentage((capacities.clone(), loadings.clone()));
                Ok(tactical_response_resources)
            }
        }
    }
}

enum LoopState {
    Unscheduled,
    Scheduled,
    ReleasedFromTactical,
}

impl TacticalAlgorithm {
    fn update_loadings(
        &mut self,
        operation_solutions: &HashMap<ActivityNumber, TacticalOperation>,
        load_operation: LoadOperation,
    ) {
        for operation in operation_solutions.values() {
            let resource = operation.resource.clone();
            for loadings in operation.scheduled.clone() {
                let day = loadings.0;
                let load = loadings.1;
                let resource_loading = self.loading(&resource, &day);

                let new_load = match load_operation {
                    LoadOperation::Add => resource_loading + &load,
                    LoadOperation::Sub => resource_loading - &load,
                };
                *self.loading_mut(&resource, &day) = new_load;
            }
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
            Some(Ordering::Greater) => operating_time.clone(),
            None => panic!("remaining work and operating_time are not comparable. There is an error in the data initialization"),
        }.min(work_remaining.clone());

        loadings.push(first_day_load.clone());
        work_remaining -= first_day_load;

        while work_remaining > Work::from(0.0) {
            let load = operating_time.clone().min(work_remaining.clone());
            loadings.push(load.clone());
            work_remaining -= load;
        }
        loadings
    }

    pub fn tactical_parameters_mut(&mut self) -> &mut HashMap<WorkOrderNumber, TacticalParameter> {
        &mut self.tactical_parameters.tactical_work_orders
    }

    pub fn tactical_parameters(&self) -> &HashMap<WorkOrderNumber, TacticalParameter> {
        &self.tactical_parameters.tactical_work_orders
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
        tactical_agent::algorithm::TacticalOperation, traits::LargeNeighborHoodSearch,
        ArcSwapSharedSolution,
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

        let operation_solution = TacticalOperation::new(
            vec![(
                tactical_algorithm.tactical_days[27].clone(),
                Work::from(1.0),
            )],
            Resources::MtnMech,
            operation_parameter.number,
            operation_parameter.work_remaining.clone(),
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
            .tactical_parameters_mut()
            .insert(work_order_number, optimized_tactical_work_order);

        tactical_algorithm.calculate_objective_value();

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
            .tactical_parameters_mut()
            .insert(work_order_number, tactical_work_order_parameter);

        let activity_number = ActivityNumber(0);

        let mut tactical_activities = HashMap::new();

        tactical_activities.insert(
            activity_number,
            TacticalOperation::new(
                vec![],
                Resources::MtnMech,
                1,
                Work::from(0.0),
                work_order_number,
                activity_number,
            ),
        );

        tactical_algorithm
            .tactical_solution
            .tactical_days
            .insert(work_order_number, Some(tactical_activities));

        tactical_algorithm.schedule().unwrap();

        let scheduled_date = tactical_algorithm
            .tactical_solution
            .tactical_day(&work_order_number, &ActivityNumber(0));

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

        let mut tactical_activities = HashMap::new();

        tactical_activities.insert(
            activity_number,
            TacticalOperation::new(
                vec![],
                Resources::MtnMech,
                1,
                Work::from(0.0),
                work_order_number,
                activity_number,
            ),
        );

        tactical_algorithm
            .tactical_solution
            .tactical_days
            .insert(work_order_number, Some(tactical_activities));

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
            .tactical_parameters_mut()
            .insert(work_order_number, optimized_tactical_work_order);

        tactical_algorithm.schedule().unwrap();

        let scheduled_date = tactical_algorithm
            .tactical_solution
            .tactical_day(&work_order_number, &ActivityNumber(1));

        assert!(scheduled_date.is_ok());
    }
}
