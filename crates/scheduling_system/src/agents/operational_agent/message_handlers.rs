use anyhow::{bail, Result};
use shared_types::agents::operational::{
    requests::operational_request_scheduling::OperationalSchedulingRequest,
    responses::operational_response_scheduling::{
        ApiAssignment, ApiAssignmentEvents, EventInfo, OperationalSchedulingResponse,
    },
    responses::operational_response_status::OperationalResponseStatus,
    OperationalRequestMessage, OperationalResponseMessage,
};
use tracing::{event, Level};

use crate::agents::{
    Agent, ActorSpecific, Algorithm, MessageHandler, OperationalSolution, StateLink,
};

use super::algorithm::{operational_parameter::OperationalParameters, OperationalNonProductive};

type OperationalAlgorithm =
    Algorithm<OperationalSolution, OperationalParameters, OperationalNonProductive>;
impl MessageHandler
    for Agent<OperationalAlgorithm, OperationalRequestMessage, OperationalResponseMessage>
{
    type Req = OperationalRequestMessage;
    type Res = OperationalResponseMessage;

    fn handle_state_link(&mut self, state_link: StateLink) -> Result<()> {
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
            StateLink::WorkerEnvironment => todo!(),
            StateLink::TimeEnvironment => todo!(),
        }
    }

    fn handle_request_message(
        &mut self,
        request: OperationalRequestMessage,
    ) -> Result<OperationalResponseMessage> {
        match request {
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
                    .supervisor
                    .count_delegate_types(&self.agent_id);

                let operational_response_status = OperationalResponseStatus::new(
                    self.agent_id.clone(),
                    assign,
                    assess,
                    unassign,
                    self.algorithm.solution.objective_value,
                );
                Ok(OperationalResponseMessage::Status(
                    operational_response_status,
                ))
            }
            OperationalRequestMessage::Scheduling(operational_scheduling_request) => {
                match operational_scheduling_request {
                    OperationalSchedulingRequest::OperationalIds => todo!(),
                    OperationalSchedulingRequest::OperationalState(_) => {
                        let mut json_assignments_events: Vec<ApiAssignmentEvents> = vec![];

                        for (work_order_activity, operational_solution) in
                            &self.algorithm.solution.scheduled_work_order_activities
                        {
                            let mut json_assignments = vec![];
                            for assignment in &operational_solution.assignments {
                                let json_assignment = ApiAssignment::new(
                                    assignment.event_type.clone().into(),
                                    assignment.start,
                                    assignment.finish,
                                );
                                json_assignments.push(json_assignment);
                            }

                            let event_info = EventInfo::new(Some(*work_order_activity));
                            let json_assignment_event =
                                ApiAssignmentEvents::new(event_info, json_assignments);
                            json_assignments_events.push(json_assignment_event);
                        }

                        let operational_scheduling_response =
                            OperationalSchedulingResponse::EventList(json_assignments_events);
                        Ok(OperationalResponseMessage::Scheduling(
                            operational_scheduling_response,
                        ))
                    }
                }
            }

            OperationalRequestMessage::Resource(_) => todo!(),
            OperationalRequestMessage::Time(_) => todo!(),
        }
    }
}
