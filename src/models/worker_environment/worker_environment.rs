use std::collections::HashMap;
use crate::models::worker_environment::crew::Crew;
pub struct WorkerEnvironment {
    crew: Crew,
    work_centers: HashMap<String, f64>,
    multiskill_tracker: HashMap<(char, char), char>, // Assuming Tuple{Symbol, Symbol} translates to (char, char) in Rust.
}

impl WorkerEnvironment {
    pub fn new(crew: Crew) -> Self {
        let work_centers = HashMap::<String, f64>::new();
        let multiskill_tracker = HashMap::new();
        for (id, worker) in crew.get_workers() {
            *work_centers.entry(*worker.get_trait()).or_insert(0.0) += worker.get_capacity();
        }
        WorkerEnvironment {
            crew,
            work_centers,
            multiskill_tracker,
        }
    }
}