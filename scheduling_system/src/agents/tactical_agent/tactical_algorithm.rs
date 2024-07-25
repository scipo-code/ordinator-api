use actix::Message;
use priority_queue::PriorityQueue;
use rand::seq::SliceRandom;
use serde::Serialize;
use shared_types::{
    agent_error::AgentError,
    scheduling_environment::{
        time_environment::day::Day,
        work_order::{
            operation::{operation_info::NumberOfPeople, ActivityNumber},
            WorkOrderNumber,
        },
        worker_environment::resources::{MainResources, Resources},
    },
    tactical::{
        tactical_resources_message::TacticalResourceRequest,
        tactical_response_resources::TacticalResponseResources,
        tactical_response_scheduling::TacticalResponseScheduling,
        tactical_response_time::TacticalResponseTime,
        tactical_scheduling_message::TacticalSchedulingRequest,
        tactical_time_message::TacticalTimeRequest, Days, TacticalInfeasibleCases,
        TacticalResources,
    },
    AlgorithmState, ConstraintState, LoadOperation,
};
use std::{borrow::Cow, cmp::Ordering, fmt::Display};
use std::{collections::HashMap, fmt};
use strum::IntoEnumIterator;
use tracing::{debug, error, info, instrument, warn};

use crate::agents::traits::{LargeNeighborHoodSearch, TestAlgorithm};

use shared_types::scheduling_environment::{
    time_environment::period::Period,
    work_order::{ActivityRelation, WorkOrder},
    WorkOrders,
};

#[derive(Clone)]
pub struct TacticalAlgorithm {
    pub objective_value: f64,
    pub tactical_periods: Vec<Period>,
    pub number_of_orders: u32,
    pub optimized_work_orders: HashMap<WorkOrderNumber, OptimizedTacticalWorkOrder>,
    pub capacity: TacticalResources,
    pub loading: TacticalResources,
    pub priority_queue: PriorityQueue<WorkOrderNumber, u32>,
    pub tactical_days: Vec<Day>,
}

#[derive(Clone, Serialize)]
pub struct OptimizedTacticalWorkOrder {
    pub main_work_center: MainResources,
    pub operation_parameters: HashMap<ActivityNumber, OperationParameters>,
    pub weight: u32,
    pub relations: Vec<ActivityRelation>,
    pub operation_solutions: Option<HashMap<ActivityNumber, OperationSolution>>,
    pub scheduled_period: Period,
}

#[allow(dead_code)]
#[derive(Clone, Serialize, Debug)]
pub struct OperationParameters {
    work_order_number: WorkOrderNumber,
    number: u32,
    duration: u32,
    operating_time: f64,
    work_remaining: f64,
    resource: Resources,
}

#[derive(Clone, Debug, Serialize)]
pub struct OperationSolution {
    pub scheduled: Vec<(Day, f64)>,
    pub resource: Resources,
    pub number: NumberOfPeople,
    pub work_order_number: WorkOrderNumber,
    pub activity_number: ActivityNumber,
}

impl OperationSolution {
    pub fn new(
        scheduled: Vec<(Day, f64)>,
        resource: Resources,
        number: NumberOfPeople,
        work_order_number: WorkOrderNumber,
        activity_number: ActivityNumber,
    ) -> OperationSolution {
        OperationSolution {
            scheduled,
            resource,
            number,
            work_order_number,
            activity_number,
        }
    }
}

impl Message for OperationSolution {
    type Result = bool;
}

impl Display for OperationParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OperationParameters:\n
        {:?}\n
        number: {}\n
        duration: {}\n
        operating_time: {}\n
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
    ) -> Self {
        TacticalAlgorithm {
            objective_value: f64::INFINITY,
            tactical_periods: time_horizon,
            number_of_orders: 0,
            optimized_work_orders: HashMap::new(),
            capacity,
            loading,
            priority_queue: PriorityQueue::new(),
            tactical_days,
        }
    }

    pub fn get_objective_value(&self) -> &f64 {
        &self.objective_value
    }

    pub fn capacity(&self, resource: &Resources, day: &Day) -> f64 {
        *self.capacity.resources.get(resource).unwrap().get(day)
    }

    pub fn capacity_mut(&mut self, resource: &Resources, day: &Day) -> &mut f64 {
        self.capacity
            .resources
            .get_mut(resource)
            .unwrap()
            .get_mut(day)
    }

    pub fn loading(&self, resource: &Resources, day: &Day) -> f64 {
        *self.loading.resources.get(resource).unwrap().get(day)
    }

    pub fn loading_mut(&mut self, resource: &Resources, day: &Day) -> &mut f64 {
        self.loading
            .resources
            .get_mut(resource)
            .unwrap()
            .get_mut(day)
    }
    pub fn update_state_based_on_strategic(
        &mut self,
        work_order: &WorkOrders,
        strategic_state: Vec<(WorkOrderNumber, Period)>,
    ) {
        for (work_order_number, period) in &strategic_state {
            let work_order = work_order.inner.get(work_order_number).unwrap();
            match self.optimized_work_orders.contains_key(work_order_number) {
                false => {
                    self.create_new_optimized_work_order(work_order, period.clone());
                }
                true => {
                    let optimized_work_order = self
                        .optimized_work_orders
                        .get_mut(work_order_number)
                        .unwrap();
                    if period != &optimized_work_order.scheduled_period {
                        optimized_work_order.scheduled_period = period.clone();
                        self.unschedule(*work_order_number);
                    }
                }
            };
        }

        let strategic_work_order_numbers: Vec<WorkOrderNumber> = strategic_state
            .iter()
            .map(|work_order_period| work_order_period.0)
            .collect();

        let leaving_work_order_numbers: Vec<WorkOrderNumber> = {
            self.optimized_work_orders
                .keys()
                .cloned()
                .filter(|tactical_work_order_number| {
                    !strategic_work_order_numbers.contains(tactical_work_order_number)
                })
                .collect()
        };

        for leaving_work_order_number in leaving_work_order_numbers {
            self.unschedule(leaving_work_order_number);
            self.optimized_work_orders
                .remove(&leaving_work_order_number);
        }

        self.schedule();

        self.calculate_objective_value();

        info!(tactical_objective_value = %self.get_objective_value());

        self.number_of_orders = self.optimized_work_orders.len() as u32;

        debug!(
            "Number of work orders in TacticalAgent: {}",
            self.number_of_orders
        );
    }

    pub fn create_new_optimized_work_order(&mut self, work_order: &WorkOrder, period: Period) {
        let mut optimized_work_order = OptimizedTacticalWorkOrder {
            main_work_center: work_order.main_work_center.clone(),
            operation_parameters: HashMap::new(),
            relations: work_order.relations().clone(),
            weight: work_order.work_order_weight(),
            scheduled_period: period,
            operation_solutions: None,
        };

        for (activity, operation) in work_order.operations() {
            let optimized_operation = OperationParameters {
                work_order_number: *work_order.work_order_number(),
                number: operation.number(),
                duration: operation.duration(),
                operating_time: operation.operating_time(),
                work_remaining: operation.work_remaining(),
                resource: operation.resource().clone(),
            };
            optimized_work_order
                .operation_parameters
                .insert(*activity, optimized_operation);
        }
        self.optimized_work_orders
            .insert(*work_order.work_order_number(), optimized_work_order);
    }

    pub fn unschedule_random_work_orders(
        &mut self,
        rng: &mut impl rand::Rng,
        number_of_work_orders: u32,
    ) {
        let work_order_numbers: Vec<WorkOrderNumber> =
            self.optimized_work_orders.clone().into_keys().collect();

        let random_work_order_numbers =
            work_order_numbers.choose_multiple(rng, number_of_work_orders as usize);
        for work_order_number in random_work_order_numbers {
            self.unschedule(*work_order_number);
        }
    }

    fn determine_aggregate_excess(&mut self) -> f64 {
        let mut objective_value_from_excess = 0.0;
        for resource in self.capacity.resources.keys() {
            for day in self.tactical_days.clone() {
                let excess_capacity = self.loading(resource, &day) - self.capacity(resource, &day);

                if excess_capacity > 0.0 {
                    objective_value_from_excess += excess_capacity;
                }
            }
        }
        objective_value_from_excess
    }
}

impl LargeNeighborHoodSearch for TacticalAlgorithm {
    type SchedulingRequest = TacticalSchedulingRequest;
    type SchedulingResponse = TacticalResponseScheduling;
    type ResourceRequest = TacticalResourceRequest;
    type ResourceResponse = TacticalResponseResources;
    type TimeRequest = TacticalTimeRequest;
    type TimeResponse = TacticalResponseTime;

    type SchedulingUnit = WorkOrderNumber;

    type Error = AgentError;

    fn calculate_objective_value(&mut self) {
        let mut objective_value_from_tardiness = 0.0;
        for (_work_order_number, optimized_work_order) in self.optimized_work_orders.iter() {
            let period_start_date = optimized_work_order
                .scheduled_period
                .start_date()
                .date_naive();

            let mut activity_keys: Vec<ActivityNumber> = optimized_work_order
                .operation_solutions
                .clone()
                .expect("When calculating the objective value all work order should be scheduled")
                .keys()
                .cloned()
                .collect();

            activity_keys.sort_unstable_by(|a, b| b.cmp(a));

            let last_activity = activity_keys.last().unwrap();

            let last_day = optimized_work_order
                .operation_solutions
                .clone()
                .unwrap()
                .get(last_activity)
                .unwrap()
                .scheduled
                .last()
                .unwrap()
                .0
                .date()
                .date_naive();

            let day_difference = last_day - period_start_date;

            objective_value_from_tardiness +=
                (optimized_work_order.weight as i64 * day_difference.num_days()) as f64;
        }

        // Calculate penalty for exceeding the capacity
        let objective_value_from_excess = 1000000.0 * self.determine_aggregate_excess();
        self.objective_value = objective_value_from_tardiness + objective_value_from_excess;
    }

    fn schedule(&mut self) {
        for (work_order_number, optimized_work_order) in self.optimized_work_orders.iter() {
            match &optimized_work_order.operation_solutions {
                None => {
                    self.priority_queue
                        .push(*work_order_number, optimized_work_order.weight);
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
            let optimized_work_order = match loop_state {
                LoopState::Unscheduled => self
                    .optimized_work_orders
                    .get(&current_work_order_number)
                    .unwrap(),
                LoopState::Scheduled => {
                    start_day_index = 0;

                    current_work_order_number = match self.priority_queue.pop() {
                        Some((work_order_number, _)) => work_order_number,
                        None => {
                            break;
                        }
                    };

                    self.optimized_work_orders
                        .get(&current_work_order_number)
                        .unwrap()
                }
            };

            let mut all_days = self.tactical_days.clone();
            let allowed_starting_days: Vec<&Day> = all_days
                .iter()
                .filter(|date| {
                    optimized_work_order
                        .scheduled_period
                        .contains_date(*date.date())
                })
                .collect();

            let start_day: Day = allowed_starting_days[start_day_index].clone();

            let allowed_days: Vec<_> = all_days
                .iter_mut()
                .filter(|date| start_day.date() <= date.date())
                .collect();

            let mut operation_solutions = HashMap::<ActivityNumber, OperationSolution>::new();

            let mut current_day = allowed_days.into_iter().peekable();

            let mut sorted_activities = optimized_work_order
                .operation_parameters
                .keys()
                .clone()
                .collect::<Vec<&ActivityNumber>>();

            sorted_activities.sort();

            for activity in sorted_activities {
                let operation_parameters = optimized_work_order
                    .operation_parameters
                    .get(activity)
                    .expect("The work order should always have its corresponding parameters");
                let mut activity_load = Vec::<(Day, f64)>::new();
                let resource = operation_parameters.resource.clone();

                let current_day_peek = match current_day.peek() {
                    Some(day) => day,
                    None => {
                        debug!(
                            current_work_order_number = ?current_work_order_number,
                            operation_parameters = ?operation_parameters,
                            optimized_work_order = ?optimized_work_order.scheduled_period,
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
                            0.0
                        }
                    };

                let loadings = self.determine_load(
                    first_day_remaining_capacity,
                    operation_parameters.operating_time,
                    operation_parameters.work_remaining,
                );

                for load in loadings {
                    let day = match current_day.peek() {
                        Some(day) => (*day).clone(),
                        None => {
                            debug!(
                                current_work_order_number = ?current_work_order_number,
                                operation_parameters = ?operation_parameters,
                                optimized_work_order = ?optimized_work_order.scheduled_period,
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
                            debug!(
                                current_work_order_number = ?current_work_order_number,
                                operation_parameters = ?operation_parameters,
                                optimized_work_order = ?optimized_work_order.scheduled_period,
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

                let operation_solution = OperationSolution::new(
                    activity_load,
                    resource,
                    operation_parameters.number,
                    current_work_order_number,
                    *activity,
                );
                operation_solutions.insert(*activity, operation_solution);
            }
            debug!(
                "Tactical {:?} has been scheduled starting on day {}",
                current_work_order_number,
                start_day.day_index()
            );
            self.update_loadings(&operation_solutions, LoadOperation::Add);
            loop_state = LoopState::Scheduled;

            self.optimized_work_orders
                .get_mut(&current_work_order_number)
                .unwrap()
                .operation_solutions = Some(operation_solutions.clone());

            if self
                .optimized_work_orders
                .get_mut(&current_work_order_number)
                .is_none()
            {
                error!(unscheduled_work_order = ?current_work_order_number);
                panic!("Unscheduled work order got through the schedule function");
            }
        }

        if self
            .optimized_work_orders
            .iter()
            .any(|wo| wo.1.operation_solutions.is_none())
        {
            panic!("The TacticalAlgorithm.schedule() did not schedule all work orders");
        }
    }

    fn unschedule(&mut self, optimized_work_order_number: Self::SchedulingUnit) {
        let optimized_work_order = self.optimized_work_orders.get_mut(&optimized_work_order_number)
            .expect("A call was made to TacticalAlgorith.unschedule(work_order_number) where the underlying work order was not in a scheduled state");

        match optimized_work_order.operation_solutions.take() {
            Some(operation_solutions) => {
                self.update_loadings(&operation_solutions, LoadOperation::Sub);
            }
            None => {
                debug!(
                    "Work order {:?} was not scheduled before leaving the tactical schedule",
                    optimized_work_order_number
                );
            }
        }
    }

    fn update_scheduling_state(
        &mut self,
        _scheduling_message: Self::SchedulingRequest,
    ) -> Result<Self::SchedulingResponse, Self::Error> {
        Ok(TacticalResponseScheduling {})
        // This is where the algorithm will update the scheduling state.
    }

    fn update_time_state(
        &mut self,
        _time_message: Self::TimeRequest,
    ) -> Result<Self::TimeResponse, Self::Error> {
        // This is where the algorithm will update the time state.
        Ok(TacticalResponseTime {})
    }

    #[instrument(level = "info", skip(self))]
    fn update_resources_state(
        &mut self,
        resource_message: Self::ResourceRequest,
    ) -> Result<Self::ResourceResponse, Self::Error> {
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
                                return Err(AgentError::StateUpdateError(
                                    "Day not found in the tactical days".to_string(),
                                ));
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

                info!(loadings = ?loadings);
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

                let mut percentage_loading = HashMap::<Resources, Days>::new();

                for (resource, days) in &capacities.resources {
                    if !percentage_loading.contains_key(resource) {
                        percentage_loading.insert(resource.clone(), Days::new(HashMap::new()));
                    }
                    for (day, capacity) in &days.days {
                        let percentage =
                            (loadings.resources.get(resource).unwrap().get(day) / capacity * 100.0)
                                .round();
                        percentage_loading
                            .get_mut(resource)
                            .unwrap()
                            .days
                            .insert(day.clone(), percentage);
                    }
                }

                let algorithm_resources = TacticalResources::new(percentage_loading);
                let tactical_response_resources =
                    TacticalResponseResources::Percentage(algorithm_resources);
                Ok(tactical_response_resources)
            }
        }
    }
}

enum LoopState {
    Unscheduled,
    Scheduled,
}

impl TacticalAlgorithm {
    fn update_loadings(
        &mut self,
        operation_solutions: &HashMap<ActivityNumber, OperationSolution>,
        load_operation: LoadOperation,
    ) {
        for operation in operation_solutions.values() {
            let resource = operation.resource.clone();
            for loadings in operation.scheduled.clone() {
                let day = loadings.0;
                let load = loadings.1;
                let resource_loading = self.loading(&resource, &day);

                let new_load = match load_operation {
                    LoadOperation::Add => resource_loading + load,
                    LoadOperation::Sub => resource_loading - load,
                };
                *self.loading_mut(&resource, &day) = new_load;
            }
        }
    }

    fn remaining_capacity(&self, resource: &Resources, day: &Day) -> Option<f64> {
        let remaining_capacity = self.capacity(resource, day) - self.loading(resource, day);

        if remaining_capacity <= 0.0 {
            None
        } else {
            Some(remaining_capacity)
        }
    }

    fn determine_load(
        &self,
        remaining_capacity: f64,
        mut operating_time: f64,
        mut work_remaining: f64,
    ) -> Vec<f64> {
        if operating_time <= 0.0 {
            operating_time = 4.0;
            warn!("Operating time is less or equal to 0.0. This is an error in the data initialization, setting it to 4.0 hours as default");
        }

        let mut loadings = Vec::new();

        let first_day_load = match remaining_capacity.partial_cmp(&operating_time) {
            Some(Ordering::Less) => remaining_capacity,
            Some(Ordering::Equal) => remaining_capacity,
            Some(Ordering::Greater) => operating_time,
            None => panic!("remaining work and operating_time are not comparable. There is an error in the data initialization"),
        }.min(work_remaining);

        loadings.push(first_day_load);
        work_remaining -= first_day_load;

        while work_remaining > 0.0 {
            let load = operating_time.min(work_remaining);
            loadings.push(load);
            work_remaining -= load;
        }
        loadings
    }

    pub fn optimized_work_orders(&self) -> &HashMap<WorkOrderNumber, OptimizedTacticalWorkOrder> {
        &self.optimized_work_orders
    }
}

#[allow(dead_code)]
enum OperationDifference {
    SameDay,
    DiffDay,
}

impl TestAlgorithm for TacticalAlgorithm {
    type InfeasibleCases = TacticalInfeasibleCases;

    #[instrument(level = "info", skip(self))]
    fn determine_algorithm_state(&self) -> AlgorithmState<Self::InfeasibleCases> {
        let mut algorithm_state = AlgorithmState::Infeasible(TacticalInfeasibleCases::default());

        let mut aggregated_load: HashMap<Resources, HashMap<Day, f64>> = HashMap::new();
        for (_work_order_number, optimized_work_order) in self.optimized_work_orders.clone() {
            for (_activity, operation_solution) in
                optimized_work_order.operation_solutions.unwrap_or_default()
            {
                let resource = operation_solution.resource;

                for (day, load) in operation_solution.scheduled {
                    *aggregated_load
                        .entry(resource.clone())
                        .or_default()
                        .entry(day)
                        .or_insert(0.0) += load;
                }
            }
        }

        algorithm_state
            .infeasible_cases_mut()
            .unwrap()
            .aggregated_load = (|| {
            for resource in Resources::iter() {
                for day in &self.tactical_days {
                    let resource_map = match aggregated_load.get(&resource) {
                        Some(map) => Cow::Borrowed(map),
                        None => Cow::Owned(HashMap::new()),
                    };

                    let agg_load = resource_map.get(day).unwrap_or(&0.0);
                    let sch_load = self.loading(&resource, day);
                    if (*agg_load - sch_load).abs() >= 0.00001 {
                        error!(agg_load = ?agg_load, sch_load = ?sch_load, resource = ?resource, day = ?day);
                        return ConstraintState::Infeasible(format!("Loads does not match on: day {}\nresource: {}\nscheduled load: {}\naggregated_load: {}", day, resource, sch_load, agg_load));
                    }
                }
            }
            ConstraintState::Feasible
        })();

        algorithm_state
            .infeasible_cases_mut()
            .unwrap()
            .earliest_start_day = (|| {
            for (work_order_number, optimized_work_order) in self.optimized_work_orders.clone() {
                let start_date_from_period = optimized_work_order.scheduled_period.start_date();

                if let Some(operation_solutions) = optimized_work_order.operation_solutions {
                    let start_days: Vec<_> =
                        operation_solutions
                            .values()
                            .map(|operation_solution| {
                                operation_solution
                            .scheduled
                            .first()
                            .expect("All scheduled operations should have a first scheduled day")
                            .0
                            .clone()
                            })
                            .collect();

                    for start_day in start_days {
                        if start_day.date().date_naive() < start_date_from_period.date_naive() {
                            error!(start_day = ?start_day.date(), start_date_from_period = ?start_date_from_period);
                            return ConstraintState::Infeasible(format!(
                                "{:?} is outside of its earliest start day: {}",
                                work_order_number, start_date_from_period
                            ));
                        }
                    }
                }
            }
            ConstraintState::Feasible
        })();

        algorithm_state
            .infeasible_cases_mut()
            .unwrap()
            .all_scheduled = (|| {
            for (work_order_number, optimized_work_order) in self.optimized_work_orders.clone() {
                if optimized_work_order.operation_solutions.is_none() {
                    return ConstraintState::Infeasible(work_order_number.0.to_string());
                }
            }
            ConstraintState::Feasible
        })();

        algorithm_state
            .infeasible_cases_mut()
            .unwrap()
            .respect_period_id = (|| {
            for (_work_order_number, optimized_work_order) in self.optimized_work_orders.clone() {
                if !self
                    .tactical_periods
                    .contains(&optimized_work_order.scheduled_period)
                {
                    error!(work_order_number = ?_work_order_number, scheduled_period = ?optimized_work_order.scheduled_period, tactical_periods = ?self.tactical_periods, "Tactical period does not contain the scheduled period of the tactical work order");
                    return ConstraintState::Infeasible(format!(
                        "{:?} has a wrong scheduled period {}",
                        _work_order_number, optimized_work_order.scheduled_period
                    ));
                }
            }
            ConstraintState::Feasible
        })();

        let infeasible_cases = algorithm_state.infeasible_cases_mut().unwrap();

        if infeasible_cases.aggregated_load == ConstraintState::Feasible
            && infeasible_cases.earliest_start_day == ConstraintState::Feasible
            && infeasible_cases.all_scheduled == ConstraintState::Feasible
            && infeasible_cases.respect_period_id == ConstraintState::Feasible
        {
            AlgorithmState::Feasible
        } else {
            algorithm_state
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::{collections::HashMap, str::FromStr};

    use chrono::{Days, Duration};
    use shared_types::scheduling_environment::{
        work_order::{operation::ActivityNumber, WorkOrderNumber},
        worker_environment::resources::{MainResources, Resources},
    };
    use strum::IntoEnumIterator;

    use crate::agents::{
        tactical_agent::tactical_algorithm::OperationSolution, traits::LargeNeighborHoodSearch,
    };

    use super::{Day, OperationParameters, OptimizedTacticalWorkOrder};
    use shared_types::scheduling_environment::{
        time_environment::period::Period, work_order::ActivityRelation,
    };

    #[test]
    fn test_determine_load_1() {
        let tactical_algorithm = super::TacticalAlgorithm::new(
            vec![],
            vec![],
            super::TacticalResources::new(HashMap::new()),
            super::TacticalResources::new(HashMap::new()),
        );

        let remaining_capacity = 3.0;
        let operating_time = 5.0;
        let work_remaining = 10.0;

        let loadings =
            tactical_algorithm.determine_load(remaining_capacity, operating_time, work_remaining);

        assert_eq!(loadings, vec![3.0, 5.0, 2.0]);
    }

    #[test]
    fn test_determine_load_2() {
        let tactical_algorithm = super::TacticalAlgorithm::new(
            vec![],
            vec![],
            super::TacticalResources::new(HashMap::new()),
            super::TacticalResources::new(HashMap::new()),
        );

        let remaining_capacity = 3.0;
        let operating_time = 0.0;
        let work_remaining = 10.0;

        let loadings =
            tactical_algorithm.determine_load(remaining_capacity, operating_time, work_remaining);

        assert_eq!(loadings, vec![3.0, 4.0, 3.0]);
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
        );

        let operation_parameter =
            OperationParameters::new(work_order_number, 1, 1, 1.0, 1.0, Resources::MtnMech);

        let operation_solution = OperationSolution::new(
            vec![(tactical_algorithm.tactical_days[27].clone(), 1.0)],
            Resources::MtnMech,
            operation_parameter.number,
            work_order_number,
            activity_number,
        );

        let mut operation_parameters = HashMap::new();
        operation_parameters.insert(activity_number, operation_parameter);

        let mut operation_solutions = HashMap::new();
        operation_solutions.insert(ActivityNumber(1), operation_solution);

        let optimized_tactical_work_order = OptimizedTacticalWorkOrder::new(
            MainResources::MtnMech,
            operation_parameters,
            10,
            vec![],
            Some(operation_solutions),
            first_period,
        );

        tactical_algorithm
            .optimized_work_orders
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
                0.0,
            ),
            super::TacticalResources::new_from_data(
                Resources::iter().collect(),
                tactical_days(56),
                0.0,
            ),
        );

        let operation_parameter =
            OperationParameters::new(work_order_number, 1, 1, 1.0, 1.0, Resources::MtnMech);

        let mut operation_parameters = HashMap::new();
        operation_parameters.insert(ActivityNumber(1), operation_parameter);

        let optimized_tactical_work_order = OptimizedTacticalWorkOrder::new(
            MainResources::MtnMech,
            operation_parameters,
            10,
            vec![],
            None,
            third_period.clone(),
        );

        tactical_algorithm
            .optimized_work_orders
            .insert(work_order_number, optimized_tactical_work_order);

        tactical_algorithm.schedule();

        let scheduled_date = tactical_algorithm
            .optimized_work_orders
            .get(&work_order_number)
            .unwrap()
            .operation_solutions
            .as_ref()
            .unwrap()
            .get(&shared_types::scheduling_environment::work_order::operation::ActivityNumber(1))
            .unwrap()
            .scheduled
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
                100.0,
            ),
            super::TacticalResources::new_from_data(
                Resources::iter().collect(),
                tactical_days(56),
                0.0,
            ),
        );

        let operation_parameter =
            OperationParameters::new(work_order_number, 1, 1, 1.0, 1.0, Resources::MtnMech);

        let mut operation_parameters = HashMap::new();
        operation_parameters.insert(ActivityNumber(1), operation_parameter);

        let optimized_tactical_work_order = OptimizedTacticalWorkOrder::new(
            MainResources::MtnMech,
            operation_parameters,
            10,
            vec![],
            None,
            third_period.clone(),
        );

        tactical_algorithm
            .optimized_work_orders
            .insert(work_order_number, optimized_tactical_work_order);

        tactical_algorithm.schedule();

        let scheduled_date = tactical_algorithm
            .optimized_work_orders
            .get(&work_order_number)
            .unwrap()
            .operation_solutions
            .as_ref()
            .unwrap()
            .get(&ActivityNumber(1))
            .unwrap()
            .scheduled
            .first()
            .unwrap()
            .0
            .date()
            .date_naive();

        assert!(scheduled_date >= third_period.start_date().date_naive());
    }
    impl OptimizedTacticalWorkOrder {
        pub fn new(
            main_work_center: MainResources,
            operation_parameters: HashMap<ActivityNumber, OperationParameters>,
            weight: u32,
            relations: Vec<ActivityRelation>,
            operation_solutions: Option<HashMap<ActivityNumber, OperationSolution>>,
            scheduled_period: Period,
        ) -> Self {
            OptimizedTacticalWorkOrder {
                main_work_center,
                operation_parameters,
                weight,
                relations,
                operation_solutions,
                scheduled_period,
            }
        }
    }

    impl OperationParameters {
        pub fn new(
            work_order_number: WorkOrderNumber,
            number: u32,
            duration: u32,
            operating_time: f64,
            work_remaining: f64,
            resource: Resources,
        ) -> Self {
            OperationParameters {
                work_order_number,
                number,
                duration,
                operating_time,
                work_remaining,
                resource,
            }
        }
    }
}
