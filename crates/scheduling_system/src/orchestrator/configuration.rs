use shared_types::{
    scheduling_environment::work_order::WorkOrderConfigurations, ActorSpecifications,
};

use crate::agents::{
    operational_agent::OperationalOptions, strategic_agent::StrategicOptions,
    supervisor_agent::SupervisorOptions, tactical_agent::TacticalOptions,
};

/// This struct is used to load in all configuraions centrally into the Orchestrator.
/// The `Orchestrator` then uses dependency injection to provide the actors with the
/// correct `Configurations`.
///
// There is something that you do not understand here. Where should
// all these configurations go?
struct SystemConfigurations {
    work_order_configurations: WorkOrderConfigurations,
    actor_configurations: ActorConfigurations,
    actor_specification: ActorSpecifications,
}

// Okay the `Option`s are looking okay, the options
// are related to the functioning of the `Actor`s and
// the `Configuration`s are related to how the data in
// the `SchedulingEnvironment` is intrepreted by the
// actors, this is a completely different concern and
// should be handled as such. Good! Good progress.
// TODO [ ]
// We should remove the `Default` on all `Option`s
// and then move a file for each of them
struct ActorConfigurations {
    strategic_options: StrategicOptions,
    tactical_options: TacticalOptions,
    supervisor_options: SupervisorOptions,
    operational_options: OperationalOptions,
}
