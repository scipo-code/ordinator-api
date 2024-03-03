use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use chrono::{DateTime, Utc};
use shared_messages::resources::Resources;

use crate::models::{work_order::operation::Operation, SchedulingEnvironment};

#[allow(dead_code)]
pub struct OperationalAgent {
    id: String,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    agent_traits: Option<HashSet<Resources>>,
    capacity: Option<f32>,
    availability: Option<Vec<Availability>>,
    assigned: Option<Vec<AssignedWork>>,
    backup_activities: Option<HashMap<u32, Operation>>,
}

#[allow(dead_code)]
struct Availability {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

#[allow(dead_code)]
struct AssignedWork {
    work: u32,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl Actor for OperationalAgent {
    type Context = Context<Self>;
}

impl OperationalAgent {
    pub fn new(
        id: String,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    ) -> OperationalAgent {
        OperationalAgent {
            id,
            scheduling_environment,
            agent_traits: None,
            capacity: None,
            availability: None,
            assigned: None,
            backup_activities: None,
        }
    }
}

pub struct OperationalAgentBuilder {
    id: String,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    agent_traits: Option<HashSet<Resources>>,
    capacity: Option<f32>,
    availability: Option<Vec<Availability>>,
    assigned: Option<Vec<AssignedWork>>,
    backup_activities: Option<HashMap<u32, Operation>>,
}

impl OperationalAgentBuilder {
    pub fn new(id: String, scheduling_environment: Arc<Mutex<SchedulingEnvironment>>) -> Self {
        OperationalAgentBuilder {
            id,
            scheduling_environment,
            agent_traits: None,
            capacity: None,
            availability: None,
            assigned: None,
            backup_activities: None,
        }
    }

    pub fn with_agent_traits(mut self, agent_traits: HashSet<Resources>) -> Self {
        self.agent_traits = Some(agent_traits);
        self
    }

    pub fn with_capacity(mut self, capacity: f32) -> Self {
        self.capacity = Some(capacity);
        self
    }

    pub fn with_availability(mut self, availability: Vec<Availability>) -> Self {
        self.availability = Some(availability);
        self
    }

    pub fn with_assigned(mut self, assigned: Vec<AssignedWork>) -> Self {
        self.assigned = Some(assigned);
        self
    }

    pub fn with_backup_activities(mut self, backup_activities: HashMap<u32, Operation>) -> Self {
        self.backup_activities = Some(backup_activities);
        self
    }

    pub fn build(self) -> OperationalAgent {
        OperationalAgent {
            id: self.id,
            scheduling_environment: self.scheduling_environment,
            agent_traits: self.agent_traits,
            capacity: self.capacity,
            availability: self.availability,
            assigned: self.assigned,
            backup_activities: self.backup_activities,
        }
    }
}
