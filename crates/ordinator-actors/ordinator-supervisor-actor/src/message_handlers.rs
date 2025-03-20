use std::collections::HashSet;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use shared_types::agents::supervisor::SupervisorRequestMessage;
use shared_types::agents::supervisor::SupervisorResponseMessage;
use shared_types::agents::supervisor::responses::supervisor_response_scheduling::SupervisorResponseScheduling;
use shared_types::agents::supervisor::responses::supervisor_response_status::SupervisorResponseStatus;
use shared_types::scheduling_environment::worker_environment::resources::Id;
use tracing::Level;
use tracing::event;

use crate::agents::Actor;
use crate::agents::ActorSpecific;
use crate::agents::MessageHandler;
use crate::agents::StateLink;
use crate::agents::SupervisorSolution;
use crate::agents::supervisor_agent::algorithm::supervisor_parameters::SupervisorParameters;

impl MessageHandler
    for Actor<
        SupervisorRequestMessage,
        SupervisorResponseMessage,
        SupervisorSolution,
        SupervisorParameters,
        (),
    >
{
    type Req = SupervisorRequestMessage;
    type Res = SupervisorResponseMessage;

    fn handle_state_link(&mut self, state_link: StateLink) -> Result<()>
    {
        match state_link {
            StateLink::WorkOrders(agent_specific) => match agent_specific {
                ActorSpecific::Strategic(changed_work_orders) => {
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
                            let operation = scheduling_environment_guard
                                .operation(&(work_order_number, *activity_number));
                            self.algorithm
                                .parameters
                                .create_and_insert_supervisor_parameter(
                                    operation,
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
                    .agent_environment
                    .operational
                    .keys()
                    .collect::<HashSet<&Id>>();

                event!(
                    Level::ERROR,
                    does_state_ids_and_addr_ids_match = self
                        .algorithm
                        .loaded_shared_solution
                        .operational
                        .keys()
                        .collect::<HashSet<&Id>>()
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
    ) -> Result<SupervisorResponseMessage>
    {
        event!(Level::WARN, "start_of_supervisor_handler");

        match supervisor_request_message {
            SupervisorRequestMessage::Scheduling(_scheduling_message) => Ok(
                SupervisorResponseMessage::Scheduling(SupervisorResponseScheduling {}),
            ),
            SupervisorRequestMessage::Update => {
                bail!(
                    "IMPLEMENT update logic for Supervisor for Asset: {:?}",
                    self.agent_id.asset()
                );
            }
            SupervisorRequestMessage::Status(supervisor_status_message) => {
                event!(Level::WARN, "start of status message initialization");
                tracing::info!(
                    "Received SupervisorStatusMessage: {:?}",
                    supervisor_status_message
                );
                let supervisor_status = SupervisorResponseStatus::new(
                    self.algorithm.parameters.operational_ids.clone(),
                    self.algorithm.solution.count_unique_woa(),
                    self.algorithm.solution.objective_value,
                );
                event!(Level::WARN, "after creation of the supervisor_status");

                Ok(SupervisorResponseMessage::Status(supervisor_status))
            }
        }
    }
}
