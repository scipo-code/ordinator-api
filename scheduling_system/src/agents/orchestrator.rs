use actix::prelude::*;
use shared_messages::scheduling_environment::time_environment::day::Day;
use shared_messages::scheduling_environment::time_environment::period::Period;
use shared_messages::scheduling_environment::worker_environment::resources;
use shared_messages::scheduling_environment::worker_environment::resources::Id;
use shared_messages::scheduling_environment::worker_environment::resources::MainResources;

use shared_messages::scheduling_environment::worker_environment::resources::Resources;
use shared_messages::strategic::Periods;
use shared_messages::strategic::StrategicResources;
use shared_messages::tactical::Days;
use shared_messages::tactical::TacticalResources;
use shared_messages::Asset;
use shared_messages::TomlAgents;
use std::collections::HashMap;
use std::path::Path;
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
use shared_messages::scheduling_environment::SchedulingEnvironment;

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
        resource: &shared_messages::scheduling_environment::worker_environment::resources::Resources,
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
                    if id.2.as_ref().unwrap() == &MainResources::MtnMech {
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
        let toml_agents_string_path = std::env::var("RESOURCE_CONFIG_INITIALIZATION")
            .expect("Could not read RESOURCE_CONFIG_INITIALIZATION");

        let toml_agents_path = Path::new(&toml_agents_string_path);

        let strategic_resources = self.generate_strategic_resources(toml_agents_path);

        let tactical_resources = self.generate_tactical_resources(toml_agents_path);

        let strategic_agent_addr = self
            .agent_factory
            .build_strategic_agent(asset.clone(), Some(strategic_resources));

        let tactical_agent_addr = self.agent_factory.build_tactical_agent(
            asset.clone(),
            strategic_agent_addr.clone(),
            Some(tactical_resources),
        );

        let mut supervisor_addrs = HashMap::<Id, Addr<SupervisorAgent>>::new();
        for main_resource in resources::MainResources::iter() {
            let id = Id::new("default".to_string(), vec![], Some(main_resource));
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

    fn generate_strategic_resources(&self, toml_agents_path: &Path) -> StrategicResources {
        let periods: Vec<Period> = self
            .scheduling_environment
            .lock()
            .unwrap()
            .periods()
            .clone();

        let contents = std::fs::read_to_string(toml_agents_path).unwrap();

        let config: TomlAgents = toml::from_str(&contents).unwrap();

        let _hours_per_day = 6.0;
        let days_in_period = 13.0;

        let gradual_reduction = |i: usize| -> f64 {
            if i == 0 {
                1.0
            } else if i == 1 {
                0.9
            } else if i == 2 {
                0.8
            } else {
                0.6
            }
        };

        let mut resources_hash_map = HashMap::<Resources, Periods>::new();
        for operational_agent in config.operational {
            for (i, period) in periods.clone().iter().enumerate() {
                let resource_periods = resources_hash_map
                    .entry(
                        operational_agent
                            .resources
                            .resources
                            .first()
                            .cloned()
                            .unwrap(),
                    )
                    .or_insert(Periods(HashMap::new()));

                *resource_periods
                    .0
                    .entry(period.clone())
                    .or_insert_with(|| 0.0) +=
                    operational_agent.hours_per_day * days_in_period * gradual_reduction(i)
            }
        }

        StrategicResources::new(resources_hash_map)
    }
    fn generate_tactical_resources(&self, toml_path: &Path) -> TacticalResources {
        let days: Vec<Day> = self
            .scheduling_environment
            .lock()
            .unwrap()
            .tactical_days()
            .clone();

        let contents = std::fs::read_to_string(toml_path).unwrap();

        let config: TomlAgents = toml::from_str(&contents).unwrap();

        let _hours_per_day = 6.0;

        let gradual_reduction = |i: usize| -> f64 {
            match i {
                0..=13 => 1.0,
                14..=27 => 1.0,
                _ => 1.0,
            }
        };

        let mut resources_hash_map = HashMap::<Resources, Days>::new();
        for operational_agent in config.operational {
            for (i, day) in days.clone().iter().enumerate() {
                let resource_periods = resources_hash_map
                    .entry(
                        operational_agent
                            .resources
                            .resources
                            .first()
                            .cloned()
                            .unwrap(),
                    )
                    .or_insert(Days::new(HashMap::new()));

                *resource_periods
                    .days
                    .entry(day.clone())
                    .or_insert_with(|| 0.0) +=
                    operational_agent.hours_per_day * gradual_reduction(i);
            }
        }
        TacticalResources::new(resources_hash_map)
    }
}
