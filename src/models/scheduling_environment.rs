use crate::models::work_order::work_order::WorkOrder;
use crate::models::worker_environment::crew::Crew;
use crate::models::worker_environment::worker_environment::WorkerEnvironment;
use std::collections::HashMap;

pub struct SchedulingEnvironment {
    work_orders: HashMap<u32, WorkOrder>,
    worker_environment: WorkerEnvironment,
    // Note: Fields like `time_and_period`, `material`, and `platform` have been omitted as their types are not provided.
}

impl SchedulingEnvironment {    
    pub fn initialize_from_sources(work_orders: HashMap<u32, WorkOrder>, crew: Crew) -> Self {
        let work_orders = HashMap::new();
        let worker_environment = WorkerEnvironment::new(crew);
        SchedulingEnvironment {
            work_orders,
            worker_environment,
        }
    }
}