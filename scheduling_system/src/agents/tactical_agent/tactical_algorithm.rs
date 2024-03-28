use chrono::{DateTime, Utc};
use colored::Colorize;
use priority_queue::PriorityQueue;
use shared_messages::{
    agent_error::AgentError,
    resources::Resources,
    tactical::{
        tactical_resources_message::TacticalResourceMessage,
        tactical_scheduling_message::TacticalSchedulingMessage,
        tactical_time_message::TacticalTimeMessage,
    },
};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Write;
use tracing::{debug, instrument, warn};

use crate::{
    agents::traits::LargeNeighborHoodSearch,
    models::{time_environment::period::Period, work_order::ActivityRelation, WorkOrders},
};

pub struct TacticalAlgorithm {
    objective_value: f64,
    time_horizon: usize,
    number_of_orders: u32,
    optimized_work_orders: HashMap<u32, OptimizedTacticalWorkOrder>,
    capacity: AlgorithmResources,
    loading: AlgorithmResources,
    priority_queue: PriorityQueue<u32, u32>,
    tactical_days: Vec<Day>,
}
#[allow(dead_code)]
struct OptimizedTacticalWorkOrder {
    optimized_activities: HashMap<u32, OptimizedOperation>,
    weight: u32,
    relations: Vec<ActivityRelation>,
    scheduled_period: Period,
}

#[derive(Debug, Clone)]
pub(crate) struct AlgorithmResources {
    resources: HashMap<Resources, HashMap<Day, f64>>,
}

impl AlgorithmResources {
    pub fn new(resources: HashMap<Resources, HashMap<Day, f64>>) -> Self {
        AlgorithmResources { resources }
    }

    pub fn capacity(&self, resource: &Resources, day: &Day) -> f64 {
        *self.resources.get(resource).unwrap().get(day).unwrap()
    }

    pub fn capacity_mut(&mut self, resource: &Resources, day: &Day) -> &mut f64 {
        self.resources
            .get_mut(resource)
            .unwrap()
            .get_mut(day)
            .unwrap()
    }

    pub fn loading(&self, resource: &Resources, day: &Day) -> f64 {
        *self.resources.get(resource).unwrap().get(day).unwrap()
    }

    pub fn loading_mut(&mut self, resource: &Resources, day: &Day) -> &mut f64 {
        self.resources
            .get_mut(resource)
            .unwrap()
            .get_mut(day)
            .unwrap()
    }

    fn to_string(&self, number_of_periods: u32) -> String {
        let mut string = String::new();
        let mut days = self
            .resources
            .values()
            .flat_map(|inner_map| inner_map.keys())
            .collect::<Vec<_>>();
        days.sort();
        days.dedup();

        // Header
        write!(string, "{:<12}", "Resource").ok();
        for (nr_day, day) in days.iter().enumerate().take(number_of_periods as usize) {
            match nr_day {
                0..=13 => write!(string, "{:>12}", day.date.to_string().red()).ok(),
                14..=27 => write!(string, "{:>12}", day.date.to_string().green()).ok(),
                _ => write!(string, "{:>12}", day.date.to_string()).ok(),
            }
            .unwrap()
        }
        writeln!(string).ok();

        // Rows
        for (resource, inner_map) in self.resources.iter() {
            write!(string, "{:<12}", resource.variant_name()).unwrap();
            for (nr_day, day) in days.iter().enumerate().take(number_of_periods as usize) {
                let value = inner_map.get(day).unwrap_or(&0.0);
                match nr_day {
                    0..=13 => write!(string, "{:>12}", value.round().to_string().red()).ok(),
                    14..=27 => write!(string, "{:>12}", value.round().to_string().green()).ok(),
                    _ => write!(string, "{:>12}", value.round()).ok(),
                }
                .unwrap();
            }
            writeln!(string).ok();
        }
        string
    }
}

#[allow(dead_code)]
struct OptimizedOperation {
    work_order_id: u32,
    scheduled_start: u32,
    scheduled_end: u32,
    number: u32,
    duration: u32,
    operating_time: f64,
    work_remaining: f64,
    resource: Resources,
}

#[derive(Eq, PartialEq, Hash, Clone, PartialOrd, Ord, Debug)]
pub struct Day {
    day_index: usize,
    date: DateTime<Utc>,
}

impl Day {
    pub fn new(day_index: usize, date: DateTime<Utc>) -> Self {
        Day { day_index, date }
    }

    pub fn date(&self) -> &DateTime<Utc> {
        &self.date
    }
}

impl TacticalAlgorithm {
    pub fn new(
        tactical_days: Vec<Day>,
        capacity: AlgorithmResources,
        loading: AlgorithmResources,
    ) -> Self {
        TacticalAlgorithm {
            objective_value: 0.0,
            time_horizon: tactical_days.len(),
            number_of_orders: 0,
            optimized_work_orders: HashMap::new(),
            capacity,
            loading,
            priority_queue: PriorityQueue::new(),
            tactical_days,
        }
    }

    pub fn update_state_based_on_strategic(
        &mut self,
        work_order: &WorkOrders,
        strategic_state: Vec<(u32, Period)>,
    ) {
        let mut optimized_work_orders = HashMap::new();
        for (work_order_id, period) in strategic_state {
            let work_order = work_order.inner.get(&work_order_id).unwrap();

            let mut optimized_work_order = OptimizedTacticalWorkOrder {
                optimized_activities: HashMap::new(),
                relations: work_order.relations().clone(),
                weight: work_order.work_order_weight(),
                scheduled_period: period,
            };

            for (activity, operation) in work_order.operations() {
                let optimized_operation = OptimizedOperation {
                    work_order_id,
                    scheduled_start: 0,
                    scheduled_end: 0,
                    number: operation.number(),
                    duration: operation.duration(),
                    operating_time: operation.operating_time(),
                    work_remaining: operation.work_remaining(),
                    resource: operation.resource().clone(),
                };
                optimized_work_order
                    .optimized_activities
                    .insert(*activity, optimized_operation);
            }
            optimized_work_orders.insert(work_order_id, optimized_work_order);
        }

        self.number_of_orders = optimized_work_orders.len() as u32;
        self.optimized_work_orders = optimized_work_orders;

        debug!(
            "Number of work orders in TacticalAgent: {}",
            self.number_of_orders
        );
    }

    pub fn loading(&self) -> &AlgorithmResources {
        &self.loading
    }

    pub fn capacity(&self) -> &AlgorithmResources {
        &self.capacity
    }
}

impl LargeNeighborHoodSearch for TacticalAlgorithm {
    type SchedulingMessage = TacticalSchedulingMessage;
    type ResourceMessage = TacticalResourceMessage;
    type TimeMessage = TacticalTimeMessage;
    type Error = AgentError;

    fn objective_value(&self) -> f64 {
        self.objective_value
    }

    fn schedule(&mut self) {
        for (work_order_number, work_order) in self.optimized_work_orders.iter() {
            self.priority_queue
                .push(*work_order_number, work_order.weight);
        }

        let mut start_day_index = 0;

        let mut loop_state: LoopState = LoopState::Scheduled;

        let mut current_work_order_number = match self.priority_queue.pop() {
            Some((work_order_number, _)) => work_order_number,
            None => {
                return;
            }
        };

        'main: loop {
            let optimized_work_order = match loop_state {
                LoopState::Unscheduled => self
                    .optimized_work_orders
                    .get(&current_work_order_number)
                    .unwrap(),
                LoopState::Scheduled => {
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

            let mut allowed_days = self.tactical_days.clone();
            let allowed_starting_days: Vec<&Day> = allowed_days
                .iter()
                .filter(|date| {
                    optimized_work_order.scheduled_period.start_date() <= &date.date
                        && &date.date <= optimized_work_order.scheduled_period.end_date()
                })
                .collect();

            let start_day: Day = allowed_starting_days[start_day_index].clone();

            let _ = allowed_days
                .iter_mut()
                .filter(|date| start_day.date <= date.date);

            let mut work_order_load = HashMap::<Resources, HashMap<Day, f64>>::new();

            let mut current_day = allowed_days.into_iter().peekable();

            for (_activity, operation) in optimized_work_order.optimized_activities.iter() {
                let mut activity_load = HashMap::<Day, f64>::new();
                let resource = operation.resource.clone();

                let remaining_capacity =
                    match self.remaining_capacity(&resource, current_day.peek().unwrap().clone()) {
                        Some(remaining_capacity) => remaining_capacity,
                        None => {
                            start_day_index += 1;
                            loop_state = LoopState::Unscheduled;
                            continue 'main;
                        }
                    };

                let loadings = self.determine_load(
                    remaining_capacity,
                    operation.operating_time,
                    operation.work_remaining,
                );

                for load in loadings {
                    activity_load.insert(current_day.peek().unwrap().clone(), load);

                    current_day.next();
                    if self
                        .remaining_capacity(&resource, current_day.peek().unwrap().clone())
                        .is_none()
                    {
                        start_day_index += 1;
                        loop_state = LoopState::Unscheduled;
                        continue 'main;
                    };
                }
                work_order_load.insert(resource, activity_load);
            }
            debug!(
                "Tactical Work Order {} has been scheduled starting on day {}",
                current_work_order_number, start_day.day_index
            );
            loop_state = LoopState::Scheduled;
            self.update_loadings(work_order_load);
        }
    }

    fn unschedule(&mut self, _message: u32) {
        // This is where the algorithm will unschedule the work orders.
    }

    fn update_scheduling_state(
        &mut self,
        _scheduling_message: Self::SchedulingMessage,
    ) -> Result<String, Self::Error> {
        Ok("".to_string())
        // This is where the algorithm will update the scheduling state.
    }

    fn update_time_state(
        &mut self,
        _time_message: Self::TimeMessage,
    ) -> Result<String, Self::Error> {
        // This is where the algorithm will update the time state.
        Ok("".to_string())
    }

    #[instrument(level = "info", skip_all)]
    fn update_resources_state(
        &mut self,
        resource_message: Self::ResourceMessage,
    ) -> Result<String, Self::Error> {
        match resource_message {
            TacticalResourceMessage::SetResources(resources) => {
                // The resources should be initialized together with the Agent itself
                for (resource, days) in resources {
                    for (day, capacity) in days {
                        let day: Day = match self
                            .tactical_days
                            .iter()
                            .find(|d| d.date().to_string() == day)
                        {
                            Some(day) => day.clone(),
                            None => {
                                return Err(AgentError::StateUpdateError(
                                    "Day not found in the tactical days".to_string(),
                                ));
                            }
                        };

                        *self.capacity.capacity_mut(&resource, &day) = capacity;
                    }
                }
                Ok("Resources have been updated".to_string())
            }
            TacticalResourceMessage::GetLoadings {
                days_end,
                select_resources: _,
            } => {
                let loading = self.loading();

                let days_end: u32 = days_end.parse().unwrap();

                Ok(loading.to_string(days_end))
            }
            TacticalResourceMessage::GetCapacities {
                days_end,
                select_resources: _,
            } => {
                let capacities = self.capacity();

                let days_end: u32 = days_end.parse().unwrap();

                Ok(capacities.to_string(days_end))
            }
        }
    }
}

enum LoopState {
    Unscheduled,
    Scheduled,
}

impl TacticalAlgorithm {
    fn update_loadings(&mut self, work_order_load: HashMap<Resources, HashMap<Day, f64>>) {
        for (resource, days) in work_order_load {
            for (day, load) in days {
                let loading = self.loading.loading_mut(&resource, &day);
                *loading += load;
            }
        }
    }

    fn remaining_capacity(&self, resource: &Resources, day: Day) -> Option<f64> {
        let remaining_capacity =
            self.capacity.capacity(resource, &day) - self.loading.loading(resource, &day);
        if remaining_capacity < 0.0 {
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
        if operating_time < 0.0 {
            operating_time = 4.0;
            warn!("Operating time is less than 0.0. This is an error in the data initialization, setting it to 4.0 hours as default");
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
}

impl TacticalAlgorithm {
    pub fn status(&self) -> Result<String, AgentError> {
        Ok(format!(
            "Objective: {}\n
            Time horizon: {} days\n
            Number of work orders: {}",
            self.objective_value, self.time_horizon, self.number_of_orders,
        ))
    }

    pub fn get_objective_value(&self) -> f64 {
        self.objective_value
    }
}

#[allow(dead_code)]
enum OperationDifference {
    SameDay,
    DiffDay,
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    #[test]
    fn test_determine_load_1() {
        let tactical_algorithm = super::TacticalAlgorithm::new(
            vec![],
            super::AlgorithmResources::new(HashMap::new()),
            super::AlgorithmResources::new(HashMap::new()),
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
            super::AlgorithmResources::new(HashMap::new()),
            super::AlgorithmResources::new(HashMap::new()),
        );

        let remaining_capacity = 3.0;
        let operating_time = 0.0;
        let work_remaining = 10.0;

        let loadings =
            tactical_algorithm.determine_load(remaining_capacity, operating_time, work_remaining);

        assert_eq!(loadings, vec![0.0, 0.0, 0.0]);
    }
}
