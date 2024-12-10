use std::collections::HashSet;

use actix::Handler;
use anyhow::{bail, Context, Result};
use shared_types::supervisor::{
    supervisor_response_scheduling::SupervisorResponseScheduling,
    supervisor_response_status::SupervisorResponseStatus, SupervisorRequestMessage,
    SupervisorResponseMessage,
};
use tracing::{event, Level};

use crate::agents::{
    supervisor_agent::algorithm::SupervisorParameters, AgentSpecific, SetAddr, StateLink,
};

use super::SupervisorAgent;

impl Handler<SetAddr> for SupervisorAgent {
    type Result = Result<()>;

    fn handle(&mut self, set_addr: SetAddr, _ctx: &mut Self::Context) -> Self::Result {
        if let SetAddr::Operational(id, addr) = set_addr {
            self.operational_agent_addrs.insert(id, addr);
            Ok(())
        } else {
            bail!("We have not created the logic for fixing this yet")
        }
    }
}

impl Handler<StateLink> for SupervisorAgent {
    type Result = Result<()>;

    fn handle(&mut self, state_link: StateLink, _ctx: &mut Self::Context) -> Self::Result {
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
                                    std::any::type_name::<SupervisorParameters>()
                                )
                            })?;
                        for activity_number in work_order.operations.keys() {
                            self.supervisor_algorithm
                                .supervisor_parameters
                                .create_and_insert_supervisor_parameter(
                                    &scheduling_environment_guard,
                                    &(work_order_number, *activity_number),
                                )
                        }
                    }
                    Ok(())
                }
            },
            StateLink::WorkerEnvironment => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let operational_agents = scheduling_environment_guard
                    .worker_environment
                    .system_agents
                    .operational
                    .iter()
                    .map(|in_op| &in_op.id)
                    .collect::<HashSet<&String>>();

                event!(Level::ERROR,
                    does_state_ids_and_addr_ids_match = self.operational_agent_addrs
                        .keys()
                        .map(|id| &id.0)
                        .collect::<HashSet<&String>>()
                        == operational_agents,
                        "Check this error later. FIX: YOU SHOULD call '.send()' instead of '.do_send()' and use the lldb debugger to trace the flow."
                );

                Ok(())
            }
            StateLink::TimeEnvironment => todo!(),
        }
    }
}

impl Handler<SupervisorRequestMessage> for SupervisorAgent {
    type Result = Result<SupervisorResponseMessage>;

    fn handle(
        &mut self,
        supervisor_request_message: SupervisorRequestMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        event!(Level::WARN, "start_of_supervisor_handler");

        match supervisor_request_message {
            SupervisorRequestMessage::Status(supervisor_status_message) => {
                event!(Level::WARN, "start of status message initialization");
                tracing::info!(
                    "Received SupervisorStatusMessage: {:?}",
                    supervisor_status_message
                );
                let supervisor_status = SupervisorResponseStatus::new(
                    self.supervisor_algorithm.resources.clone(),
                    self.supervisor_algorithm
                        .supervisor_solution
                        .count_unique_woa(),
                    self.supervisor_algorithm.objective_value,
                );
                event!(Level::WARN, "after creation of the supervisor_status");

                Ok(SupervisorResponseMessage::Status(supervisor_status))
            }

            SupervisorRequestMessage::Scheduling(_scheduling_message) => Ok(
                SupervisorResponseMessage::Scheduling(SupervisorResponseScheduling {}),
            ),
            SupervisorRequestMessage::Update => {
                bail!(
                    "IMPLEMENT update logic for Supervisor for Asset: {:?}",
                    self.asset
                );
            }
        }
    }
}
