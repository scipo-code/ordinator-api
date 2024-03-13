use actix::prelude::*;
use shared_messages::resources;
use shared_messages::resources::Id;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::agents::operational_agent::OperationalAgent;
use crate::agents::strategic_agent::StrategicAgent;
use crate::agents::supervisor_agent::SupervisorAgent;
use crate::agents::tactical_agent::TacticalAgent;
use crate::init::agent_factory;
use crate::init::agent_factory::AgentFactory;
use crate::models::SchedulingEnvironment;

pub struct Orchestrator {
    pub scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub agent_factory: AgentFactory,
    pub agent_registry: ActorRegistry,
}

pub struct ActorRegistry {
    pub strategic_agent_addr: Addr<StrategicAgent>,
    pub tactical_agent_addr: Addr<TacticalAgent>,
    pub supervisor_agent_addrs: HashMap<Id, Addr<SupervisorAgent>>,
    pub operational_agent_addrs: HashMap<Id, Addr<OperationalAgent>>,
}

impl ActorRegistry {
    fn new(
        strategic_agent_addr: Addr<StrategicAgent>,
        tactical_agent_addr: Addr<TacticalAgent>,
    ) -> Self {
        ActorRegistry {
            strategic_agent_addr,
            tactical_agent_addr,
            supervisor_agent_addrs: HashMap::new(),
            operational_agent_addrs: HashMap::new(),
        }
    }

    pub fn add_supervisor_agent(&mut self, id: Id, addr: Addr<SupervisorAgent>) {
        self.supervisor_agent_addrs.insert(id, addr);
    }

    pub fn add_operational_agent(&mut self, id: Id, addr: Addr<OperationalAgent>) {
        self.operational_agent_addrs.insert(id, addr);
    }

    pub fn get_strategic_agent_addr(&self) -> Addr<StrategicAgent> {
        self.strategic_agent_addr.clone()
    }

    pub fn get_tactical_agent_addr(&self) -> Addr<TacticalAgent> {
        self.tactical_agent_addr.clone()
    }

    pub fn get_supervisor_agent_addr(&self, id: Id) -> Addr<SupervisorAgent> {
        self.supervisor_agent_addrs.get(&id).unwrap().clone()
    }

    pub fn get_supervisor_agent_addr_by_resource(
        &self,
        resource: &shared_messages::resources::Resources,
    ) -> Addr<SupervisorAgent> {
        let matching_supervisor = self.supervisor_agent_addrs.iter().find_map(|(id, addr)| {
            if id.1.contains(resource) {
                Some(addr)
            } else {
                None
            }
        });

        match matching_supervisor {
            Some(addr) => addr.clone(),
            None => self
                .supervisor_agent_addrs
                .iter()
                .find_map(|(id, addr)| {
                    if id.1.contains(&resources::Resources::MtnMech) {
                        Some(addr)
                    } else {
                        None
                    }
                })
                .unwrap()
                .clone(),
        }
    }

    pub fn get_operational_agent_addr(&self, id: Id) -> Addr<OperationalAgent> {
        self.operational_agent_addrs.get(&id).unwrap().clone()
    }

    pub fn get_supervisor_by_id_string(&self, id_string: String) -> Id {
        self.supervisor_agent_addrs
            .keys()
            .find(|id| id.0 == id_string)
            .unwrap()
            .clone()
    }
    pub fn get_operational_by_id_string(&self, id_string: String) -> Id {
        self.operational_agent_addrs
            .keys()
            .find(|id| id.0 == id_string)
            .unwrap()
            .clone()
    }
}

impl Orchestrator {
    pub fn new(scheduling_environment: Arc<Mutex<SchedulingEnvironment>>) -> Self {
        let agent_factory = agent_factory::AgentFactory::new(scheduling_environment.clone());

        let strategic_agent_addr = agent_factory.build_strategic_agent();
        let tactical_agent_addr =
            agent_factory.build_tactical_agent(56, strategic_agent_addr.clone());

        Orchestrator {
            scheduling_environment,
            agent_factory,
            agent_registry: ActorRegistry::new(strategic_agent_addr, tactical_agent_addr),
        }
    }
}

#[cfg(test)]
mod tests {}
