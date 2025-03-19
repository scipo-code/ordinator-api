use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::worker_environment::OperationalId;
use ordinator_scheduling_environment::worker_environment::crew::AgentEnvironment;
use ordinator_scheduling_environment::worker_environment::crew::OperationalConfiguration;
use ordinator_scheduling_environment::worker_environment::crew::OperationalConfigurationAll;
use ordinator_scheduling_environment::worker_environment::crew::SupervisorConfigurationAll;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use serde::Deserialize;
use serde::Serialize;

// WARN
// This type if for initializing data based on the configuration
// .toml files. It should not be used as the structure for the
// `WorkerEnvironments` that is inappropriate.
// Good, so this type is an `API` types. And all API types
// should be located together. I think that is the best approach
// here.
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

impl From<ActorSpecifications> for AgentEnvironment {
    fn from(value: ActorSpecifications) -> Self {
        let operational = value
            .operational
            .into_iter()
            .map(|io| {
                let id = Id::new(&io.id, vec![], io.assets);
                let operational_config = OperationalConfigurationAll {
                    id: id.clone(),
                    hours_per_day: io.hours_per_day,
                    operational_configuration: io.operational_configuration,
                };

                (id, operational_config)
            })
            .collect();

        let supervisor = value
            .supervisors
            .into_iter()
            .map(|is| {
                // The umber of supervisor periods is misleading.
                let id = Id::new(&is.id, vec![], is.assets);
                let supervisor_config =
                    SupervisorConfigurationAll::new(id.clone(), is.number_of_supervisor_periods);
                (id.clone(), supervisor_config)
            })
            .collect();

        Self {
            operational,
            supervisor,
        }
    }
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
    pub number_of_removed_work_orders: u64,
    pub urgency: u64,
    pub resource_penalty: u64,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
pub struct SupervisorOptionsConfig {
    pub number_of_removed_work_orders: u64,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
pub struct OperationalOptionsConfig {
    pub number_of_removed_work_orders: u64,
}
