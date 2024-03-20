use std::cmp::Ordering;

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use shared_messages::{
    agent_error::AgentError,
    resources::Resources,
    tactical::{
        tactical_resources_message::TacticalResourceMessage,
        tactical_scheduling_message::TacticalSchedulingMessage,
        tactical_time_message::TacticalTimeMessage,
    },
};
use tracing_subscriber::filter::combinator::Or;

use crate::{
    agents::{
        strategic_agent::strategic_algorithm::OptimizedWorkOrders, traits::LargeNeighborHoodSearch,
    },
    models::{
        time_environment::period::Period,
        work_order::{self, ActivityRelation},
        WorkOrders,
    },
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
    capacity: HashMap<Resources, HashMap<Day, f64>>,
    loading: HashMap<Resources, HashMap<Day, f64>>,
    dates: Vec<Day>,
}

struct OptimizedTacticalWorkOrder {
    optimized_activities: HashMap<u32, OptimizedOperation>,
    relations: Vec<ActivityRelation>,
    scheduled_period: Period,
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
#[derive(Eq, PartialEq, Hash, Clone)]
struct Day {
    day: usize,
    date: DateTime<Utc>,
}

impl TacticalAlgorithm {
    pub fn new(tactical_days: Vec<DateTime<Utc>>) -> Self {
        TacticalAlgorithm {
            objective_value: 0.0,
            time_horizon: tactical_days.len(),
            number_of_orders: 0,
            optimized_work_orders: HashMap::new(),
            capacity: HashMap::new(),
            loading: HashMap::new(),
            dates: Vec::new(),
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
                relations: work_order.get_relations().clone(),
                scheduled_period: period,
            };

            for (activity, operation) in work_order.get_operations() {
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
}

impl LargeNeighborHoodSearch for TacticalAlgorithm {
    type SchedulingMessage = TacticalSchedulingMessage;
    type ResourceMessage = TacticalResourceMessage;
    type TimeMessage = TacticalTimeMessage;
    type Error = AgentError;

    fn get_objective_value(&self) -> f64 {
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
    /// Should we make a day struct? Yes I think so.
    fn schedule(&mut self) {
        for work_order in self.optimized_work_orders.values() {
            let snapshot_work_order = work_order.to_owned();

            let dates_clone = self.dates.clone();
            // The first day is given by the scheduled period. Yes we should remember that.
            // You are feeling a little stressed about this.
            let allowed_starting_days: Vec<Day> = dates_clone
                .into_iter()
                .filter(|date| {
                    work_order.scheduled_period.start_date() <= &date.date
                        && &date.date <= work_order.scheduled_period.end_date()
                })
                .collect();

            let mut current_day: Day = allowed_starting_days.first().unwrap().clone();

            // How can I iterate through the days in a good way? So now days are the contain all the
            // days in which the work order can start, that is a crucial point to make. What should
            // we do about the code that makes it possible to schedule into the nest period? This
            // will also be crucial. The easiest path will be to simply, let it extend but then how
            // do we solve the problem of the... Ahh we should simply let the days variable above be
            // the one that determines the start date of the work order, yes that is a good approach

            for (activity, operation) in work_order.optimized_activities.iter() {
                let resource = operation.resource.clone();

                let load_pattern = Vec::new();

                let remaining_capacity = self
                    .capacity
                    .get(&resource)
                    .unwrap()
                    .get(&current_day)
                    .unwrap()
                    - self
                        .loading
                        .get(&resource)
                        .unwrap()
                        .get(&current_day)
                        .unwrap();

                let first_day_load = match remaining_capacity.partial_cmp(&operation.operating_time) {
                    Some(Ordering::Less) => remaining_capacity,
                    Some(Ordering::Equal) => remaining_capacity,
                    Some(Ordering::Greater) => operation.operating_time,
                    None => panic!("remaining work and operating_time are not comparable. There is an error in the data initialization"),
                };

                for data in 0..operation.duration {
                    if *self
                        .capacity
                        .get(&resource)
                        .unwrap()
                        .get(&current_day)
                        .unwrap()
                        > operation.work_remaining
                    {}
                }

                let load_pattern = self
                    .capacity
                    .get(&resource)
                    .unwrap()
                    .get(current_day)
                    .unwrap();

                for data in 0..operation.duration {
                    if *self
                        .capacity
                        .get(&resource)
                        .unwrap()
                        .get(current_day)
                        .unwrap()
                        > operation.work_remaining
                    {}
                }
            }
        }

        // This is where the algorithm will schedule the work orders.
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

    fn update_resources_state(
        &mut self,
        resource_message: Self::ResourceMessage,
    ) -> Result<String, Self::Error> {
        Ok("".to_string())
        // This is where the algorithm will update the resources state.
    }
}

impl TacticalAlgorithm {
    pub fn status(&self) -> String {
        format!(
            "Objective: {}\n
            Time horizon: {} days\n
            Number of work orders: {}",
            self.objective_value, self.time_horizon, self.number_of_orders,
        )
    }

    pub fn get_objective_value(&self) -> f64 {
        self.objective_value
    }
}

enum OperationDifference {
    SameDay,
    DiffDay,
}
