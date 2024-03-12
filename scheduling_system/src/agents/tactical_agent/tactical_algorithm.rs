use std::collections::HashMap;

use shared_messages::resources::Resources;

use crate::models::{
    time_environment::period::Period,
    work_order::{self, ActivityRelation},
    WorkOrders,
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
    time_horizon: u32,
    number_of_orders: u32,
    optimized_work_orders: HashMap<u32, OptimizedTacticalWorkOrder>,
    capacity: HashMap<Resources, HashMap<u32, f64>>,
}

struct OptimizedTacticalWorkOrder {
    optimized_activities: HashMap<u32, OptimizedOperation>,
    relations: Vec<ActivityRelation>,
    scheduled_period: Period,
}

/// The fundamental unit here is the day. Nothing else than the day is important.
struct OptimizedOperation {
    work_order_id: u32,
    scheduled_start: u32,
    scheduled_end: u32,
    number: u32,
    operating_time: f64,
}

impl TacticalAlgorithm {
    pub fn new() -> Self {
        TacticalAlgorithm {
            objective_value: 0.0,
            time_horizon: 56,
            number_of_orders: 0,
            optimized_work_orders: HashMap::new(),
            capacity: HashMap::new(),
        }
    }

    pub fn update_state_based_on_strategic(
        &mut self,
        work_order: &WorkOrders,
        strategic_state: Vec<(u32, Period)>,
    ) {
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
                    number: operation.get_number(),
                    operating_time: operation.get_operating_time(),
                };
                optimized_work_order
                    .optimized_activities
                    .insert(*activity, optimized_operation);
            }

            self.optimized_work_orders
                .insert(work_order_id, optimized_work_order);
        }

        self.number_of_orders = self.optimized_work_orders.len() as u32;
        dbg!(self.number_of_orders);
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
