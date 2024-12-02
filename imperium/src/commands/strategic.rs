use std::collections::HashMap;
use std::str::FromStr;

use clap::Args;
use clap::Subcommand;
use reqwest::blocking::Client;
use shared_types::scheduling_environment::time_environment::period::Period;
use shared_types::scheduling_environment::work_order::operation::Work;
use shared_types::scheduling_environment::worker_environment::resources::Resources;
use shared_types::strategic::strategic_request_resources_message::StrategicResourceRequest;
use shared_types::strategic::strategic_request_scheduling_message::ScheduleChange;
use shared_types::strategic::strategic_request_scheduling_message::StrategicSchedulingRequest;
use shared_types::strategic::strategic_request_status_message::StrategicStatusMessage;
use shared_types::strategic::Periods;
use shared_types::strategic::StrategicRequest;
use shared_types::strategic::StrategicRequestMessage;
use shared_types::strategic::StrategicResources;
use shared_types::strategic::StrategicSchedulingEnvironmentCommands;
use shared_types::Asset;
use shared_types::SystemMessages;
use shared_types::TomlAgents;

#[derive(Subcommand, Debug)]
pub enum StrategicCommands {
    /// overview of the strategic agent
    Status {
        asset: Asset,
        #[clap(subcommand)]
        status_commands: Option<StatusCommands>,
    },
    /// Scheduling commands
    Scheduling {
        asset: Asset,
        #[clap(subcommand)]
        scheduling_commands: SchedulingCommands,
    },
    /// Resources commands
    Resources {
        asset: Asset,
        #[clap(subcommand)]
        resource_commands: ResourceCommands,
    },

    /// Access the Scheduling Environment with the options that the StrategicAgent can change
    StrategicSchedulingEnvironmentCommands {
        asset: Asset,
        #[clap(subcommand)]
        strategic_scheduling_environment_commands: StrategicSchedulingEnvironmentCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ResourceCommands {
    /// Get the loading of the resources
    Loading {
        periods_end: String,
        select_resources: Option<Vec<String>>,
    },

    /// Get the capacity of the resources
    Capacity {
        periods_end: String,
        select_resources: Option<Vec<String>>,
    },

    /// Get the percentage loading
    PercentageLoading {
        periods_end: String,
        select_resources: Option<Vec<String>>,
    },
    /// Set the capacity of a resource
    SetCapacity {
        resource: Resources,
        period: String,
        capacity: f64,
    },

    /// Set the capacity policy of a resource (used for operation)
    SetCapacityPolicy { resource: Resources, capacity: f64 },
    /// Set the capacity policy to default (used for testing)
    LoadCapacityFile { toml_path: String },
}

#[derive(Subcommand, Debug)]
pub enum StatusCommands {
    /// List all work orders in a given period
    WorkOrders { period: String },
}

#[derive(Subcommand, Debug)]
pub enum SchedulingCommands {
    /// Schedule a specific work order in a given period
    Schedule(ScheduleChange),
    /// Lock a period from any scheduling changes
    PeriodLock { period: String },
    /// Exclude a work order from a period
    Exclude(ScheduleChange),
}

#[derive(Debug, Args)]
pub struct WorkOrderSchedule {
    pub work_order: u64,
    pub period: String,
}

impl StrategicCommands {
    pub fn execute(self, client: &Client) -> SystemMessages {
        match self {
            StrategicCommands::Status {
                asset,
                status_commands,
            } => match status_commands {
                Some(StatusCommands::WorkOrders { period }) => {
                    let strategic_status_message =
                        StrategicStatusMessage::new_period(period.to_string());

                    let strategic_request = StrategicRequest {
                        asset,
                        strategic_request_message: StrategicRequestMessage::Status(
                            strategic_status_message,
                        ),
                    };

                    SystemMessages::Strategic(strategic_request)
                }
                None => {
                    let strategic_status_message: StrategicStatusMessage =
                        StrategicStatusMessage::General;

                    let strategic_request_message =
                        StrategicRequestMessage::Status(strategic_status_message);

                    let strategic_request = StrategicRequest {
                        asset: asset.clone(),
                        strategic_request_message,
                    };

                    SystemMessages::Strategic(strategic_request)
                }
            },
            StrategicCommands::Scheduling {
                asset,
                scheduling_commands: subcommand,
            } => match subcommand {
                SchedulingCommands::Schedule(schedule) => {
                    let strategic_scheduling_message: StrategicSchedulingRequest =
                        StrategicSchedulingRequest::Schedule(schedule);

                    let strategic_request_message =
                        StrategicRequestMessage::Scheduling(strategic_scheduling_message);

                    let strategic_request = StrategicRequest {
                        asset: asset.clone(),
                        strategic_request_message,
                    };

                    SystemMessages::Strategic(strategic_request)
                }
                SchedulingCommands::PeriodLock { period: _ } => {
                    todo!()
                }
                SchedulingCommands::Exclude(schedule_change) => {
                    let strategic_scheduling_message: StrategicSchedulingRequest =
                        StrategicSchedulingRequest::ExcludeFromPeriod(schedule_change);

                    let strategic_request_message =
                        StrategicRequestMessage::Scheduling(strategic_scheduling_message);

                    let strategic_request = StrategicRequest {
                        asset: asset.clone(),
                        strategic_request_message,
                    };

                    SystemMessages::Strategic(strategic_request)
                }
            },
            StrategicCommands::Resources {
                asset,
                resource_commands: subcommand,
            } => match subcommand {
                ResourceCommands::Loading {
                    periods_end,
                    select_resources,
                } => {
                    let resources = match select_resources {
                        Some(select_resources) => {
                            let mut resources: Vec<Resources> = vec![];
                            for resource in select_resources {
                                resources.push(Resources::from_str(&resource).unwrap());
                            }
                            Some(resources)
                        }
                        None => None,
                    };

                    let strategic_resources_message = StrategicResourceRequest::GetLoadings {
                        periods_end: periods_end.to_string(),
                        select_resources: resources,
                    };

                    let strategic_request_message =
                        StrategicRequestMessage::Resources(strategic_resources_message);

                    let strategic_request = StrategicRequest {
                        asset: asset.clone(),
                        strategic_request_message,
                    };

                    SystemMessages::Strategic(strategic_request)
                }
                ResourceCommands::Capacity {
                    periods_end,
                    select_resources,
                } => {
                    let resources = match select_resources {
                        Some(select_resources) => {
                            let mut resources: Vec<Resources> = vec![];
                            for resource in select_resources {
                                resources.push(Resources::from_str(&resource).unwrap());
                            }
                            Some(resources)
                        }
                        None => None,
                    };

                    let strategic_resources_message = StrategicResourceRequest::GetCapacities {
                        periods_end: periods_end.to_string(),
                        select_resources: resources,
                    };

                    let strategic_request_message =
                        StrategicRequestMessage::Resources(strategic_resources_message);

                    let strategic_request = StrategicRequest {
                        asset: asset.clone(),
                        strategic_request_message,
                    };

                    SystemMessages::Strategic(strategic_request)
                }

                ResourceCommands::PercentageLoading {
                    periods_end,
                    select_resources,
                } => {
                    let resources = match select_resources {
                        Some(select_resources) => {
                            let mut resources: Vec<Resources> = vec![];
                            for resource in select_resources {
                                resources.push(Resources::from_str(&resource).unwrap());
                            }
                            Some(resources)
                        }
                        None => None,
                    };

                    let strategic_resources_message =
                        StrategicResourceRequest::GetPercentageLoadings {
                            periods_end: periods_end.to_string(),
                            resources,
                        };

                    let strategic_request_message =
                        StrategicRequestMessage::Resources(strategic_resources_message);

                    let strategic_request = StrategicRequest {
                        asset: asset.clone(),
                        strategic_request_message,
                    };

                    SystemMessages::Strategic(strategic_request)
                }
                ResourceCommands::SetCapacity {
                    resource: _,
                    period: _,
                    capacity: _,
                } => {
                    todo!()
                    // let mut resources = HashMap::new();

                    // let mut periods: HashMap<Period, f64> = HashMap::new();

                    // periods.insert(period.clone(), *capacity);
                    // resources.insert(resource.clone(), periods);

                    // let strategic_resources_message =
                    //     StrategicResourceMessage::new_set_resources(resources);

                    // let strategic_request_message =
                    //     StrategicRequestMessage::Resources(strategic_resources_message);

                    // let strategic_request = StrategicRequest {
                    //     asset: asset.clone(),
                    //     strategic_request_message,
                    // };

                    // SystemMessages::Strategic(strategic_request)
                }
                ResourceCommands::SetCapacityPolicy {
                    resource: _,
                    capacity: _,
                } => {
                    todo!()
                }
                ResourceCommands::LoadCapacityFile { toml_path } => {
                    let resources = generate_manual_resources(client, toml_path.clone());

                    let strategic_resources_message =
                        StrategicResourceRequest::new_set_resources(resources);

                    let strategic_request_message =
                        StrategicRequestMessage::Resources(strategic_resources_message);

                    let strategic_request = StrategicRequest {
                        asset: asset.clone(),
                        strategic_request_message,
                    };

                    SystemMessages::Strategic(strategic_request)
                }
            },
            StrategicCommands::StrategicSchedulingEnvironmentCommands {
                asset,
                strategic_scheduling_environment_commands,
            } => {
                let strategic_request_message = StrategicRequestMessage::SchedulingEnvironment(
                    strategic_scheduling_environment_commands,
                );

                let strategic_request = StrategicRequest {
                    asset,
                    strategic_request_message,
                };
                SystemMessages::Strategic(strategic_request)
            }
        }
    }
}

// TODO: This really has to leave the system.
fn generate_manual_resources(client: &Client, toml_path: String) -> StrategicResources {
    let periods: Vec<Period> = crate::commands::orchestrator::strategic_periods(client);
    let contents = std::fs::read_to_string(toml_path).unwrap();

    let config: TomlAgents = toml::from_str(&contents).unwrap();

    let _hours_per_day = 6.0;
    let days_in_period = 13.0;

    let gradual_reduction = |i: usize| -> f64 {
        if i == 0 {
            1.0
        } else if i == 1 {
            0.9
        } else if i == 2 {
            0.8
        } else {
            0.6
        }
    };

    let mut resources_hash_map = HashMap::<Resources, Periods>::new();
    for operational_agent in config.operational {
        for (i, period) in periods.clone().iter().enumerate() {
            let resource_periods = resources_hash_map
                .entry(
                    operational_agent
                        .resources
                        .resources
                        .first()
                        .cloned()
                        .unwrap(),
                )
                .or_insert(Periods(HashMap::new()));

            *resource_periods.0.entry(period.clone()).or_insert_with(|| {
                Work::from(operational_agent.hours_per_day * days_in_period * gradual_reduction(i))
            }) +=
                Work::from(operational_agent.hours_per_day * days_in_period * gradual_reduction(i))
        }
    }
    StrategicResources::new(resources_hash_map)
}
