use std::{cmp::Ordering, ops::Deref};

use std::collections::HashMap;

use chrono::{DateTime, Utc};
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
    capacity: HashMap<Resources, HashMap<Day, f64>>,
    loading: HashMap<Resources, HashMap<Day, f64>>,
    priority_queue: PriorityQueue<u32, u32>,
    dates: Vec<Day>,
}

struct OptimizedTacticalWorkOrder {
    optimized_activities: HashMap<u32, OptimizedOperation>,
    weight: u32,
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
            priority_queue: PriorityQueue::new(),
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

        let mut current_work_order_number = self.priority_queue.pop().unwrap().0;
        'main: loop {
            let optimized_work_order = match loop_state {
                LoopState::Unscheduled => self
                    .optimized_work_orders
                    .get(&current_work_order_number)
                    .unwrap(),
                LoopState::Scheduled => {
                    current_work_order_number = self.priority_queue.pop().unwrap().0;
                    self.optimized_work_orders
                        .get(&current_work_order_number)
                        .unwrap()
                }
            };

            let mut allowed_days = self.dates.clone();
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

            // How can I iterate through the days in a good way? So now days are the contain all the
            // days in which the work order can start, that is a crucial point to make. What should
            // we do about the code that makes it possible to schedule into the nest period? This
            // will also be crucial. The easiest path will be to simply, let it extend but then how
            // do we solve the problem of the... Ahh we should simply let the days variable above be
            // the one that determines the start date of the work order, yes that is a good approach
            let mut current_day = allowed_days.into_iter().peekable();
            for (activity, operation) in optimized_work_order.optimized_activities.iter() {
                let mut activity_load = HashMap::<Day, f64>::new();
                let resource = operation.resource.clone();
                let mut work_remaining = operation.work_remaining;

                let remaining_capacity = match self
                    .remaining_capacity(resource.clone(), current_day.peek().unwrap().clone())
                {
                    Some(remaining_capacity) => remaining_capacity,
                    None => {
                        start_day_index += 1;
                        loop_state = LoopState::Unscheduled;
                        continue 'main;
                    }
                };

                // So what is it that we want now? Remember being focused feels calm and empty. It
                // you do not feel this way then you are not focused. Prioritize meditation is this
                // case.

                // The first day load should not be the same as the other days. This is because the
                // first day is the one that yields and all the remaining days are the ones that are
                // follow suit. This means that the determine_load function should return an array
                // of loads. For each of the days. But what to do if there is no capacity left? then
                // it would move to the next day. Yes, what we are doing now could essentially be
                // wrong, in that a 12 hour operation with [2, 4, 4, 2] could be scheduled on like
                // [2, 1, 1, 1] and the function would not complain. This is not the way to go about
                // it. We need to make sure that the
                let first_day_load = self.determine_load(
                    remaining_capacity,
                    operation.operating_time,
                    &mut work_remaining,
                );

                activity_load.insert(current_day.peek().unwrap().clone(), first_day_load);

                for _ in 0..operation.duration {
                    current_day.next();
                    let remaining_capacity = match self
                        .remaining_capacity(resource.clone(), current_day.peek().unwrap().clone())
                    {
                        Some(remaining_capacity) => remaining_capacity,
                        None => {
                            start_day_index += 1;
                            loop_state = LoopState::Unscheduled;
                            continue 'main;
                        }
                    };
                    let load = self.determine_load(
                        remaining_capacity,
                        operation.operating_time,
                        &mut work_remaining,
                    );
                    activity_load.insert(current_day.peek().unwrap().clone(), load);
                }
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

    fn update_resources_state(
        &mut self,
        resource_message: Self::ResourceMessage,
    ) -> Result<String, Self::Error> {
        Ok("".to_string())
        // This is where the algorithm will update the resources state.
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
                let loading = self
                    .loading
                    .get_mut(&resource)
                    .unwrap()
                    .get_mut(&day)
                    .unwrap();
                *loading += load;
            }
        }
    }

    fn remaining_capacity(&self, resource: Resources, day: Day) -> Option<f64> {
        let remaining_capacity = self.capacity.get(&resource).unwrap().get(&day).unwrap()
            - self.loading.get(&resource).unwrap().get(&day).unwrap();

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
        work_remaining: &mut f64,
    ) -> Vec<f64> {
        let loadings = Vec::new();

        let first_day_load = match remaining_capacity.partial_cmp(&operating_time) {
            Some(Ordering::Less) => remaining_capacity,
            Some(Ordering::Equal) => remaining_capacity,
            Some(Ordering::Greater) => operating_time,
            None => panic!("remaining work and operating_time are not comparable. There is an error in the data initialization"),
        }.min(*work_remaining);

        loadings.push(first_day_load);
        // The problem is that we do not know if there will be enough capacity. The only logical
        // thing to do will be to make the load vector and then we will see if there is enough
        // capacity in the loop afterwards. Yes this is a good idea. Also, we should be allowed to
        // excced the capacity, but I am not really sure about the mechanism, though. This is so
        // exciting. I will meditate now.
        while work_remaining > &0.0 {
            work_remaining -= first_day_load;
        }

        *work_remaining -= load;
        load
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
