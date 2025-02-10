use std::collections::HashSet;

use anyhow::{bail, Context, Result};
use shared_types::supervisor::{
    supervisor_response_scheduling::SupervisorResponseScheduling,
    supervisor_response_status::SupervisorResponseStatus, SupervisorRequestMessage,
    SupervisorResponseMessage,
};
use tracing::{event, Level};

use crate::agents::{
    supervisor_agent::algorithm::SupervisorParameters, Agent, AgentSpecific, MessageHandler,
    StateLink,
};

use super::algorithm::SupervisorAlgorithm;

impl MessageHandler
    for Agent<SupervisorAlgorithm, SupervisorRequestMessage, SupervisorResponseMessage>
{
    type Req = SupervisorRequestMessage;
    type Res = SupervisorResponseMessage;

    fn handle_state_link(&mut self, state_link: StateLink) -> Result<()> {
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
                            self.algorithm
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
                    does_state_ids_and_addr_ids_match = self
                        .algorithm
                        .loaded_shared_solution
                        .operational
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

    fn handle_request_message(
        &mut self,
        supervisor_request_message: SupervisorRequestMessage,
    ) -> Result<SupervisorResponseMessage> {
        event!(Level::WARN, "start_of_supervisor_handler");

        match supervisor_request_message {
            SupervisorRequestMessage::Scheduling(_scheduling_message) => Ok(
                SupervisorResponseMessage::Scheduling(SupervisorResponseScheduling {}),
            ),
            SupervisorRequestMessage::Update => {
                bail!(
                    "IMPLEMENT update logic for Supervisor for Asset: {:?}",
                    self.asset
                );
            }
            SupervisorRequestMessage::Status(supervisor_status_message) => {
                event!(Level::WARN, "start of status message initialization");
                tracing::info!(
                    "Received SupervisorStatusMessage: {:?}",
                    supervisor_status_message
                );
                let supervisor_status = SupervisorResponseStatus::new(
                    self.algorithm.resources.clone(),
                    self.algorithm.supervisor_solution.count_unique_woa(),
                    self.algorithm.objective_value,
                );
                event!(Level::WARN, "after creation of the supervisor_status");

                Ok(SupervisorResponseMessage::Status(supervisor_status))
            }
        }
    }
}
