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
// here. This should be the same. I think that it would be better
// to only have a single
// TODO [ ]
// Move this to the constracts. Or the configuration. What is the best choice?
// I think that the best choice is the configuration, but does the configuration
// depend on the contracts?
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TomlTimeInterval {
    pub start: toml::value::Datetime,
    pub end: toml::value::Datetime,
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
