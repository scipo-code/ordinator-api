pub mod availability;
pub mod crew;
pub mod resources;
pub mod worker;

use std::collections::HashSet;

use crew::AgentEnvironment;
use serde::Deserialize;
use serde::Serialize;
use strum::IntoEnumIterator;

use self::resources::Resources;

pub type OperationalId = String;
// There is something rotten about all this! I think that the best
// approach is to create something that will allow us to better
// forcast how the system will behave.
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct WorkerEnvironment
{
    pub agent_environment: AgentEnvironment,
    work_centers: HashSet<Resources>,
}

pub struct WorkerEnvironmentBuilder
{
    pub agent_environment: Option<AgentEnvironment>,
    work_centers: Option<HashSet<Resources>>,
}

impl WorkerEnvironment
{
    // TODO [ ]
    // This should be refactored!
    pub fn new() -> Self
    {
        let mut work_centers = HashSet::new();
        for resource in Resources::iter() {
            work_centers.insert(resource);
        }
        WorkerEnvironment {
            agent_environment: AgentEnvironment::default(),
            work_centers,
        }
    }
}

pub enum EmptyFull
{
    Empty,
    Full,
}

impl WorkerEnvironmentBuilder
{
    pub fn build(self) -> WorkerEnvironment
    {
        WorkerEnvironment {
            agent_environment: self.agent_environment.unwrap_or_default(),
            work_centers: self.work_centers.unwrap_or_default(),
        }
    }

    pub fn agent_environment(&mut self, agent_environment: AgentEnvironment) -> &mut Self
    {
        self.agent_environment = Some(agent_environment);
        self
    }
}
