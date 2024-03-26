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
use tracing::{info, instrument};

use crate::{
    agents::traits::LargeNeighborHoodSearch,
    models::{time_environment::period::Period, work_order::ActivityRelation, WorkOrders},
};

/// The TacticalAlgorithm contains everything that is needed to run the tactical algorithm. For this
/// to work we will need something that can cleanly represent the solution, as well as how they
/// should be initialized into the algorithm. What is the goal now? I think that the goal is to make
/// sure that. When the algorithm gets a message from the strategic message about a new work order,
/// the TacticalAgent should integrate it into his TacticalAlgorithm. This means that we need a
/// message that can update the state of the TacticalAlgorithm. Based on the SchedulingEnvironment.
///
/// The critical goal now is to construct the TacticalAlgorithm so that it can hold the
///
/// How should the TacticalAlgorithm represent a schedule? Having a hashmap of the activities is not
/// enough we need something that is richer. Should the relationships between the activities be
/// inside or outside of the optimized_activities? I think that it should be outside. That will make
/// the most sense. I think that the optimized_activities should be wrapped in a should an activity
/// relation be a internal to the activity? I like that idea. I will make it easier for an activity
/// to live its own life, and this I feel in my guts will be the right approach to take in the
/// tactical algorithm. I want this to be a very clean and as simple as possible implementation.
/// This means that I should work on. The thing is there will always only be the number of
/// activities minus one relations. This means that
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

/// The fundamental unit here is the day. Nothing else than the day is important. Should the data be
/// inside of the OprimizedOperation? I do not think that this is the best idea. I think that we
/// should strive to have all the data inside of the. Hmm... For now we will just put the data
/// inside of the OptimizedOperation.
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

// This should come from the scheduling environment, as that is the single point of entry into the
// application for these kinds of data.
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
                    number: operation.number,
                    duration: operation.duration,
                    operating_time: operation.operating_time,
                    work_remaining: operation.work_remaining,
                    resource: operation.resource.clone(),
                };
                optimized_work_order
                    .optimized_activities
                    .insert(*activity, optimized_operation);
            }
            optimized_work_orders.insert(work_order_id, optimized_work_order);
        }

        self.number_of_orders = optimized_work_orders.len() as u32;
        self.optimized_work_orders = optimized_work_orders;

        dbg!(self.number_of_orders);
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

    /// This scheduling methods will handle a complete scheduling of the work orders that are a part
    /// of the TacticalAlgorithm. This means that it will be the method that will be called when the
    /// agent is running pasively.
    ///
    /// This is the most beautiful thing. I can hear it talking to me
    ///
    /// We need to schedule all the work orders against the available resource, here the important
    /// thing to note is that the penality is also a part of the objective function, meaning that
    /// all the work orders that are in the scope will always be able to be scheduled.
    ///
    /// So all work orders will be scheduled, then question is whether or not we should allow hmm...
    /// The question becomes how to handle the boundary between the periods. I know that it is the
    /// start of the work order that counts, but beyond that I am not so sure. The penality will be
    /// applied on the days. Should the algorithm be able to exceed the penality, or should it
    /// postphone the work order and corresponding activities into the next week.
    ///
    /// Let us dive deep into this. I think that the algorithm should be able to exceed the penalty
    /// but only in specific circumstances. The question is what you would rather want
    ///     * Do you prefer to contain the work orders in the period, or do you want to map the
    /// work orders out and exceed the period? I think that the latter option is the way to go.
    ///
    /// We want to see how we are progressing with the work orders. What about the patterns?
    ///
    /// Should we make a day struct? Yes I think so. At the moment we are simply scheduling
    /// everything every time, I do not think that this is the most appropriate way to go about it.
    /// But I cannot determine if I should do it in some different way instead. Should
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

                for day_index in 0..operation.duration {
                    activity_load.insert(
                        current_day.peek().unwrap().clone(),
                        loadings[day_index as usize],
                    );

                    if self
                        .remaining_capacity(&resource, current_day.peek().unwrap().clone())
                        .is_none()
                    {
                        start_day_index += 1;
                        loop_state = LoopState::Unscheduled;
                        continue 'main;
                    };

                    activity_load.insert(
                        current_day.peek().unwrap().clone(),
                        loadings[day_index as usize],
                    );

                    current_day.next();
                }
                info!(
                    "Tactical Work Order {} has been scheduled starting on day {}",
                    current_work_order_number, start_day.day_index
                );
                work_order_load.insert(resource, activity_load);
            }
            loop_state = LoopState::Scheduled;
            self.update_loadings(work_order_load);
        }
    }

    fn unschedule(&mut self, message: u32) {
        // This is where the algorithm will unschedule the work orders.
    }

    fn update_scheduling_state(
        &mut self,
        scheduling_message: Self::SchedulingMessage,
    ) -> Result<String, Self::Error> {
        Ok("".to_string())
        // This is where the algorithm will update the scheduling state.
    }

    fn update_time_state(
        &mut self,
        time_message: Self::TimeMessage,
    ) -> Result<String, Self::Error> {
        // This is where the algorithm will update the time state.
        Ok("".to_string())
    }

    #[instrument(level = "info", skip_all)]
    fn update_resources_state(
        &mut self,
        resource_message: Self::ResourceMessage,
    ) -> Result<String, Self::Error> {
        dbg!();
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
        dbg!();
        if remaining_capacity < 0.0 {
            None
        } else {
            Some(remaining_capacity)
        }
    }

    fn determine_load(
        &self,
        remaining_capacity: f64,
        operating_time: f64,
        mut work_remaining: f64,
    ) -> Vec<f64> {
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
