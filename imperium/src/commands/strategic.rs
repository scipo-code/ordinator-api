use std::collections::HashMap;

use clap::Args;
use clap::Subcommand;
use reqwest::blocking::Client;
use serde::Deserialize;
use shared_messages::resources::Resources;
use shared_messages::strategic::strategic_resources_message;
use shared_messages::strategic::strategic_resources_message::StrategicResourceMessage;
use shared_messages::strategic::strategic_scheduling_message::SingleWorkOrder;
use shared_messages::strategic::strategic_scheduling_message::StrategicSchedulingMessage;
use shared_messages::strategic::strategic_status_message::StrategicStatusMessage;
use shared_messages::strategic::StrategicRequest;
use shared_messages::strategic::StrategicRequestMessage;
use shared_messages::Asset;
use shared_messages::SystemMessages;
use shared_messages::TomlResources;
use strum::IntoEnumIterator;

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
        resource: String,
        period: String,
        capacity: u32,
    },

    /// Set the capacity policy of a resource (used for operation)
    SetCapacityPolicy { resource: String, capacity: u32 },
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
    Schedule(WorkOrderSchedule),
    /// Lock a period from any scheduling changes
    PeriodLock { period: String },
    /// Exclude a work order from a period
    Exclude { work_order: u32, period: String },
}

#[derive(Debug, Args)]
pub struct WorkOrderSchedule {
    pub work_order: u32,
    pub period: String,
}

impl StrategicCommands {
    pub fn execute(&self, client: &Client) -> SystemMessages {
        match self {
            StrategicCommands::Status {
                asset,
                status_commands,
            } => match status_commands {
                Some(StatusCommands::WorkOrders { period }) => {
                    let strategic_status_message: StrategicStatusMessage =
                        StrategicStatusMessage::new_period(period.to_string());

                    let strategic_request = StrategicRequest {
                        asset: asset.clone(),
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
                    let schedule_single_work_order =
                        SingleWorkOrder::new(schedule.work_order, schedule.period.clone());

                    let strategic_scheduling_message: StrategicSchedulingMessage =
                        StrategicSchedulingMessage::Schedule(schedule_single_work_order);

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
                SchedulingCommands::Exclude { work_order, period } => {
                    let exclude_single_work_order =
                        SingleWorkOrder::new(*work_order, period.clone());

                    let strategic_scheduling_message: StrategicSchedulingMessage =
                        StrategicSchedulingMessage::ExcludeFromPeriod(exclude_single_work_order);

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
                                resources.push(Resources::new_from_string(resource.clone()));
                            }
                            Some(resources)
                        }
                        None => None,
                    };

                    let strategic_resources_message = StrategicResourceMessage::GetLoadings {
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
                                resources.push(Resources::new_from_string(resource.clone()));
                            }
                            Some(resources)
                        }
                        None => None,
                    };

                    let strategic_resources_message = StrategicResourceMessage::GetCapacities {
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
                                resources.push(Resources::new_from_string(resource.clone()));
                            }
                            Some(resources)
                        }
                        None => None,
                    };

                    let strategic_resources_message =
                        StrategicResourceMessage::GetPercentageLoadings {
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
                        StrategicResourceMessage::new_set_resources(resources);

                    let strategic_request_message =
                        StrategicRequestMessage::Resources(strategic_resources_message);

                    let strategic_request = StrategicRequest {
                        asset: asset.clone(),
                        strategic_request_message,
                    };

                    SystemMessages::Strategic(strategic_request)
                }
            },
        }
    }
}

fn generate_manual_resources(
    client: &Client,
    toml_path: String,
) -> HashMap<Resources, HashMap<String, f64>> {
    let periods: Vec<String> = crate::commands::orchestrator::strategic_periods(client);
    let contents = std::fs::read_to_string(toml_path).unwrap();
    let config: TomlResources = toml::from_str(&contents).unwrap();

    let hours_per_day = 6.0;
    let days_in_period = 13.0;

    let gradual_reduction = |i: usize| -> f64 {
        if i == 0 {
            1.0
        } else if i == 1 {
            1.0
        } else if i == 2 {
            0.8
        } else {
            0.6
        }
    };

    let resource_specific = |resource: &Resources| -> f64 {
        match resource {
            Resources::Medic => config.Medic * hours_per_day * days_in_period, //0.0, //50.0,
            Resources::MtnCran => config.MtnCran * hours_per_day * days_in_period, //70.0,
            Resources::MtnElec => config.MtnElec * hours_per_day * days_in_period, //170.0,
            Resources::MtnInst => config.MtnInst * hours_per_day * days_in_period, //170.0,
            Resources::MtnLagg => config.MtnLagg * hours_per_day * days_in_period, //0.0, //300.0,
            Resources::MtnMech => config.MtnMech * hours_per_day * days_in_period, //350.0,
            Resources::MtnPain => config.MtnPain * hours_per_day * days_in_period, //0.0,   //300.0,
            Resources::MtnPipf => config.MtnPipf * hours_per_day * days_in_period, //0.0,   //300.0,
            Resources::MtnRigg => config.MtnRigg * hours_per_day * days_in_period, //200.0, //300.0,
            Resources::MtnRope => config.MtnRope * hours_per_day * days_in_period, //0.0,   //300.0,
            Resources::MtnRous => config.MtnRous * hours_per_day * days_in_period, //0.0,   //300.0,
            Resources::MtnSat => config.MtnSat * hours_per_day * days_in_period, //0.0,    //300.0,
            Resources::MtnScaf => config.MtnScaf * hours_per_day * days_in_period, //200.0, //300.0,
            Resources::MtnTele => config.MtnTele * hours_per_day * days_in_period, //170.0,
            Resources::MtnTurb => config.MtnTurb * hours_per_day * days_in_period, //80.0,
            Resources::InpSite => config.InpSite * hours_per_day * days_in_period, //300.0,
            Resources::Prodlabo => config.Prodlabo * hours_per_day * days_in_period, //0.0, //300.0,
            Resources::Prodtech => config.Prodtech * hours_per_day * days_in_period, //180.0,
            Resources::VenAcco => config.VenAcco * hours_per_day * days_in_period, //0.0,  //300.0,
            Resources::VenComm => config.VenComm * hours_per_day * days_in_period, //0.0,  //300.0,
            Resources::VenCran => config.VenCran * hours_per_day * days_in_period, //0.0,  //300.0,
            Resources::VenElec => config.VenElec * hours_per_day * days_in_period, //0.0,  //300.0,
            Resources::VenHvac => config.VenHvac * hours_per_day * days_in_period, //0.0,  //300.0,
            Resources::VenInsp => config.VenInsp * hours_per_day * days_in_period, //0.0,  //300.0,
            Resources::VenInst => config.VenInst * hours_per_day * days_in_period, //0.0,  //300.0,
            Resources::VenMech => config.VenMech * hours_per_day * days_in_period, //0.0,  //300.0,
            Resources::VenMete => config.VenMete * hours_per_day * days_in_period, //0.0,  //300.0,
            Resources::VenRope => config.VenRope * hours_per_day * days_in_period, //0.0,  //300.0,
            Resources::VenScaf => config.VenScaf * hours_per_day * days_in_period, //0.0,  //300.0,
            Resources::VenSubs => config.VenSubs * hours_per_day * days_in_period, //0.0,  //300.0,
            Resources::QaqcElec => config.QaqcElec * hours_per_day * days_in_period, //0.0, //300.0,
            Resources::QaqcMech => config.QaqcMech * hours_per_day * days_in_period, //0.0, //300.0,
            Resources::QaqcPain => config.QaqcPain * hours_per_day * days_in_period, //0.0, //300.0,
            Resources::WellSupv => config.WellSupv * hours_per_day * days_in_period, //0.0, //300.0,
        }
    };

    let mut resources_hash_map = HashMap::new();
    for resource in shared_messages::resources::Resources::iter() {
        let mut periods_hash_map = HashMap::new();
        for (i, period) in periods.clone().iter().enumerate() {
            periods_hash_map.insert(
                period.to_string(),
                resource_specific(&resource) * gradual_reduction(i),
            );
        }
        resources_hash_map.insert(resource, periods_hash_map);
    }
    resources_hash_map
}
