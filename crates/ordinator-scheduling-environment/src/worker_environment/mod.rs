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
pub struct WorkerEnvironment {
    // I think that the actor environment is the correct term here.
    // Changes to the parameters should be changable in the application
    // itself. Where does that leave all this. Maybe we should actually
    // just make the... I think that we would require to make the. There
    // will be required some extreme logic here.
    pub actor_specification: HashMap<Asset, ActorSpecifications>,
}

pub struct WorkerEnvironmentBuilder {
    pub actor_specification: HashMap<Asset, ActorSpecifications>,
}

impl WorkerEnvironment {
    // TODO [ ]
    // This should be refactored!
    pub fn new() -> Self {
        WorkerEnvironment {
            actor_specification: HashMap::default(),
        }
    }
}

pub enum EmptyFull {
    Empty,
    Full,
}

impl WorkerEnvironmentBuilder {
    pub fn build(self) -> WorkerEnvironment {
        WorkerEnvironment {
            actor_specification: self.actor_specification.unwrap_or_default(),
        }
    }

    // We should insert... This builder is a little bothersome.
    // Ideally we need to provide a resource file for each of the different.
    // assets. That means that this should be callable many times over for
    // this to work.
    pub fn agent_environment(
        &mut self,
        asset: Asset,
        actor_specification: ActorSpecifications,
    ) -> &mut Self {
        self.actor_specification.insert(asset, actor_specification);
        self
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ActorSpecifications {
    pub strategic: InputStrategic,
    pub tactical: InputTactical,
    pub supervisors: Vec<InputSupervisor>,
    // QUESTION
    // Why not just store the OperationalParameters here?
    // Hmm... because the WorkOrders should not be part of this
    // what about the options? The options should be defined in
    // a separate config file
    // TODO [] Make separate config files for options
    pub operational: Vec<InputOperational>,
}
#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
pub struct InputStrategic {
    pub id: String,
    pub asset: String,
    pub strategic_options_config: StrategicOptionsConfig,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
pub struct InputTactical {
    pub id: String,
    pub asset: String,
    pub tactical_options_config: TacticalOptionsConfig,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
pub struct InputSupervisor {
    pub id: String,
    pub resource: Option<Resources>,
    pub number_of_supervisor_periods: u64,
    pub assets: Vec<Asset>,
    pub supervisor_options: SupervisorOptionsConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InputOperational {
    pub id: OperationalId,
    pub resources: Vec<Resources>,
    pub hours_per_day: f64,
    pub operational_configuration: OperationalConfiguration,
    pub assets: Vec<Asset>,
    pub operational_options: OperationalOptionsConfig,
}
/// This type is for loading in the `Strategic` configurations
/// so that the `StrategicOptions` can be loaded in to the `Agent`
/// in the correct format.
#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
pub struct StrategicOptionsConfig {
    pub number_of_removed_work_orders: usize,
    pub urgency_weight: usize,
    pub resource_penalty_weight: usize,
    pub clustering_weight: usize,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
pub struct TacticalOptionsConfig {
    pub number_of_removed_work_orders: usize,
    pub urgency: usize,
    pub resource_penalty: usize,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
pub struct SupervisorOptionsConfig {
    pub number_of_removed_work_orders: usize,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
pub struct OperationalOptionsConfig {
    pub number_of_removed_work_orders: usize,
}
