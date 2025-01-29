pub mod strategic_request_periods_message;
pub mod strategic_request_resources_message;
pub mod strategic_request_scheduling_message;
pub mod strategic_request_status_message;

pub mod strategic_response_periods;
pub mod strategic_response_resources;
pub mod strategic_response_scheduling;
pub mod strategic_response_status;

use anyhow::{ensure, Context, Result};
use clap::Subcommand;
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::{self},
};

use actix::Message;
use serde::{Deserialize, Serialize};
use serde_json_any_key::any_key_map;

use crate::{
    orchestrator::WorkOrdersStatus,
    scheduling_environment::{
        time_environment::period::Period,
        work_order::{operation::Work, status_codes::StrategicUserStatusCodes},
        worker_environment::resources::Resources,
    },
    Asset, ConstraintState, LoadOperation, OperationalId,
};

use self::{
    strategic_request_periods_message::StrategicTimeRequest,
    strategic_request_resources_message::{ManualResource, StrategicResourceRequest},
    strategic_request_scheduling_message::StrategicSchedulingRequest,
    strategic_request_status_message::StrategicStatusMessage,
    strategic_response_periods::StrategicResponsePeriods,
    strategic_response_resources::StrategicResponseResources,
    strategic_response_scheduling::StrategicResponseScheduling,
    strategic_response_status::StrategicResponseStatus,
};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct StrategicObjectiveValue {
    pub objective_value: u64,
    pub urgency: (u64, u64),
    pub resource_penalty: (u64, u64),
    pub clustering_value: (u64, u64),
}

impl Default for StrategicObjectiveValue {
    fn default() -> Self {
        Self {
            objective_value: u64::MAX,
            urgency: (u64::MAX, u64::MAX),
            resource_penalty: (u64::MAX, u64::MAX),
            clustering_value: (u64::MAX, u64::MAX),
        }
    }
}

impl StrategicObjectiveValue {
    pub fn new(
        urgency: (u64, u64),
        resource_penalty: (u64, u64),
        clustering_value: (u64, u64),
    ) -> Self {
        Self {
            objective_value: 0,
            urgency,
            resource_penalty,
            clustering_value,
        }
    }

    pub fn aggregate_objectives(&mut self) {
        self.objective_value = self.urgency.0 * self.urgency.1
            + self.resource_penalty.0 * self.resource_penalty.1
            - self.clustering_value.0 * self.clustering_value.1;
    }
}

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
#[derive(Subcommand, Serialize, Deserialize, Clone, Debug)]
pub enum StrategicSchedulingEnvironmentCommands {
    UserStatus(StrategicUserStatusCodes),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum StrategicRequestMessage {
    Status(StrategicStatusMessage),
    Scheduling(StrategicSchedulingRequest),
    Resources(StrategicResourceRequest),
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
impl Message for StrategicRequestMessage {
    type Result = Result<StrategicResponseMessage>;
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

#[derive(Serialize)]
pub struct StrategicInfeasibleCases {
    pub respect_awsc: ConstraintState<String>,
    pub respect_unloading: ConstraintState<String>,
    pub respect_sch: ConstraintState<String>,
    pub respect_aggregated_load: ConstraintState<String>,
}

impl Default for StrategicInfeasibleCases {
    fn default() -> Self {
        StrategicInfeasibleCases {
            respect_awsc: ConstraintState::Infeasible("Infeasible".to_string()),
            respect_unloading: ConstraintState::Infeasible("Infeasible".to_string()),
            respect_sch: ConstraintState::Infeasible("Infeasible".to_string()),
            respect_aggregated_load: ConstraintState::Infeasible("Infeasible".to_string()),
        }
    }
}

// Where should the operational struct be found? I think that it should
// be in the shared types
#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct StrategicResources(
    #[serde(with = "any_key_map")] pub HashMap<Period, HashMap<OperationalId, OperationalResource>>,
);

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct OperationalResource {
    id: String,
    pub total_hours: Work,
    pub skill_hours: HashMap<Resources, Work>,
}

impl OperationalResource {
    pub fn new(id: String, total_hours: Work, skill_hours: HashMap<Resources, Work>) -> Self {
        Self {
            id,
            total_hours,
            skill_hours,
        }
    }
}

impl StrategicResources {
    pub fn assert_well_shaped_resources(&self) -> Result<()> {
        for period in &self.0 {
            for operational_resource in period.1 {
                let total_hours = operational_resource.1.total_hours;
                ensure!(
                    operational_resource
                        .1
                        .skill_hours
                        .values()
                        .all(|wor| *wor == total_hours),
                    format!(
                        "StrategicResources are not well shaped: {:#?}",
                        operational_resource.1
                    )
                )
            }
        }
        Ok(())
    }
}

impl StrategicResources {
    pub fn new(resources: HashMap<Period, HashMap<OperationalId, OperationalResource>>) -> Self {
        Self(resources)
    }

    // Okay so you have to determine a good way of updating the load here. The best approach
    // would probably be to create a small heuristic
    //
    // The load should be updated and this means that we need to generate a small heuristic.
    // As this is no longer deterministic.
    pub fn update_load(
        &mut self,
        period: &Period,
        resource: Resources,
        load: Work,
        operational_id: &(OperationalId, OperationalResource),
        load_operation: LoadOperation,
    ) -> Result<()> {
        let period_entry = self.0.entry(period.clone());
        let operational = match period_entry {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(HashMap::new()),
        };

        // Here the heuristic starts! I think that the best approach would be to
        // make the code work on. Okay I think that the code for the
        //
        // Do we need the &Resources here? Ideally no as that information is already
        // present in the operational_id of the problem. I think that the best approach
        // here is to leave it out. The resources in question is given uniquely by the
        // operational_id. There is something here that is not quite right. Be ready to
        // change it.
        // FIX:
        // We should make sure that the code here respects the new resource if it does not exist
        // QUESTION:
        // Where should this process occur? It does not change the understanding of the total
        match operational.entry(operational_id.0.clone()) {
            Entry::Occupied(mut operational_resource) => match load_operation {
                LoadOperation::Add => {
                    let previous_total_hours = operational_resource.get().total_hours;
                    operational_resource
                        .get_mut()
                        .skill_hours
                        .entry(resource)
                        .or_insert(previous_total_hours);

                    operational_resource.get_mut().total_hours += load;

                    operational_resource
                        .get_mut()
                        .skill_hours
                        .iter_mut()
                        .for_each(|ski_loa| *ski_loa.1 += load);
                }
                LoadOperation::Sub => {
                    let previous_total_hours = operational_resource.get().total_hours;
                    operational_resource
                        .get_mut()
                        .skill_hours
                        .entry(resource)
                        .or_insert(previous_total_hours);
                    operational_resource.get_mut().total_hours -= load;
                    operational_resource
                        .get_mut()
                        .skill_hours
                        .iter_mut()
                        .for_each(|ski_loa| *ski_loa.1 -= load);
                }
            },
            // I do not think that this should be here! The update_load here should be called the
            // update_or_create_load as that is what the function is doing! I think that if a new
            // load enters the system it should not be done through here. But through elsewhere.
            // FIX This should be possible to reach! I am not sure.
            Entry::Vacant(operational_resource_entry) => match load_operation {
                LoadOperation::Add => {
                    let total_load_hours = Work::from(load.to_f64());
                    let mut skill_hours: HashMap<Resources, Work> = HashMap::new();

                    operational_id
                        .1
                        .skill_hours
                        .keys()
                        .chain(std::iter::once(&resource))
                        .for_each(|res| {
                            skill_hours.insert(*res, total_load_hours);
                        });

                    let operational_resource = OperationalResource::new(
                        operational_id.0.clone(),
                        total_load_hours,
                        skill_hours,
                    );

                    operational_resource_entry.insert(operational_resource);
                }
                LoadOperation::Sub => {
                    let total_load_hours = Work::from(-load.to_f64());
                    let mut skill_hours = HashMap::new();

                    operational_id
                        .1
                        .skill_hours
                        .keys()
                        .chain(std::iter::once(&resource))
                        .for_each(|res| {
                            skill_hours.insert(*res, total_load_hours);
                        });

                    let operational_resource = OperationalResource::new(
                        operational_id.0.clone(),
                        total_load_hours,
                        skill_hours,
                    );
                    operational_resource_entry.insert(operational_resource);
                }
            },
        };
        Ok(())
    }

    pub fn update_resource_capacities(&mut self, resources: Self) -> Result<()> {
        for period in &resources.0 {
            for operational in period.1 {
                self.0
                    .entry(period.0.clone())
                    .or_default()
                    .entry(operational.0.clone())
                    .and_modify(|existing| *existing = operational.1.clone())
                    .or_insert(operational.1.clone());
            }
        }
        Ok(())
    }

    pub fn initialize_resource_loadings(&mut self, resources: Self) {
        for period in resources.0 {
            for operational in period.1 {
                let mut operational_resource = operational.1;

                operational_resource.total_hours = Work::from(0.0);

                operational_resource
                    .skill_hours
                    .iter_mut()
                    .for_each(|ele| *ele.1 = Work::from(0.0));

                self.0
                    .entry(period.0.clone())
                    .or_default()
                    .entry(operational.0.clone())
                    .and_modify(|existing| *existing = operational_resource.clone())
                    .or_insert(operational_resource);
            }
        }
    }

    pub fn aggregated_capacity_by_period_and_resource(
        &self,
        period: &Period,
        resource: &Resources,
    ) -> Result<Work> {
        Ok(self
            .0
            .get(period)
            .with_context(|| {
                format!(
                    "{} not found is {:?}",
                    period,
                    std::any::type_name::<StrategicResources>()
                )
            })?
            // WARN START HERE
            .values()
            .fold(Work::from(0.0), |acc, or| {
                acc + *or.skill_hours.get(resource).unwrap_or(&Work::from(0.0))
            }))
    }
}
