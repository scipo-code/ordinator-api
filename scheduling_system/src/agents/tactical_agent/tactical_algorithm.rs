use actix::Message;
use anyhow::{bail, Context, Result};
use arc_swap::Guard;
use chrono::NaiveDate;
use priority_queue::PriorityQueue;
use rand::seq::SliceRandom;
use serde::Serialize;
use shared_types::{
    agent_error::AgentError,
    scheduling_environment::{
        time_environment::day::Day,
        work_order::{
            operation::{operation_info::NumberOfPeople, ActivityNumber, Work},
            WorkOrderActivity, WorkOrderNumber,
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
        tactical_time_message::TacticalTimeRequest, TacticalResources,
    },
    LoadOperation,
};
use std::{
    cmp::Ordering,
    fmt::Display,
    sync::{Arc, MutexGuard},
};
use std::{collections::HashMap, fmt};
use tracing::{event, instrument, Level};

use crate::agents::{
    traits::LargeNeighborHoodSearch, ArcSwapSharedSolution, SharedSolution, TacticalSolution,
};

use shared_types::scheduling_environment::{
    time_environment::period::Period,
    work_order::{ActivityRelation, WorkOrder},
};

pub struct TacticalAlgorithm {
    pub objective_value: f64,
    pub tactical_periods: Vec<Period>,
    pub strategic_tactical_solution_arc_swap: Arc<ArcSwapSharedSolution>,
    pub loaded_shared_solution: Guard<Arc<SharedSolution>>,
    pub tactical_solution: TacticalSolution,
    pub tactical_parameters: TacticalParameters,
    pub capacity: TacticalResources,
    pub loading: TacticalResources,
    pub priority_queue: PriorityQueue<WorkOrderNumber, u64>,
    pub tactical_days: Vec<Day>,
}

#[derive(Default, Clone)]
pub struct TacticalParameters(pub HashMap<WorkOrderNumber, TacticalParameter>);

#[derive(Clone, Serialize)]
pub struct TacticalParameter {
    pub main_work_center: Resources,
    pub operation_parameters: HashMap<ActivityNumber, OperationParameter>,
    pub weight: u64,
    pub relations: Vec<ActivityRelation>,
    // TODO: These two should be moved out of the pa
    pub earliest_allowed_start_date: NaiveDate,
}

impl TacticalParameter {
    pub fn new(
        main_work_center: Resources,
        operation_parameters: HashMap<ActivityNumber, OperationParameter>,
        weight: u64,
        relations: Vec<ActivityRelation>,
        earliest_allowed_start_date: NaiveDate,
    ) -> Self {
        Self {
            main_work_center,
            operation_parameters,
            weight,
            relations,
            earliest_allowed_start_date,
        }
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct OperationParameter {
    work_order_number: WorkOrderNumber,
    number: NumberOfPeople,
    duration: Work,
    operating_time: Work,
    work_remaining: Work,
    resource: Resources,
}

impl OperationParameter {
    pub fn new(
        work_order_number: WorkOrderNumber,
        number: NumberOfPeople,
        duration: Work,
        operating_time: Work,
        work_remaining: Work,
        resource: Resources,
    ) -> Self {
        Self {
            work_order_number,
            number,
            duration,
            operating_time,
            work_remaining,
            resource,
        }
    }
}

#[derive(Hash, PartialEq, PartialOrd, Ord, Eq, Clone, Debug, Serialize)]
pub struct TacticalOperation {
    pub scheduled: Vec<(Day, Work)>,
    pub scheduled_period: Option<Period>,
    pub resource: Resources,
    pub number: NumberOfPeople,
    pub work_remaining: Work,
    pub work_order_activity: WorkOrderActivity,
}

impl TacticalOperation {
    pub fn new(
        scheduled: Vec<(Day, Work)>,
        scheduled_period: Option<Period>,
        resource: Resources,
        number: NumberOfPeople,
        work_remaining: Work,
        work_order_number: WorkOrderNumber,
        activity_number: ActivityNumber,
    ) -> TacticalOperation {
        TacticalOperation {
            scheduled,
            scheduled_period,
            resource,
            number,
            work_remaining,
            work_order_activity: (work_order_number, activity_number),
        }
    }
}

impl Message for TacticalOperation {
    type Result = bool;
}

impl Display for OperationParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OperationParameters:\n
        {:?}\n
        number: {}\n
        duration: {}\n
        operating_time: {:?}\n
        work_remaining: {}\n
        resource: {}",
            self.work_order_number,
            self.number,
            self.duration,
            self.operating_time,
            self.work_remaining,
            self.resource
        )
    }
}

impl TacticalAlgorithm {
    pub fn new(
        tactical_days: Vec<Day>,
        time_horizon: Vec<Period>,
        capacity: TacticalResources,
        loading: TacticalResources,
        strategic_tactical_solution_arc_swap: Arc<ArcSwapSharedSolution>,
    ) -> Self {
        let loaded_shared_solution = strategic_tactical_solution_arc_swap.0.load();
        TacticalAlgorithm {
            objective_value: f64::INFINITY,
            tactical_periods: time_horizon,
            strategic_tactical_solution_arc_swap,
            loaded_shared_solution,
            tactical_solution: TacticalSolution::default(),
            tactical_parameters: TacticalParameters::default(),
            capacity,
            loading,
            priority_queue: PriorityQueue::new(),
            tactical_days,
        }
    }

    pub fn get_objective_value(&self) -> &f64 {
        &self.objective_value
    }

    pub fn capacity(&self, resource: &Resources, day: &Day) -> &Work {
        self.capacity.resources.get(resource).unwrap().get(day)
    }

    pub fn capacity_mut(&mut self, resource: &Resources, day: &Day) -> &mut Work {
        self.capacity
            .resources
            .get_mut(resource)
            .unwrap()
            .get_mut(day)
    }

    pub fn loading(&self, resource: &Resources, day: &Day) -> &Work {
        self.loading.resources.get(resource).unwrap().get(day)
    }

    pub fn loading_mut(&mut self, resource: &Resources, day: &Day) -> &mut Work {
        self.loading
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
                .operation_parameters
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

    fn determine_aggregate_excess(&mut self) -> f64 {
        let mut objective_value_from_excess = 0.0;
        for resource in self.capacity.resources.keys() {
            for day in self.tactical_days.clone() {
                let excess_capacity = self.loading(resource, &day) - self.capacity(resource, &day);

                if excess_capacity > Work::from(0.0) {
                    objective_value_from_excess += excess_capacity.to_f64();
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
            .work_orders()
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
            self.tactical_solution
                .tactical_period
                .insert(*work_order_number, None);
        }
        self.make_atomic_pointer_swap_for_with_the_better_tactical_solution();
    }

    pub(crate) fn make_atomic_pointer_swap_for_with_the_better_tactical_solution(&self) {
        let mut shared_solution = (**self.loaded_shared_solution).clone();
        shared_solution.tactical = self.tactical_solution.clone();
        self.strategic_tactical_solution_arc_swap
            .0
            .store(Arc::new(shared_solution));
    }

    pub fn load_shared_solution(&mut self) {
        self.loaded_shared_solution = self.strategic_tactical_solution_arc_swap.0.load();
    }
}

impl LargeNeighborHoodSearch for TacticalAlgorithm {
    type BetterSolution = ();
    type SchedulingRequest = TacticalSchedulingRequest;
    type SchedulingResponse = TacticalResponseScheduling;
    type ResourceRequest = TacticalResourceRequest;
    type ResourceResponse = TacticalResponseResources;
    type TimeRequest = TacticalTimeRequest;
    type TimeResponse = TacticalResponseTime;

    type SchedulingUnit = WorkOrderNumber;

    type Error = AgentError;

    fn calculate_objective_value(&mut self) -> Self::BetterSolution {
        let mut objective_value_from_tardiness = 0.0;
        for (work_order_number, _tactical_solution) in self.tactical_solution.tactical_days.iter() {
            let tactical_parameter = self.tactical_parameters().get(work_order_number).unwrap();
            let period_start_date = match &self.tactical_solution.tactical_period(work_order_number)
            {
                Some(period) => period.start_date().date_naive(),
                None => tactical_parameter.earliest_allowed_start_date,
            };

            let mut activity_keys: Vec<ActivityNumber> = tactical_parameter
                .operation_parameters
                .keys()
                .cloned()
                .collect();

            activity_keys.sort_unstable_by(|a, b| b.cmp(a));

            let last_activity = activity_keys.last().unwrap();

            let last_day = self
                .tactical_solution
                .tactical_day(work_order_number, last_activity)
                .last()
                .unwrap()
                .0
                .date()
                .date_naive();

            let day_difference = last_day - period_start_date;

            objective_value_from_tardiness +=
                (tactical_parameter.weight as i64 * day_difference.num_days()) as f64;
        }

        // Calculate penalty for exceeding the capacity
        let objective_value_from_excess = 1000000.0 * self.determine_aggregate_excess();
        self.objective_value = objective_value_from_tardiness + objective_value_from_excess;
    }

    fn schedule(&mut self) {
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

        let mut current_work_order_number = match self.priority_queue.pop() {
            Some((work_order_number, _)) => work_order_number,
            None => return,
        };

        'main: loop {
            let tactical_parameter = match loop_state {
                LoopState::Unscheduled => self
                    .tactical_parameters()
                    .get(&current_work_order_number)
                    .unwrap(),
                LoopState::ScheduledOrRemoved => {
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
            let allowed_starting_days: Vec<&Day> = match self
                .tactical_solution
                .tactical_period(&current_work_order_number)
            {
                Some(period) => all_days
                    .iter()
                    .filter(|date| period.contains_date(date.date().date_naive()))
                    .collect(),
                None => self
                    .tactical_days
                    .iter()
                    .filter(|day| {
                        tactical_parameter.earliest_allowed_start_date <= day.date().date_naive()
                    })
                    .collect(),
            };

            let start_day: Day = match allowed_starting_days.get(start_day_index) {
                Some(start_day) => (*start_day).clone(),
                None => {
                    self.tactical_solution
                        .tactical_remove_work_order(&current_work_order_number);

                    loop_state = LoopState::ScheduledOrRemoved;
                    continue 'main;
                }
            };

            let allowed_days: Vec<_> = all_days
                .iter_mut()
                .filter(|date| start_day.date() <= date.date())
                .collect();

            let mut operation_solutions = HashMap::<ActivityNumber, TacticalOperation>::new();

            let mut current_day = allowed_days.into_iter().peekable();

            let mut sorted_activities = tactical_parameter
                .operation_parameters
                .keys()
                .clone()
                .collect::<Vec<&ActivityNumber>>();

            sorted_activities.sort();

            for activity in sorted_activities {
                let operation_parameters = tactical_parameter
                    .operation_parameters
                    .get(activity)
                    .expect("The work order should always have its corresponding parameters");
                let mut activity_load = Vec::<(Day, Work)>::new();
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
                        break;
                    }
                };

                let first_day_remaining_capacity =
                    match self.remaining_capacity(&resource, current_day_peek) {
                        Some(remaining_capacity) => remaining_capacity,
                        None => {
                            if start_day_index <= 12 {
                                start_day_index += 1;
                                loop_state = LoopState::Unscheduled;
                                continue 'main;
                            }
                            Work::from(0.0)
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
                            // brak should schedule what is possible and cut the rest out.
                            break;
                        }
                    };
                    if start_day_index <= 12
                        && self.remaining_capacity(&resource, current_day).is_none()
                    {
                        start_day_index += 1;
                        loop_state = LoopState::Unscheduled;
                        continue 'main;
                    };
                }

                let scheduled_period = self.tactical_periods.iter().find(|per| {
                    per.contains_date(activity_load.first().unwrap().0.date().date_naive())
                });

                let operation_solution = TacticalOperation::new(
                    activity_load,
                    scheduled_period.cloned(),
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
            loop_state = LoopState::ScheduledOrRemoved;

            *self
                .tactical_solution
                .tactical_days
                .get_mut(&current_work_order_number)
                .unwrap() = Some(operation_solutions.clone());

            if self
                .tactical_parameters_mut()
                .get_mut(&current_work_order_number)
                .is_none()
            {
                event!(Level::ERROR, unscheduled_work_order = ?current_work_order_number);
                panic!("Unscheduled work order got through the schedule function");
            }
        }

        if self
            .tactical_solution
            .tactical_days
            .iter()
            .any(|wo| wo.1.is_none())
        {
            panic!("The TacticalAlgorithm.schedule() did not schedule all work orders");
        }
    }

    fn unschedule(&mut self, work_order_number: Self::SchedulingUnit) -> Result<()> {
        let tactical_solution = self
            .tactical_solution
            .tactical_days
            .get_mut(&work_order_number)
            .with_context(|| {
                format!("This means that the TacticalAlgorithm has been initialized wrong")
            })?;

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
                let loadings = self.loading.clone();

                event!(Level::DEBUG,loadings = ?loadings);
                let tactical_response_resources = TacticalResponseResources::Loading(loadings);
                Ok(tactical_response_resources)
            }
            TacticalResourceRequest::GetCapacities {
                days_end: _,
                select_resources: _,
            } => {
                let capacities = self.capacity.clone();

                let tactical_response_resources = TacticalResponseResources::Capacity(capacities);

                Ok(tactical_response_resources)
            }
            TacticalResourceRequest::GetPercentageLoadings {
                days_end: _,
                resources: _,
            } => {
                let capacities = &self.capacity;
                let loadings = &self.loading;

                let tactical_response_resources =
                    TacticalResponseResources::Percentage((capacities.clone(), loadings.clone()));
                Ok(tactical_response_resources)
            }
        }
    }
}

enum LoopState {
    Unscheduled,
    ScheduledOrRemoved,
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

        let first_day_load = match remaining_capacity.partial_cmp(&operating_time) {
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
        &mut self.tactical_parameters.0
    }

    pub fn tactical_parameters(&self) -> &HashMap<WorkOrderNumber, TacticalParameter> {
        &self.tactical_parameters.0
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

    use chrono::{Days, Duration, NaiveDate};
    use shared_types::scheduling_environment::{
        work_order::{
            operation::{ActivityNumber, Work},
            WorkOrderNumber,
        },
        worker_environment::resources::Resources,
    };
    use strum::IntoEnumIterator;

    use crate::agents::{
        tactical_agent::tactical_algorithm::TacticalOperation, traits::LargeNeighborHoodSearch,
        ArcSwapSharedSolution,
    };

    use super::{Day, OperationParameter, TacticalParameter};
    use shared_types::scheduling_environment::time_environment::period::Period;

    #[test]
    fn test_determine_load_1() {
        let tactical_algorithm = super::TacticalAlgorithm::new(
            vec![],
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
            vec![first_period.clone()],
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
            None,
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

        assert_eq!(tactical_algorithm.get_objective_value(), &270.0);
    }

    #[test]
    fn test_schedule_1() {
        let work_order_number = WorkOrderNumber(2100000001);
        let first_period = Period::from_str("2024-W13-14").unwrap();
        let second_period = first_period.clone() + Duration::weeks(2);
        let third_period = second_period.clone() + Duration::weeks(2);

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
            vec![
                first_period.clone(),
                second_period.clone(),
                third_period.clone(),
            ],
            super::TacticalResources::new_from_data(
                Resources::iter().collect(),
                tactical_days(56),
                Work::from(0.0),
            ),
            super::TacticalResources::new_from_data(
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

        tactical_algorithm.schedule();

        let scheduled_date = tactical_algorithm
            .tactical_solution
            .tactical_day(&work_order_number, &ActivityNumber(1))
            .first()
            .unwrap()
            .0
            .date()
            .date_naive();

        assert!(scheduled_date >= third_period.start_date().date_naive());
    }

    #[test]
    fn test_schedule_2() {
        let work_order_number = WorkOrderNumber(2100000010);
        let first_period = Period::from_str("2024-W13-14").unwrap();
        let second_period = first_period.clone() + Duration::weeks(2);
        let third_period = second_period.clone() + Duration::weeks(2);

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
            vec![
                first_period.clone(),
                second_period.clone(),
                third_period.clone(),
            ],
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

        tactical_algorithm.schedule();

        let scheduled_date = tactical_algorithm
            .tactical_solution
            .tactical_day(&work_order_number, &ActivityNumber(1))
            .first()
            .unwrap()
            .0
            .date()
            .date_naive();

        assert!(scheduled_date >= third_period.start_date().date_naive());
    }
}
