use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use chrono::{DateTime, Utc};
use shared_messages::{
    resources::{Id, Resources},
    StatusMessage, StopMessage,
};

use crate::models::{work_order::operation::Operation, SchedulingEnvironment};

use super::{supervisor_agent::SupervisorAgent, SetAddr};

#[allow(dead_code)]
pub struct OperationalAgent {
    id: Id,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    operational_algorithm: OperationalAlgorithm,
    capacity: Option<f32>,
    availability: Option<Vec<Availability>>,
    assigned: Option<Vec<AssignedWork>>,
    backup_activities: Option<HashMap<u32, Operation>>,
    supervisor_agent_addr: Addr<SupervisorAgent>,
}

struct OperationalAlgorithm {
    objective_value: f32,
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
        self.supervisor_agent_addr
            .do_send(SetAddr::SetOperational(self.id.clone(), ctx.address()));
    }
}

pub struct OperationalAgentBuilder {
    id: Id,
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
        id: Id,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        supervisor_agent_addr: Addr<SupervisorAgent>,
    ) -> Self {
        OperationalAgentBuilder {
            id,
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
            id: self.id,
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

impl Handler<StatusMessage> for OperationalAgent {
    type Result = String;

    fn handle(&mut self, _msg: StatusMessage, _ctx: &mut Self::Context) -> Self::Result {
        format!(
            "ID: {}, traits: {}, Objective: {}",
            self.id.0,
            self.id
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
