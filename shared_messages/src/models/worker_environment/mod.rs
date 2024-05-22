pub mod availability;
pub mod crew;
pub mod resources;
pub mod worker;

use std::{collections::HashSet};

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::models::worker_environment::crew::Crew;

use crate::models::worker_environment::resources::Resources;

#[derive(Serialize, Deserialize)]
pub struct WorkerEnvironment {
    crew: Option<Crew>,
    work_centers: HashSet<Resources>,
}

impl WorkerEnvironment {
    pub fn new() -> Self {
        let mut work_centers = HashSet::new();
        for resource in Resources::iter() {
            work_centers.insert(resource);
        }
        WorkerEnvironment {
            crew: Crew::new(None),
            work_centers,
        }
    }

    pub fn get_crew(&self) -> &Option<Crew> {
        &self.crew
    }

    pub fn get_work_centers(&self) -> &HashSet<Resources> {
        &self.work_centers
    }

    // We should create the worker environment from SAP based on the available crew.
    // The idea here is to simply load in the workers nothing else. The final goal is to make sure
    // that the worker environment will be used to initialize the resources for the algorithms.

    // What is it that I want here? I want to find a way to generate the resources from the crew.
    /// does this make anysense? Okay, I am think about creating a JSON file that will be used as an
    /// example of a crew. And then load this in as the Crew in the absence of external data. This
    /// will be the default crew.
    pub fn initialize(&mut self) {
        match self.crew {
            Some(ref mut _crew) => {
                // for (_, worker) in crew.get_workers().iter_mut() {
                //     for resource in worker.get_resources().iter() {
                //         self.work_centers.remove(resource);
                //     }
                // }
            }
            None => {
                // warn!("The json for the worker has been left out");
                // let worker_json =
                //     fs::read_to_string("scheduling_system/parameters/example_crew.json").unwrap();

                // dbg!(&worker_json);
                // let crew: Crew = serde_json::from_str(&worker_json).unwrap();

                // self.crew = Some(crew);
            }
        }
    }
}
