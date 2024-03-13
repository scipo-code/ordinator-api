use std::collections::HashMap;

use clap::Args;
use clap::Subcommand;
use reqwest::Client;
use shared_messages::resources::Resources;
use shared_messages::strategic::strategic_resources_message::StrategicResourceMessage;
use shared_messages::strategic::strategic_scheduling_message::SingleWorkOrder;
use shared_messages::strategic::strategic_scheduling_message::StrategicSchedulingMessage;
use shared_messages::strategic::strategic_status_message::StrategicStatusMessage;
use shared_messages::strategic::StrategicRequest;
use shared_messages::SystemMessages;
use strum::IntoEnumIterator;

#[derive(Subcommand, Debug)]
pub enum StrategicCommands {
    #[command(name = "default")]
    Default,
    /// Schedule one ScheduleIteration
    ScheduleIteration,

    /// overview of the strategic agent
    Status {
        #[clap(subcommand)]
        subcommand: StatusStrategic,
    },
    /// Scheduling commands
    Scheduling {
        #[clap(subcommand)]
        subcommand: SchedulingSubcommands,
    },
    /// Resources commands
    Resources {
        #[clap(subcommand)]
        subcommand: ResourcesSubcommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ResourcesSubcommands {
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

    /// Set the capacity of a resource
    SetCapacity {
        resource: String,
        period: String,
        capacity: u32,
    },

    /// Set the capacity policy of a resource (used for operation)
    SetCapacityPolicy { resource: String, capacity: u32 },
    /// Set the capacity policy to default (used for testing)
    SetCapacityPolicyDefault,
}

#[derive(Subcommand, Debug)]
pub enum StatusStrategic {
    /// List all work orders in a given period
    WorkOrders { period: String },
}

#[derive(Subcommand, Debug)]
pub enum SchedulingSubcommands {
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
    pub async fn execute(&self, client: &Client) -> SystemMessages {
        match self {
            StrategicCommands::Default => {
                let strategic_status_message: StrategicStatusMessage =
                    StrategicStatusMessage::General;

                SystemMessages::Strategic(StrategicRequest::Status(strategic_status_message))
            }
            StrategicCommands::ScheduleIteration => {
                let strategic_iteration_message = StrategicRequest::ScheduleIteration;
                SystemMessages::Strategic(strategic_iteration_message)
            }
            StrategicCommands::Status { subcommand } => match subcommand {
                StatusStrategic::WorkOrders { period } => {
                    let strategic_status_message: StrategicStatusMessage =
                        StrategicStatusMessage::new_period(period.to_string());

                    SystemMessages::Strategic(StrategicRequest::Status(strategic_status_message))
                }
            },
            StrategicCommands::Scheduling { subcommand } => match subcommand {
                SchedulingSubcommands::Schedule(schedule) => {
                    let schedule_single_work_order =
                        SingleWorkOrder::new(schedule.work_order, schedule.period.clone());

                    let strategic_scheduling_message: StrategicSchedulingMessage =
                        StrategicSchedulingMessage::Schedule(schedule_single_work_order);

                    let strategic_request =
                        StrategicRequest::Scheduling(strategic_scheduling_message);

                    SystemMessages::Strategic(strategic_request)
                }
                SchedulingSubcommands::PeriodLock { period: _ } => {
                    todo!()
                }
                SchedulingSubcommands::Exclude { work_order, period } => {
                    let exclude_single_work_order =
                        SingleWorkOrder::new(*work_order, period.clone());

                    let strategic_scheduling_message: StrategicSchedulingMessage =
                        StrategicSchedulingMessage::ExcludeFromPeriod(exclude_single_work_order);

                    let strategic_request =
                        StrategicRequest::Scheduling(strategic_scheduling_message);

                    SystemMessages::Strategic(strategic_request)
                }
            },
            StrategicCommands::Resources { subcommand } => match subcommand {
                ResourcesSubcommands::Loading {
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

                    let strategic_request =
                        StrategicRequest::Resources(strategic_resources_message);

                    SystemMessages::Strategic(strategic_request)
                }
                ResourcesSubcommands::Capacity {
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

                    let strategic_request =
                        StrategicRequest::Resources(strategic_resources_message);

                    SystemMessages::Strategic(strategic_request)
                }

                ResourcesSubcommands::SetCapacity {
                    resource: _,
                    period: _,
                    capacity: _,
                } => {
                    todo!()
                }
                ResourcesSubcommands::SetCapacityPolicy {
                    resource: _,
                    capacity: _,
                } => {
                    todo!()
                }
                ResourcesSubcommands::SetCapacityPolicyDefault => {
                    let resources = generate_manual_resources(client);

                    let strategic_resources_message =
                        StrategicResourceMessage::new_set_resources(resources.await);

                    let strategic_request =
                        StrategicRequest::Resources(strategic_resources_message);

                    SystemMessages::Strategic(strategic_request)
                }
            },
        }
    }
}

async fn generate_manual_resources(
    client: &Client,
) -> HashMap<shared_messages::resources::Resources, HashMap<String, f64>> {
    let periods: Vec<String> = crate::commands::orchestrator::get_periods(client).await;

    let gradual_reduction = |i: usize| -> f64 {
        if i == 0 {
            1.0
        } else if i == 1 {
            0.8
        } else {
            0.6
        }
    };

    let resource_specific = |resource: &Resources| -> f64 {
        match resource {
            Resources::Medic => 0.0, //50.0,
            Resources::MtnCran => 70.0,
            Resources::MtnElec => 170.0,
            Resources::MtnInst => 170.0,
            Resources::MtnLagg => 0.0, //300.0,
            Resources::MtnMech => 350.0,
            Resources::MtnPain => 0.0,   //300.0,
            Resources::MtnPipf => 0.0,   //300.0,
            Resources::MtnRigg => 200.0, //300.0,
            Resources::MtnRope => 0.0,   //300.0,
            Resources::MtnRous => 0.0,   //300.0,
            Resources::MtnSat => 0.0,    //300.0,
            Resources::MtnScaf => 200.0, //300.0,
            Resources::MtnTele => 170.0,
            Resources::MtnTurb => 80.0,
            Resources::InpSite => 300.0,
            Resources::Prodlabo => 0.0, //300.0,
            Resources::Prodtech => 180.0,
            Resources::VenAcco => 0.0,  //300.0,
            Resources::VenComm => 0.0,  //300.0,
            Resources::VenCran => 0.0,  //300.0,
            Resources::VenElec => 0.0,  //300.0,
            Resources::VenHvac => 0.0,  //300.0,
            Resources::VenInsp => 0.0,  //300.0,
            Resources::VenInst => 0.0,  //300.0,
            Resources::VenMech => 0.0,  //300.0,
            Resources::VenMete => 0.0,  //300.0,
            Resources::VenRope => 0.0,  //300.0,
            Resources::VenScaf => 0.0,  //300.0,
            Resources::VenSubs => 0.0,  //300.0,
            Resources::QaqcElec => 0.0, //300.0,
            Resources::QaqcMech => 0.0, //300.0,
            Resources::QaqcPain => 0.0, //300.0,
            Resources::WellSupv => 0.0, //300.0,
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
