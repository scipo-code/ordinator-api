pub mod requests;
pub mod responses;

use anyhow::Result;
use clap::Subcommand;
use std::fmt::{self};
use strategic_request_periods_message::StrategicTimeRequest;
use strategic_request_resources_message::{ManualResource, StrategicRequestResource};
use strategic_request_scheduling_message::StrategicRequestScheduling;
use strategic_request_status_message::StrategicStatusMessage;
use strategic_response_periods::StrategicResponsePeriods;
use strategic_response_resources::StrategicResponseResources;
use strategic_response_scheduling::StrategicResponseScheduling;
use strategic_response_status::StrategicResponseStatus;

use serde::{Deserialize, Serialize};

use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::work_order::work_order_analytic::status_codes::StrategicUserStatusCodes;

use crate::orchestrator::WorkOrdersStatus;

use self::requests::*;
use self::responses::*;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "strategic_message_type")]
pub struct StrategicRequest {
    pub asset: Asset,
    pub strategic_request_message: StrategicRequestMessage,
}

impl StrategicRequest {
    pub fn asset(&self) -> &Asset {
        &self.asset
    }
}
// You should determine a better way of making this in the
// code I think that the best approach is to make something.
#[derive(Subcommand, Serialize, Deserialize, Clone, Debug)]
pub enum StrategicSchedulingEnvironmentCommands {
    UserStatus(StrategicUserStatusCodes),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum StrategicRequestMessage {
    Status(StrategicStatusMessage),
    Scheduling(StrategicRequestScheduling),
    Resources(StrategicRequestResource),
    Periods(StrategicTimeRequest),
    SchedulingEnvironment(StrategicSchedulingEnvironmentCommands),
}

#[derive(Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum StrategicResponseMessage {
    Status(StrategicResponseStatus),
    Scheduling(StrategicResponseScheduling),
    Resources(StrategicResponseResources),
    Periods(StrategicResponsePeriods),
    WorkOrder(WorkOrdersStatus),
    Success,
}

#[derive(Serialize)]
pub struct StrategicResponse {
    asset: Asset,
    strategic_response_message: StrategicResponseMessage,
}

impl StrategicResponse {
    pub fn new(asset: Asset, strategic_response_message: StrategicResponseMessage) -> Self {
        Self {
            asset,
            strategic_response_message,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimePeriod {
    pub period_string: String,
}

impl TimePeriod {
    pub fn get_period_string(&self) -> String {
        self.period_string.clone()
    }
}
impl fmt::Display for StrategicRequestMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            StrategicRequestMessage::Status(strategic_status_message) => {
                write!(f, "status: {}", strategic_status_message)?;
                Ok(())
            }
            StrategicRequestMessage::Scheduling(scheduling_message) => {
                write!(f, "scheduling_message: {:?}", scheduling_message)?;

                Ok(())
            }
            StrategicRequestMessage::Resources(_resources_message) => {
                // for manual_resource in resources_message.get_manual_resources().iter() {
                //     writeln!(f, "manual_resource: {:?}", manual_resource)?;
                // }
                Ok(())
            }
            StrategicRequestMessage::Periods(period_message) => {
                write!(f, "period_message: {:?}", period_message)?;
                Ok(())
            }
            _ => todo!(),
        }
    }
}

impl fmt::Display for ManualResource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "resource: {:?}, period: {}, capacity: {}",
            self.resource, self.period.period_string, self.capacity
        )
    }
}

impl TimePeriod {
    pub fn new(period_string: String) -> Self {
        Self { period_string }
    }
}

// #[derive(Serialize)]
// pub struct StrategicInfeasibleCases {
//     pub respect_awsc: ConstraintState<String>,
//     pub respect_unloading: ConstraintState<String>,
//     pub respect_sch: ConstraintState<String>,
//     pub respect_aggregated_load: ConstraintState<String>,
// }

// impl Default for StrategicInfeasibleCases {
//     fn default() -> Self {
//         StrategicInfeasibleCases {
//             respect_awsc: ConstraintState::Infeasible("Infeasible".to_string()),
//             respect_unloading: ConstraintState::Infeasible("Infeasible".to_string()),
//             respect_sch: ConstraintState::Infeasible("Infeasible".to_string()),
//             respect_aggregated_load: ConstraintState::Infeasible("Infeasible".to_string()),
//         }
//     }
// }
