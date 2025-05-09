use anyhow::Result;
use anyhow::bail;
use ordinator_orchestrator_actor_traits::ActorSpecific;
use ordinator_orchestrator_actor_traits::MessageHandler;
use ordinator_orchestrator_actor_traits::StateLink;
use ordinator_orchestrator_actor_traits::SupervisorInterface;
use ordinator_orchestrator_actor_traits::SystemSolutionTrait;
use tracing::Level;
use tracing::event;

use super::OperationalRequestMessage;
use super::OperationalResponseMessage;
use super::OperationalResponseStatus;
use super::OperationalSchedulingRequest;
use crate::OperationalActor;
use crate::algorithm::operational_solution::OperationalSolution;

// Was this actually needed? I am not really sure here I believe that
// the best approach is to make something.
impl<Ss> MessageHandler for OperationalActor<Ss>
where
    Ss: SystemSolutionTrait<Operational = OperationalSolution>,
{
    type Req = OperationalRequestMessage;
    type Res = OperationalResponseMessage;

    fn handle_state_link(&mut self, state_link: StateLink) -> Result<OperationalResponseMessage>
    {
        event!(
            Level::INFO,
            self.algorithm.operational_parameters =
                self.algorithm.parameters.work_order_parameters.len()
        );
        match state_link {
            StateLink::WorkOrders(ActorSpecific::Strategic(changed_work_orders)) => {
                // TODO:
                event!(Level::ERROR, unhandled_work_orders = ?changed_work_orders);
                bail!("IMPLEMENT STATELINK FOR THE OPERATIONAL AGENT");
            }
            // Here you should make a clear separation between the different
            // ways
            StateLink::WorkerEnvironment => todo!(),

            StateLink::TimeEnvironment => todo!(),
        }
    }

    fn handle_request_message(
        &mut self,
        request: OperationalRequestMessage,
    ) -> Result<OperationalResponseMessage>
    {
        match request {
            // You should put the `contract` message types in here. I do not see
            // any different way of doing it.
            OperationalRequestMessage::Status(_) => {
                // WARN DEBUG: This should be included if you get an error
                //     format!(
                //         "ID: {}, traits: {}, Objective: {:?}",
                //         self.agent_id.0,
                //         self.agent_id
                //             .1
                //             .iter()
                //             .map(|resource| resource.to_string())
                //             .collect::<Vec<String>>()
                //             .join(", "),
                //         self.algorithm
                //             .operational_solution
                //             .objective_value
                //     )
                // }
                let (assign, assess, unassign): (u64, u64, u64) = self
                    .algorithm
                    .loaded_shared_solution
                    .supervisor()
                    .count_delegate_types(&self.actor_id);

                // Remember that the business types should not be the same type as the
                // algorithm types. That is crucial to understand in all this.
                // These should not have the `OperationalResponseStatus`
                // QUESTION
                // Should the `OperationalObjectiveValue` be shareable? No I do not think so.
                let operational_response_status = OperationalResponseStatus::new(
                    self.actor_id.clone(),
                    assign,
                    assess,
                    unassign,
                    self.algorithm.solution.objective_value,
                );
                Ok(OperationalResponseMessage::Status(
                    operational_response_status,
                ))
            }
            // The messages should change the parameters the SchedulingEnvironment and
            // this should in turn update the `Parameters` this is the flow that we need
            // in the program for this to work. The issue here is that we cannot actually
            // update anything in the `SchedulingEnvironment` as there are no data for
            // the individual workers.
            OperationalRequestMessage::Scheduling(operational_scheduling_request) => {
                match operational_scheduling_request {
                    OperationalSchedulingRequest::OperationalIds => todo!(),
                    OperationalSchedulingRequest::OperationalState(_) => {
                        // let mut json_assignments_events: Vec<ApiAssignmentEvents> = vec![];

                        // I think that you should starte removing code that does not really
                        // work here. You have to make something operational fast.
                        // for (work_order_activity, operational_solution) in
                        //     &self.algorithm.solution.scheduled_work_order_activities
                        // {
                        //     let mut json_assignments = vec![];
                        //     for assignment in &operational_solution.assignments {
                        //         // ApiAssignment is an API type not a business type, so where
                        //         // should it go in the code?
                        //         let json_assignment = ApiAssignment::new(
                        //             assignment.operational_events,
                        //             assignment.start,
                        //             assignment.finish,
                        //         );
                        //         json_assignments.push(json_assignment);
                        //     }

                        //     let event_info = EventInfo::new(Some(*work_order_activity));
                        //     let json_assignment_event =
                        //         ApiAssignmentEvents::new(event_info, json_assignments);
                        //     json_assignments_events.push(json_assignment_event);
                        // }

                        todo!()
                        // let operational_scheduling_response =
                        //     OperationalSchedulingResponse::EventList(json_assignments_events);
                        // Ok(OperationalResponseMessage::Scheduling(
                        //     operational_scheduling_response,
                        // ))
                    }
                }
            }

            OperationalRequestMessage::Resource(_) => todo!(),
            OperationalRequestMessage::Time(_) => todo!(),
        }
    }
}
