use std::collections::HashMap;

use shared_messages::resources::Resources;

/// The TacticalAlgorithm contains everything that is needed to run the tactical algorithm. For this
/// to work we will need something that can cleanly represent the solution, as well as how they
/// should be initialized into the algorithm. What is the goal now? I think that the goal is to make
/// sure that. When the algorithm gets a message from the strategic message about a new work order,
/// the TacticalAgent should integrate it into his TacticalAlgorithm. This means that we need a
/// message that can update the state of the TacticalAlgorithm. Based on the SchedulingEnvironment.
///
/// The critical goal now is to construct the TacticalAlgorithm so that it can hold the
pub struct TacticalAlgorithm {
    objective_value: f32,
    time_horizon: u32,
    number_of_orders: u32,
    optimized_work_orders: HashMap<u32, OptimizedActivity>,
    capacity: HashMap<Resources, HashMap<u32, f32>>,
}

/// The fundamental unit here is the day. Nothing else than the day is important.
struct OptimizedActivity {
    scheduled_start: u32,
    scheduled_end: u32,
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

    pub fn get_objective_value(&self) -> f32 {
        self.objective_value
    }
}
