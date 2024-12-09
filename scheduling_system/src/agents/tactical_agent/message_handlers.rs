use actix::prelude::*;
use anyhow::{bail, Context, Result};
use shared_types::{
    tactical::{TacticalRequestMessage, TacticalResponseMessage},
    StatusMessage,
};

use crate::agents::{
    tactical_agent::algorithm::tactical_parameters::TacticalParameters,
    traits::LargeNeighborHoodSearch, AgentSpecific, SetAddr, StateLink,
};

use super::TacticalAgent;

impl Handler<StatusMessage> for TacticalAgent {
    type Result = String;

    fn handle(&mut self, _msg: StatusMessage, _ctx: &mut Self::Context) -> Self::Result {
        format!(
            "Id: {}, Time horizon: {:?}, Objective: {:?}",
            self.id_tactical,
            self.tactical_algorithm.tactical_days.clone(),
            self.tactical_algorithm.objective_value()
        )
    }
}
impl Handler<TacticalRequestMessage> for TacticalAgent {
    type Result = Result<TacticalResponseMessage>;

    fn handle(
        &mut self,
        tactical_request: TacticalRequestMessage,
        _ctx: &mut actix::Context<Self>,
    ) -> Self::Result {
        match tactical_request {
            TacticalRequestMessage::Status(_tactical_status_message) => {
                let status_message = self.status().unwrap();
                Ok(TacticalResponseMessage::Status(status_message))
            }
            TacticalRequestMessage::Scheduling(_tactical_scheduling_message) => {
                todo!()
            }
            TacticalRequestMessage::Resources(tactical_resources_message) => {
                let resource_response = self
                    .tactical_algorithm
                    .update_resources_state(tactical_resources_message)
                    .unwrap();
                Ok(TacticalResponseMessage::Resources(resource_response))
            }
            TacticalRequestMessage::Days(_tactical_time_message) => {
                todo!()
            }
            TacticalRequestMessage::Update => {
                let locked_scheduling_environment = &self.scheduling_environment.lock().unwrap();
                let asset = &self.asset;

                self.tactical_algorithm
                    .create_tactical_parameters(locked_scheduling_environment, asset);
                Ok(TacticalResponseMessage::Update)
            }
        }
    }
}

impl Handler<StateLink> for TacticalAgent {
    type Result = Result<()>;

    fn handle(&mut self, state_link: StateLink, _ctx: &mut actix::Context<Self>) -> Self::Result {
        match state_link {
            StateLink::WorkOrders(agent_specific) => match agent_specific {
                AgentSpecific::Strategic(changed_work_orders) => {
                    let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                    let work_orders = &scheduling_environment_guard.work_orders.inner;

                    for work_order_number in changed_work_orders {
                        let work_order =
                            work_orders.get(&work_order_number).with_context(|| {
                                format!(
                                    "{:?} should always be present in {}",
                                    work_order_number,
                                    std::any::type_name::<TacticalParameters>()
                                )
                            })?;

                        self.tactical_algorithm
                            .create_and_insert_tactical_parameter(work_order)
                    }
                    Ok(())
                }
            },
            StateLink::WorkerEnvironment => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();
                let tactical_resources = scheduling_environment_guard
                    .worker_environment
                    .generate_tactical_resources(&self.tactical_algorithm.tactical_days);

                self.tactical_algorithm
                    .tactical_parameters
                    .tactical_capacity
                    .update_resources(tactical_resources);

                Ok(())
            }

            StateLink::TimeEnvironment => {
                todo!()
            }
        }
    }
}

impl Handler<SetAddr> for TacticalAgent {
    type Result = Result<()>;

    fn handle(&mut self, msg: SetAddr, _ctx: &mut actix::Context<Self>) -> Self::Result {
        match msg {
            SetAddr::Supervisor(id, addr) => {
                self.main_supervisor_addr = Some((id, addr));
                Ok(())
            }
            _ => {
                bail!("The tactical agent received an Addr<T>, where T is not a valid Actor")
            }
        }
    }
}
