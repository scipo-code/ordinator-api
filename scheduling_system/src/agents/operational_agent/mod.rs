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

#[allow(dead_code)]
pub struct OperationalAgent {
    id: Id,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    operational_algorithm: OperationalAlgorithm,
    agent_traits: HashSet<Resources>,
    capacity: Option<f32>,
    availability: Option<Vec<Availability>>,
    assigned: Option<Vec<AssignedWork>>,
    backup_activities: Option<HashMap<u32, Operation>>,
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
}

pub struct OperationalAgentBuilder {
    id: Id,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    operational_algorithm: OperationalAlgorithm,
    agent_traits: HashSet<Resources>,
    capacity: Option<f32>,
    availability: Option<Vec<Availability>>,
    assigned: Option<Vec<AssignedWork>>,
    backup_activities: Option<HashMap<u32, Operation>>,
}

impl OperationalAgentBuilder {
    pub fn new(
        id: Id,
        agent_traits: HashSet<Resources>,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    ) -> Self {
        OperationalAgentBuilder {
            id,
            scheduling_environment,
            operational_algorithm: OperationalAlgorithm {
                objective_value: 0.0,
            },
            agent_traits,
            capacity: None,
            availability: None,
            assigned: None,
            backup_activities: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_agent_traits(mut self, agent_traits: HashSet<Resources>) -> Self {
        self.agent_traits = agent_traits;
        self
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
            agent_traits: self.agent_traits,
            capacity: self.capacity,
            availability: self.availability,
            assigned: self.assigned,
            backup_activities: self.backup_activities,
        }
    }
}

impl Handler<StatusMessage> for OperationalAgent {
    type Result = String;

    fn handle(&mut self, _msg: StatusMessage, _ctx: &mut Self::Context) -> Self::Result {
        format!(
            "ID: {}, traits: {}, objective_value: {}",
            self.id,
            self.agent_traits
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
