pub mod availability;
pub mod crew;
pub mod resources;
pub mod worker;

use crate::models::worker_environment::crew::Crew;
use std::collections::HashMap;
#[allow(dead_code)]
pub struct WorkerEnvironment {
    pub crew: Crew,
    work_centers: HashMap<String, f64>,
}

impl WorkerEnvironment {
    #[allow(dead_code)]
    pub fn based_on_crew(crew: Crew) -> Self {
        let mut work_centers = HashMap::<String, f64>::new();
        for worker in crew.get_workers().values() {
            let worker_trait = worker.get_trait().clone();
            *work_centers.entry(worker_trait).or_insert(0.0) += worker.get_capacity();
        }
        WorkerEnvironment { crew, work_centers }
    }

    pub fn based_on_workcenter(crew: Crew, work_centers: HashMap<String, f64>) -> Self {
        WorkerEnvironment { crew, work_centers }
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
