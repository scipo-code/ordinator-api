pub mod availability;
pub mod crew;
pub mod resources;
pub mod worker;

use std::collections::HashSet;

use strum::IntoEnumIterator;

use crate::models::worker_environment::crew::Crew;

use self::resources::Resources;
pub struct WorkerEnvironment {
    crew: Crew,
    work_centers: HashSet<Resources>,
}

impl WorkerEnvironment {
    pub fn new() -> Self {
        let mut work_centers = HashSet::new();
        for resource in Resources::iter() {
            work_centers.insert(resource);
        }
        WorkerEnvironment {
            crew: Crew::new(),
            work_centers,
        }
    }

    pub fn get_crew(&self) -> &Crew {
        &self.crew
    }

    pub fn get_work_centers(&self) -> &HashSet<Resources> {
        &self.work_centers
    }
}
