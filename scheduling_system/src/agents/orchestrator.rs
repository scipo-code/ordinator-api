use actix::prelude::*;
use shared_messages::models::worker_environment::resources;
use shared_messages::models::worker_environment::resources::Id;
use shared_messages::models::worker_environment::resources::MainResources;

use shared_messages::models::worker_environment::resources::Shift;
use shared_messages::Asset;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use strum::IntoEnumIterator;

use crate::agents::operational_agent::OperationalAgent;
use crate::agents::strategic_agent::StrategicAgent;
use crate::agents::supervisor_agent::SupervisorAgent;
use crate::agents::tactical_agent::TacticalAgent;
use crate::init::agent_factory;
use crate::init::agent_factory::AgentFactory;
use crate::init::logging::LogHandles;
use shared_messages::models::SchedulingEnvironment;

pub struct Orchestrator {
    pub scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub agent_factory: AgentFactory,
    pub agent_registries: HashMap<Asset, ActorRegistry>,
    pub log_handles: LogHandles,
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
        supervisor_agent_addrs: HashMap<Id, Addr<SupervisorAgent>>,
    ) -> Self {
        ActorRegistry {
            strategic_agent_addr,
            tactical_agent_addr,
            supervisor_agent_addrs,
            operational_agent_addrs: HashMap::new(),
        }
    }

    pub fn add_supervisor_agent(&mut self, id: Id, addr: Addr<SupervisorAgent>) {
        self.supervisor_agent_addrs.insert(id, addr);
    }

    pub fn add_operational_agent(&mut self, id: Id, addr: Addr<OperationalAgent>) {
        self.operational_agent_addrs.insert(id, addr);
    }

    pub fn supervisor_agent_addr(&self, id: Id) -> Addr<SupervisorAgent> {
        self.supervisor_agent_addrs.get(&id).unwrap().clone()
    }

    pub fn supervisor_agent_addr_by_resource(
        &self,
        resource: &shared_messages::models::worker_environment::resources::Resources,
    ) -> Addr<SupervisorAgent> {
        let matching_supervisor = self.supervisor_agent_addrs.iter().find_map(|(id, addr)| {
            if id.2.contains(resource) {
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
                    if id.3.as_ref().unwrap() == &MainResources::MtnMech {
                        Some(addr)
                    } else {
                        None
                    }
                })
                .unwrap()
                .clone(),
        }
    }

    pub fn operational_agent_addr(&self, id: Id) -> Addr<OperationalAgent> {
        self.operational_agent_addrs.get(&id).unwrap().clone()
    }

    pub fn supervisor_by_id_string(&self, id_string: String) -> Id {
        self.supervisor_agent_addrs
            .keys()
            .find(|id| id.0 == id_string)
            .unwrap()
            .clone()
    }

    #[allow(dead_code)]
    pub fn operational_by_id_string(&self, id_string: String) -> Id {
        self.operational_agent_addrs
            .keys()
            .find(|id| id.0 == id_string)
            .unwrap()
            .clone()
    }
}

impl Orchestrator {
    pub fn new(
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        log_handles: LogHandles,
    ) -> Self {
        let agent_factory = agent_factory::AgentFactory::new(scheduling_environment.clone());

        let agent_registries = HashMap::new();

        Orchestrator {
            scheduling_environment,
            agent_factory,
            agent_registries,
            log_handles,
        }
    }

    pub fn add_asset(&mut self, asset: Asset) {
        let strategic_agent_addr = self.agent_factory.build_strategic_agent(asset.clone());

        let tactical_agent_addr = self
            .agent_factory
            .build_tactical_agent(asset.clone(), strategic_agent_addr.clone());

        let mut supervisor_addrs = HashMap::<Id, Addr<SupervisorAgent>>::new();
        for main_resource in resources::MainResources::iter() {
            let id = Id::new(
                "default".to_string(),
                Shift::Day.generate_time_intervals(),
                vec![],
                Some(main_resource),
            );
            let supervisor_addr = self.agent_factory.build_supervisor_agent(
                asset.clone(),
                id.clone(),
                tactical_agent_addr.clone(),
            );

            supervisor_addrs.insert(id, supervisor_addr);
        }

        let agent_registry =
            ActorRegistry::new(strategic_agent_addr, tactical_agent_addr, supervisor_addrs);

        self.agent_registries.insert(asset, agent_registry);
    }
}
