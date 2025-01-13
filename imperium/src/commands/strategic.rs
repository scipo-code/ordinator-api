use std::str::FromStr;

use clap::Args;
use clap::Subcommand;
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use shared_types::scheduling_environment::worker_environment::resources::Resources;
use shared_types::strategic::strategic_request_resources_message::StrategicResourceRequest;
use shared_types::strategic::strategic_request_scheduling_message::ScheduleChange;
use shared_types::strategic::strategic_request_scheduling_message::StrategicSchedulingRequest;
use shared_types::strategic::strategic_request_status_message::StrategicStatusMessage;
use shared_types::strategic::StrategicRequest;
use shared_types::strategic::StrategicRequestMessage;
use shared_types::strategic::StrategicSchedulingEnvironmentCommands;
use shared_types::Asset;
use shared_types::SystemMessages;

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
        /// Format YYYY-Wxx-xx (e.g. 2024-W41-42)
        period: String,
        resource: Vec<Resources>,
        capacity: f64,
    },
}

#[derive(Subcommand, Debug)]
pub enum StatusCommands {
    /// List all work orders in a given period
    WorkOrders { period: String },
    /// List relevant information about a specific work order
    WorkOrder { work_order_number: u64 },
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
    pub fn execute(self) -> SystemMessages {
        match self {
            StrategicCommands::Status {
                asset,
                status_commands,
            } => match status_commands {
                Some(StatusCommands::WorkOrder { work_order_number }) => {
                    let strategic_status_message =
                        StrategicStatusMessage::WorkOrder(WorkOrderNumber(work_order_number));

                    let strategic_request = StrategicRequest {
                        asset,
                        strategic_request_message: StrategicRequestMessage::Status(
                            strategic_status_message,
                        ),
                    };

                    SystemMessages::Strategic(strategic_request)
                }
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
