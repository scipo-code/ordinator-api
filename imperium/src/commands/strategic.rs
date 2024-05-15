use std::collections::HashMap;

use clap::Args;
use clap::Subcommand;
use reqwest::blocking::Client;
use shared_messages::models::time_environment::period::Period;
use shared_messages::models::work_order::WorkOrderNumber;
use shared_messages::models::worker_environment::resources::Resources;
use shared_messages::strategic::strategic_request_resources_message::StrategicResourceMessage;
use shared_messages::strategic::strategic_request_scheduling_message::SingleWorkOrder;
use shared_messages::strategic::strategic_request_scheduling_message::StrategicSchedulingMessage;
use shared_messages::strategic::strategic_request_status_message::StrategicStatusMessage;
use shared_messages::strategic::Periods;
use shared_messages::strategic::StrategicRequest;
use shared_messages::strategic::StrategicRequestMessage;
use shared_messages::strategic::StrategicResources;
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
    Test {
        asset: Asset,
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
                    let strategic_status_message =
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
                    let work_order_number = WorkOrderNumber(schedule.work_order);
                    let schedule_single_work_order =
                        SingleWorkOrder::new(work_order_number, schedule.period.clone());

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
                    let work_order_number = WorkOrderNumber(*work_order);
                    let exclude_single_work_order =
                        SingleWorkOrder::new(work_order_number, period.clone());

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
            StrategicCommands::Test { asset } => {
                let strategic_request = StrategicRequest {
                    asset: asset.clone(),
                    strategic_request_message: StrategicRequestMessage::Test,
                };

                SystemMessages::Strategic(strategic_request)
            }
        }
    }
}

fn generate_manual_resources(client: &Client, toml_path: String) -> StrategicResources {
    let periods: Vec<Period> = crate::commands::orchestrator::strategic_periods(client);
    let contents = std::fs::read_to_string(toml_path).unwrap();
    let config: TomlResources = toml::from_str(&contents).unwrap();

    let hours_per_day = 6.0;
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

    let resource_specific = |resource: &Resources| -> f64 {
        match resource {
            Resources::Medic => config.medic * hours_per_day * days_in_period,
            Resources::MtnCran => config.mtncran * hours_per_day * days_in_period,
            Resources::MtnElec => config.mtnelec * hours_per_day * days_in_period,
            Resources::MtnInst => config.mtninst * hours_per_day * days_in_period,
            Resources::MtnLagg => config.mtnlagg * hours_per_day * days_in_period,
            Resources::MtnMech => config.mtnmech * hours_per_day * days_in_period,
            Resources::MtnPain => config.mtnpain * hours_per_day * days_in_period,
            Resources::MtnPipf => config.mtnpipf * hours_per_day * days_in_period,
            Resources::MtnRigg => config.mtnrigg * hours_per_day * days_in_period,
            Resources::MtnRope => config.mtnrope * hours_per_day * days_in_period,
            Resources::MtnRous => config.mtnrous * hours_per_day * days_in_period,
            Resources::MtnSat => config.mtnsat * hours_per_day * days_in_period,
            Resources::MtnScaf => config.mtnscaf * hours_per_day * days_in_period,
            Resources::MtnTele => config.mtntele * hours_per_day * days_in_period,
            Resources::MtnTurb => config.mtnturb * hours_per_day * days_in_period,
            Resources::InpSite => config.inpsite * hours_per_day * days_in_period,
            Resources::Prodlabo => config.prodlabo * hours_per_day * days_in_period,
            Resources::Prodtech => config.prodtech * hours_per_day * days_in_period,
            Resources::VenAcco => config.venacco * hours_per_day * days_in_period,
            Resources::VenComm => config.vencomm * hours_per_day * days_in_period,
            Resources::VenCran => config.vencran * hours_per_day * days_in_period,
            Resources::VenElec => config.venelec * hours_per_day * days_in_period,
            Resources::VenHvac => config.venhvac * hours_per_day * days_in_period,
            Resources::VenInsp => config.veninsp * hours_per_day * days_in_period,
            Resources::VenInst => config.veninst * hours_per_day * days_in_period,
            Resources::VenMech => config.venmech * hours_per_day * days_in_period,
            Resources::VenMete => config.venmete * hours_per_day * days_in_period,
            Resources::VenRope => config.venrope * hours_per_day * days_in_period,
            Resources::VenScaf => config.venscaf * hours_per_day * days_in_period,
            Resources::VenSubs => config.vensubs * hours_per_day * days_in_period,
            Resources::QaqcElec => config.qaqcelec * hours_per_day * days_in_period,
            Resources::QaqcMech => config.qaqcmech * hours_per_day * days_in_period,
            Resources::QaqcPain => config.qaqcpain * hours_per_day * days_in_period,
            Resources::WellSupv => config.wellsupv * hours_per_day * days_in_period,
        }
    };

    let mut resources_hash_map = HashMap::new();
    for resource in shared_messages::models::worker_environment::resources::Resources::iter() {
        let mut periods_hash_map = HashMap::<Period, f64>::new();
        for (i, period) in periods.clone().iter().enumerate() {
            periods_hash_map.insert(
                period.clone(),
                resource_specific(&resource) * gradual_reduction(i),
            );
        }
        resources_hash_map.insert(resource, Periods(periods_hash_map));
    }
    StrategicResources::new(resources_hash_map)
}
