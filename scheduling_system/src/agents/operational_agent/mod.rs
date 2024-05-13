use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use chrono::{DateTime, Utc};
use shared_messages::{
    agent_error::AgentError,
    models::worker_environment::resources::Id,
    operational::{
        operational_response_status::{self, OperationalResponseStatus},
        OperationalRequestMessage, OperationalResponseMessage,
    },
    StatusMessage, StopMessage,
};

use shared_messages::models::{work_order::operation::Operation, SchedulingEnvironment};
use tracing::warn;

use super::{supervisor_agent::SupervisorAgent, SetAddr, UpdateWorkOrderMessage};

#[allow(dead_code)]
pub struct OperationalAgent {
    id_operational: Id,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    operational_algorithm: OperationalAlgorithm,
    capacity: Option<f32>,
    availability: Option<Vec<Availability>>,
    assigned: Option<Vec<AssignedWork>>,
    backup_activities: Option<HashMap<u32, Operation>>,
    supervisor_agent_addr: Addr<SupervisorAgent>,
}

struct OperationalAlgorithm {
    objective_value: f64,
}

#[allow(dead_code)]
pub struct Availability {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

#[allow(dead_code)]
pub struct AssignedWork {
    work: u32,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl Actor for OperationalAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.supervisor_agent_addr.do_send(SetAddr::Operational(
            self.id_operational.clone(),
            ctx.address(),
        ));
    }
}

pub struct OperationalAgentBuilder {
    id_operational: Id,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    operational_algorithm: OperationalAlgorithm,
    capacity: Option<f32>,
    availability: Option<Vec<Availability>>,
    assigned: Option<Vec<AssignedWork>>,
    backup_activities: Option<HashMap<u32, Operation>>,
    supervisor_agent_addr: Addr<SupervisorAgent>,
}

impl OperationalAgentBuilder {
    pub fn new(
        id_operational: Id,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        supervisor_agent_addr: Addr<SupervisorAgent>,
    ) -> Self {
        OperationalAgentBuilder {
            id_operational,
            scheduling_environment,
            operational_algorithm: OperationalAlgorithm {
                objective_value: 0.0,
            },
            capacity: None,
            availability: None,
            assigned: None,
            backup_activities: None,
            supervisor_agent_addr,
        }
    }

    #[allow(dead_code)]
    pub fn with_capacity(mut self, capacity: f32) -> Self {
        self.capacity = Some(capacity);
        self
    }

    #[allow(dead_code)]
    pub fn with_availability(mut self, availability: Vec<Availability>) -> Self {
        self.availability = Some(availability);
        self
    }

    #[allow(dead_code)]
    pub fn with_assigned(mut self, assigned: Vec<AssignedWork>) -> Self {
        self.assigned = Some(assigned);
        self
    }

    #[allow(dead_code)]
    pub fn with_backup_activities(mut self, backup_activities: HashMap<u32, Operation>) -> Self {
        self.backup_activities = Some(backup_activities);
        self
    }

    pub fn build(self) -> OperationalAgent {
        OperationalAgent {
            id_operational: self.id_operational,
            scheduling_environment: self.scheduling_environment,
            operational_algorithm: self.operational_algorithm,
            capacity: self.capacity,
            availability: self.availability,
            assigned: self.assigned,
            backup_activities: self.backup_activities,
            supervisor_agent_addr: self.supervisor_agent_addr,
        }
    }
}

impl Handler<OperationalRequestMessage> for OperationalAgent {
    type Result = Result<OperationalResponseMessage, AgentError>;

    fn handle(
        &mut self,
        request: OperationalRequestMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        match request {
            OperationalRequestMessage::Status => {
                let operational_response_status = OperationalResponseStatus::new(
                    self.id_operational.clone(),
                    self.assigned.as_ref().unwrap().len(),
                    self.operational_algorithm.objective_value,
                );
                Ok(OperationalResponseMessage::Status(
                    operational_response_status,
                ))
            }
        }
    }
}

impl Handler<StatusMessage> for OperationalAgent {
    type Result = String;

    fn handle(&mut self, _msg: StatusMessage, _ctx: &mut Self::Context) -> Self::Result {
        format!(
            "ID: {}, traits: {}, Objective: {}",
            self.id_operational.0,
            self.id_operational
                .1
                .iter()
                .map(|resource| resource.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.operational_algorithm.objective_value
        )
    }
}

impl Handler<StopMessage> for OperationalAgent {
    type Result = ();

    fn handle(&mut self, _msg: StopMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}

impl Handler<UpdateWorkOrderMessage> for OperationalAgent {
    type Result = ();

    fn handle(
        &mut self,
        _update_work_order: UpdateWorkOrderMessage,

        _ctx: &mut Self::Context,
    ) -> Self::Result {
        // todo!();
        warn!("Update 'impl Handler<UpdateWorkOrderMessage> for SupervisorAgent'");
    }
}
