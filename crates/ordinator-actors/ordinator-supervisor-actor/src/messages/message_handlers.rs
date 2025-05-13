use std::collections::HashSet;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use ordinator_orchestrator_actor_traits::ActorSpecific;
use ordinator_orchestrator_actor_traits::MessageHandler;
use ordinator_orchestrator_actor_traits::StateLink;
use ordinator_orchestrator_actor_traits::SystemSolutionTrait;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use tracing::Level;
use tracing::event;

use super::SupervisorRequestMessage;
use super::SupervisorResponseMessage;
use crate::SupervisorActor;
use crate::algorithm::supervisor_parameters::SupervisorParameters;
use crate::algorithm::supervisor_solution::SupervisorSolution;
use crate::messages::responses::SupervisorResponseScheduling;
use crate::messages::responses::SupervisorResponseStatus;

impl<Ss> MessageHandler for SupervisorActor<Ss>
where
    Ss: SystemSolutionTrait<Supervisor = SupervisorSolution>,
{
    type Req = SupervisorRequestMessage;
    type Res = SupervisorResponseMessage;

    fn handle_state_link(&mut self, state_link: StateLink) -> Result<Self::Res>
    {
        match state_link {
            StateLink::WorkOrders(agent_specific) => match agent_specific {
                ActorSpecific::Strategic(changed_work_orders) => {
                    // It is beginning to seem a little horrible that the self. here holds both the
                    // `scheduling_environment` and the `algorithm`. There is a
                    // couple of issues here relating to how we interact
                    // with the algorithm. I
                    let work_orders = {
                        let scheduling_environment_guard =
                            self.scheduling_environment.lock().unwrap();

                        scheduling_environment_guard.work_orders.inner.clone()
                    };

                    for work_order_number in changed_work_orders {
                        let work_order =
                            work_orders.get(&work_order_number).with_context(|| {
                                format!(
                                    "{:?} should always be present in {}",
                                    work_order_number,
                                    std::any::type_name::<SupervisorParameters>()
                                )
                            })?;
                        // TODO [ ]
                        // You need to take a clear stance on this in the code. Should you make an
                        // API for this? Of course you should.
                        //
                        // This is written so sloppy.
                        // I can sense that we should instead think about the data flow in
                        // the program. That probably has a higher chance of success. Yes.
                        for (activity_number, operation) in &work_order.operations.0 {
                            self.algorithm
                                .parameters
                                .create_and_insert_supervisor_parameter(
                                    operation,
                                    &(work_order_number, *activity_number),
                                )
                        }
                    }
                    Ok(SupervisorResponseMessage::StateLink)
                }
            },
            StateLink::WorkerEnvironment => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let operational_agents = scheduling_environment_guard
                    .worker_environment
                    .actor_specification
                    .get(self.actor_id.asset())
                    .unwrap()
                    .operational
                    .iter()
                    .map(|e| &e.id)
                    .collect::<HashSet<&Id>>();

                event!(
                    Level::ERROR,
                    does_state_ids_and_addr_ids_match = self
                        .algorithm
                        .loaded_shared_solution
                        .all_operational()
                        .iter()
                        .eq(operational_agents.iter().copied()),
                    "Check this error later. FIX: YOU SHOULD call '.send()' instead of '.do_send()' and use the lldb debugger to trace the flow."
                );

                Ok(SupervisorResponseMessage::StateLink)
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
                    self.actor_id.asset()
                );
            }
            SupervisorRequestMessage::Status(supervisor_status_message) => {
                event!(Level::WARN, "start of status message initialization");
                tracing::info!(
                    "Received SupervisorStatusMessage: {:?}",
                    supervisor_status_message
                );
                let supervisor_status = SupervisorResponseStatus {
                    supervisor_resource: self.algorithm.parameters.operational_ids.clone(),
                    delegated_work_order_activities: self.algorithm.solution.count_unique_woa(),
                    objective: self.algorithm.solution.objective_value,
                };
                event!(Level::WARN, "after creation of the supervisor_status");

                Ok(SupervisorResponseMessage::Status(supervisor_status))
            }
        }
    }
}
