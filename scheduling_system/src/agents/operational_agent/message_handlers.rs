use actix::Handler;
use anyhow::{bail, Result};
use shared_types::operational::{
    operational_request_scheduling::OperationalSchedulingRequest,
    operational_response_scheduling::{
        ApiAssignment, ApiAssignmentEvents, EventInfo, OperationalSchedulingResponse,
    },
    operational_response_status::OperationalStatusResponse,
    OperationalRequestMessage, OperationalResponseMessage,
};
use tracing::{event, Level};

use crate::agents::{AgentSpecific, StateLink};

impl Handler<StateLink> for OperationalAgent {
    type Result = Result<()>;

    fn handle(&mut self, state_link: StateLink, _ctx: &mut Self::Context) -> Self::Result {
        event!(
            Level::INFO,
            self.operational_algorithm.operational_parameters = self
                .operational_algorithm
                .operational_parameters
                .work_order_parameters
                .len()
        );
        match state_link {
            StateLink::WorkOrders(AgentSpecific::Strategic(changed_work_orders)) => {
                // TODO:
                event!(Level::ERROR, unhandled_work_orders = ?changed_work_orders);
                bail!("IMPLEMENT STATELINK FOR THE OPERATIONAL AGENT");
            }
            StateLink::WorkerEnvironment => todo!(),
            StateLink::TimeEnvironment => todo!(),
        }
    }
}

impl Handler<OperationalRequestMessage> for OperationalAgent {
    type Result = Result<OperationalResponseMessage>;

    fn handle(
        &mut self,
        request: OperationalRequestMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        match request {
            OperationalRequestMessage::Status(_) => {
                // WARN DEBUG: This should be included if you get an error
                //     format!(
                //         "ID: {}, traits: {}, Objective: {:?}",
                //         self.operational_id.0,
                //         self.operational_id
                //             .1
                //             .iter()
                //             .map(|resource| resource.to_string())
                //             .collect::<Vec<String>>()
                //             .join(", "),
                //         self.operational_algorithm
                //             .operational_solution
                //             .objective_value
                //     )
                // }
                let (assign, assess, unassign): (u64, u64, u64) = self
                    .operational_algorithm
                    .loaded_shared_solution
                    .supervisor
                    .count_delegate_types(&self.operational_id);

                let operational_response_status = OperationalStatusResponse::new(
                    self.operational_id.clone(),
                    assign,
                    assess,
                    unassign,
                    self.operational_algorithm
                        .operational_solution
                        .objective_value,
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

                        for (work_order_activity, operational_solution) in &self
                            .operational_algorithm
                            .operational_solution
                            .work_order_activities_assignment
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
