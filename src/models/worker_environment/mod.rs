pub mod availability;
pub mod crew;
pub mod worker;

use std::collections::HashMap;
use crate::models::worker_environment::crew::Crew;
pub struct WorkerEnvironment {
    crew: Crew,
    work_centers: HashMap<String, f64>,
}

impl WorkerEnvironment {
    pub fn based_on_crew(crew: Crew) -> Self {
        let mut work_centers = HashMap::<String, f64>::new();
        for (id, worker) in crew.get_workers() {
            let worker_trait = worker.get_trait().clone();
            *work_centers.entry(worker_trait).or_insert(0.0) += worker.get_capacity();
        }
        WorkerEnvironment {
            crew,
            work_centers,
        }
    }

    pub fn based_on_workcenter(crew: Crew, work_centers: HashMap<String, f64>) -> Self {
        WorkerEnvironment {
            crew,
            work_centers,
        }
    }



}

impl WorkerEnvironment {
    pub fn new() -> Self {
        WorkerEnvironment {
            crew: Crew::new(),
            work_centers: HashMap::<String, f64>::new(),
        }
    }
}