pub mod availability;
pub mod crew;
pub mod resources;
pub mod worker;

use std::collections::HashMap;

use crew::OperationalConfiguration;
use resources::Id;
use serde::Deserialize;
use serde::Serialize;

use crate::Asset;
use crate::time_environment::MaterialToPeriod;
use crate::work_order::WorkOrderConfigurations;

pub type OperationalId = String;
// There is something rotten about all this! I think that the best
// approach is to create something that will allow us to better
// forcast how the system will behave.
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct WorkerEnvironment {
    // I think that the actor environment is the correct term here.
    // Changes to the parameters should be changable in the application
    // itself. Where does that leave all this. Maybe we should actually
    // just make the... I think that we would require to make the. There
    // will be required some extreme logic here.
    pub actor_specification: HashMap<Asset, ActorSpecifications>,
}

pub struct WorkerEnvironmentBuilder {
    pub actor_environment: HashMap<Asset, ActorSpecifications>,
}

impl WorkerEnvironment {
    // TODO [ ]
    // This should be refactored!
    pub fn builder() -> WorkerEnvironmentBuilder {
        WorkerEnvironmentBuilder {
            actor_environment: HashMap::default(),
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
            actor_specification: self.actor_environment,
        }
    }

    // We should insert... This builder is a little bothersome.
    // Ideally we need to provide a resource file for each of the different.
    // assets. That means that this should be callable many times over for
    // this to work.
    pub fn actor_environment(mut self, asset: Asset) -> Self {
        // This should then be changed into something different for this to
        // work. You need to put it into the Asset and the ... I think that
        // it is okay to simply hard code the information for now. Hmm...
        // The issues comes from the difference between using the toml file
        // for initialization and using it for data storage... I think that
        // for now you should simply follow the same model that is used in
        // for the work orders: If the database file is missing you should
        // perform a complete reinitialization of the system. And if not
        // you should simply use the JSON file.
        //
        // For now the most important thing is getting all the data into the
        // `SchedulingEnvironment`
        // WARN This should not be needed to solve this problem. Keep it for now
        // DATE 2025-05-01
        // let list_of_actor_specification = vec![
        //     (
        //         Asset::DF,
        //         "./configuration/actor_specification/actor_specification_df.toml",
        //     ),
        //     (
        //         Asset::HB,
        //         "./configuration/actor_specification/actor_specification_hb.toml",
        //     ),
        //     (
        //         Asset::HD,
        //         "./configuration/actor_specification/actor_specification_hd.toml",
        //     ),
        //     (
        //         Asset::Test,
        //         "./configuration/actor_specification/actor_specification_test.toml",
        //     ),
        //     (
        //         Asset::TE,
        //         "./configuration/actor_specification/actor_specification_te.toml",
        //     ),
        // ];

        // You should put the data into the toml? Yes I think that is the best approach
        // here.
        let asset_string = asset.to_string();

        let asset_string_format = asset_string
            .split('.')
            .next()
            .unwrap()
            .split('_')
            .last()
            .expect("This function splits the path by the '.'");

        let path = format!(
            "./temp_scheduling_environment_database/actor_specification/actor_specification_{}.toml",
            asset_string
        );

        let contents = std::fs::read_to_string(path).unwrap();
        let actor_specifications: ActorSpecifications = toml::from_str(&contents).unwrap();

        self.actor_environment.insert(asset, actor_specifications);
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
    // TODO [ ] Make separate config files for options
    pub operational: Vec<InputOperational>,
    // QUESTION [ ] Is this the way to do it?
    // It cannot be like this. The idea of a relational database is beginning
    // to make a lot of sense.
    pub work_order_configurations: WorkOrderConfigurations,
    pub time_input: TimeInput,
    pub material_to_period: MaterialToPeriod,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeInput {
    pub number_of_strategic_periods: u64,
    pub number_of_tactical_periods: u64,
    pub number_of_days: u64,
    pub number_of_supervisor_periods: u64,
}

// This should be handled as well. What should you do not? I think that a
// meditation session.
//

// TODO #00 #00 #03 [x] Move the `./configuration/work_order_parameters.json`
// here. Is this
#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct InputStrategic {
    pub id: Id,
    pub strategic_options_config: StrategicOptions,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct InputTactical {
    pub id: Id,
    pub tactical_options_config: TacticalOptions,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
pub struct InputSupervisor {
    pub id: Id,
    pub number_of_supervisor_periods: u64,
    pub supervisor_options: SupervisorOptions,
}

// TODO [ ]
// Load in the IDs directly.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InputOperational {
    pub id: Id,
    pub hours_per_day: f64,
    pub operational_configuration: OperationalConfiguration,
    pub operational_options: OperationalOptions,
}
/// This type is for loading in the `Strategic` configurations
/// so that the `StrategicOptions` can be loaded in to the `Agent`
/// in the correct format.
/// How to resolve this duplication? Do you want this in the database?
/// So you have already understood that this is the case. This is the
/// priority that you need to understand here.
// QUESTION [ ]
// What should you do about the `StdRng`? I think that the best approach
// here is to make the code. You have to make you own Deser
//
// It should leave. The five why was essential. Leave the code out of this. I think
// that the correct way of making this is the the Orchestrator should apply changes
//
// QUESTION [ ]
// So the key question here is whether the Actor will ever need to
// see the options? I do not believe that it is. Actuallu does the
// [`Orchestrator`] even need to know the Actors?
//
// The issue here is that you are afraid of using `dyn`. That is the
// main thing that you need to have more decoupling.
//
// This has to be Clone. Otherwise you will not be able to understand the
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct StrategicOptions {
    pub number_of_removed_work_order: usize,
    pub urgency_weight: usize,
    pub resource_penalty_weight: usize,
    pub clustering_weight: usize,
    // These two should go into the `SchedulingEnvironment` that means that
    // the code should strive to... This means that the StrategicAgent, would
    // simply import this directly into itself. There is no need for a
    //
    // You can move this directly from the scheduling environment into the
    // Actor. Is the a good idea? I think that it is
    pub work_order_configurations: WorkOrderConfigurations,
    pub material_to_period: MaterialToPeriod,
}

// The `rng` should not be inside of the `ordinator-scheduling-environment`
#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct TacticalOptions {
    pub number_of_removed_work_orders: usize,
    pub urgency: usize,
    pub resource_penalty: usize,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct SupervisorOptions {
    pub number_of_unassigned_work_orders: usize,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct OperationalOptions {
    pub number_of_removed_activities: usize,
}
#[cfg(test)]
mod tests {
    use chrono::NaiveTime;

    use crate::worker_environment::ActorSpecifications;
    use crate::worker_environment::resources::Resources;

    #[test]
    fn test_toml_operational_parsing() {
        let toml_operational_string = r#"
            [[supervisors]]
            id = "main"
            number_of_supervisAgentEnvironmentr_periods = 3

            # [[supervisors]]
            # id = "supervisor-second"
            ################################
            ###          MTN-ELEC        ###
            ################################
            [[operational]]
            id = "OP-01-001"
            resources.resources = ["MTN-ELEC" ]
            hours_per_day = 6.0
            operational_configuration.off_shift_interval = { start = "19:00:00",  end = "07:00:00" }
            operational_configuration.break_interval = { start = "11:00:00", end = "12:00:00" }
            operational_configuration.toolbox_interval = { start = "07:00:00", end = "08:00:00" }
            operational_configuration.availability.start_date = "2024-12-02T07:00:00Z"
            operational_configuration.availability.finish_date = "2024-12-15T15:00:00Z"
        "#;

        let system_agents: ActorSpecifications = toml::from_str(toml_operational_string).unwrap();

        assert_eq!(system_agents.operational[0].id.0, "OP-01-001".to_string());

        assert_eq!(system_agents.operational[0].id.1, [Resources::MtnElec]);

        assert_eq!(
            system_agents.operational[0]
                .operational_configuration
                .off_shift_interval
                .start,
            NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
        );
        assert_eq!(
            system_agents.operational[0]
                .operational_configuration
                .off_shift_interval
                .end,
            NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
        );
    }
}
