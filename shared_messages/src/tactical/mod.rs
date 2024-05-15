pub mod tactical_resources_message;
pub mod tactical_response_resources;
pub mod tactical_response_scheduling;
pub mod tactical_response_status;
pub mod tactical_response_time;
pub mod tactical_scheduling_message;
pub mod tactical_status_message;
pub mod tactical_time_message;

use std::collections::HashMap;

use crate::{
    agent_error::AgentError,
    models::{time_environment::day::Day, worker_environment::resources::Resources},
    AlgorithmState, Asset, ConstraintState,
};
use actix::Message;
use serde::{Deserialize, Serialize};
use serde_json_any_key::*;
use std::fmt::Write;

use self::{
    tactical_resources_message::TacticalResourceMessage,
    tactical_response_resources::TacticalResponseResources,
    tactical_response_scheduling::TacticalResponseScheduling,
    tactical_response_status::TacticalResponseStatus, tactical_response_time::TacticalResponseTime,
    tactical_scheduling_message::TacticalSchedulingMessage,
    tactical_status_message::TacticalStatusMessage, tactical_time_message::TacticalTimeMessage,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TacticalRequest {
    pub asset: Asset,
    pub tactical_request_message: TacticalRequestMessage,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalRequestMessage {
    Status(TacticalStatusMessage),
    Scheduling(TacticalSchedulingMessage),
    Resources(TacticalResourceMessage),
    Days(TacticalTimeMessage),
    Test,
}

impl Message for TacticalRequestMessage {
    type Result = Result<TacticalResponseMessage, AgentError>;
}

#[derive(Serialize)]
pub struct TacticalResponse {
    asset: Asset,
    tactical_response_message: TacticalResponseMessage,
}

impl TacticalResponse {
    pub fn new(asset: Asset, tactical_response_message: TacticalResponseMessage) -> Self {
        Self {
            asset,
            tactical_response_message,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum TacticalResponseMessage {
    Status(TacticalResponseStatus),
    Scheduling(TacticalResponseScheduling),
    Resources(TacticalResponseResources),
    Time(TacticalResponseTime),
    Test(AlgorithmState<TacticalInfeasibleCases>),
}
#[derive(Debug, Clone, Serialize)]
pub struct TacticalInfeasibleCases {
    pub aggregated_load: ConstraintState<String>,
    pub earliest_start_day: ConstraintState<String>,
    pub all_scheduled: ConstraintState<String>,
    pub respect_period_id: ConstraintState<String>,
}

impl Default for TacticalInfeasibleCases {
    fn default() -> Self {
        TacticalInfeasibleCases {
            aggregated_load: ConstraintState::Infeasible("Infeasible".to_owned()),
            earliest_start_day: ConstraintState::Infeasible("Infeasible".to_owned()),
            all_scheduled: ConstraintState::Infeasible("Infeasible".to_owned()),
            respect_period_id: ConstraintState::Infeasible("Infeasible".to_owned()),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TacticalResources {
    #[serde(with = "any_key_map")]
    pub resources: HashMap<Resources, Days>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Days {
    #[serde(with = "any_key_map")]
    pub days: HashMap<Day, f64>,
}

impl Days {
    pub fn new(days: HashMap<Day, f64>) -> Self {
        Self { days }
    }

    pub fn get(&self, day: &Day) -> &f64 {
        self.days.get(day).unwrap()
    }

    pub fn get_mut(&mut self, day: &Day) -> &mut f64 {
        self.days.get_mut(day).unwrap()
    }
}

impl TacticalResources {
    pub fn new(resources: HashMap<Resources, Days>) -> Self {
        TacticalResources { resources }
    }

    // fn to_string(&self, number_of_periods: u32) -> String {
    //     let mut string = String::new();
    //     let mut days = self
    //         .resources
    //         .values()
    //         .flat_map(|inner_map| inner_map.keys())
    //         .collect::<Vec<_>>();
    //     days.sort();
    //     days.dedup();

    //     write!(string, "{:<12}", "Resource").ok();
    //     for (nr_day, day) in days.iter().enumerate().take(number_of_periods as usize) {
    //         match nr_day {
    //             0..=13 => write!(string, "{:>12}", day.date().date_naive().to_string()).ok(),
    //             14..=27 => write!(string, "{:>12}", day.date().date_naive().to_string()).ok(),
    //             _ => write!(string, "{:>12}", day.date().date_naive().to_string()).ok(),
    //         }
    //         .unwrap()
    //     }
    //     writeln!(string).ok();

    //     let mut sorted_resources: Vec<_> = self.resources.iter().collect();
    //     sorted_resources.sort_by(|resource_a, resource_b| {
    //         resource_a.0.to_string().cmp(&resource_b.0.to_string())
    //     });
    //     for resource in sorted_resources {
    //         let inner_map = self.resources.get(resource.0).unwrap();
    //         write!(string, "{:<12}", resource.0.variant_name()).unwrap();
    //         for (nr_day, day) in days.iter().enumerate().take(number_of_periods as usize) {
    //             let value = inner_map.get(day).unwrap();
    //             match nr_day {
    //                 0..=13 => write!(string, "{:>12}", value.round().to_string()).ok(),
    //                 14..=27 => write!(string, "{:>12}", value.round().to_string()).ok(),
    //                 _ => write!(string, "{:>12}", value.round()).ok(),
    //             }
    //             .unwrap();
    //         }
    //         writeln!(string).ok();
    // }
    // string
    // }

    pub fn new_from_data(resources: Vec<Resources>, tactical_days: Vec<Day>, load: f64) -> Self {
        let mut resource_capacity: HashMap<Resources, Days> = HashMap::new();
        for resource in resources {
            let mut days = HashMap::new();
            for day in tactical_days.iter() {
                days.insert(day.clone(), load);
            }

            resource_capacity.insert(resource, Days { days });
        }
        TacticalResources::new(resource_capacity)
    }
}
