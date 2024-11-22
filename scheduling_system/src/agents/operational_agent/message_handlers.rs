use actix::Handler;
use anyhow::Result;
use shared_types::{
    operational::{
        operational_request_scheduling::OperationalSchedulingRequest,
        operational_response_scheduling::{
            ApiAssignment, ApiAssignmentEvents, EventInfo, OperationalSchedulingResponse,
        },
        operational_response_status::OperationalStatusResponse,
        OperationalRequestMessage, OperationalResponseMessage,
    },
    StatusMessage,
};
use tracing::{event, Level};

use crate::agents::{StateLink, StateLinkWrapper, UpdateWorkOrderMessage};

use super::OperationalAgent;

type StrategicMessage = ();
type TacticalMessage = ();
type SupervisorMessage = ();
type OperationalMessage = ();

impl
    Handler<
        StateLinkWrapper<StrategicMessage, TacticalMessage, SupervisorMessage, OperationalMessage>,
    > for OperationalAgent
{
    type Result = Result<()>;

    fn handle(
        &mut self,
        state_link_wrapper: StateLinkWrapper<
            StrategicMessage,
            TacticalMessage,
            SupervisorMessage,
            OperationalMessage,
        >,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let state_link = state_link_wrapper.state_link;
        let span = state_link_wrapper.span;
        let _enter = span.enter();

        event!(
            Level::INFO,
            self.operational_algorithm.operational_parameters = self
                .operational_algorithm
                .operational_parameters
                .work_order_parameters
                .len()
        );
        match state_link {
            StateLink::Strategic(_) => todo!(),
            StateLink::Tactical(_) => todo!(),
            StateLink::Supervisor(_initial_message) => todo!(),
            StateLink::Operational(_) => todo!(),
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
                            .work_order_activities
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

impl Handler<StatusMessage> for OperationalAgent {
    type Result = String;

    fn handle(&mut self, _msg: StatusMessage, _ctx: &mut Self::Context) -> Self::Result {
        format!(
            "ID: {}, traits: {}, Objective: {:?}",
            self.operational_id.0,
            self.operational_id
                .1
                .iter()
                .map(|resource| resource.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.operational_algorithm
                .operational_solution
                .objective_value
        )
    }
}

impl Handler<UpdateWorkOrderMessage> for OperationalAgent {
    type Result = ();

    fn handle(
        &mut self,
        _update_work_order: UpdateWorkOrderMessage,

        _ctx: &mut Self::Context,
    ) -> Self::Result {
        // todo!();
        event!(
            Level::WARN,
            "Update 'impl Handler<UpdateWorkOrderMessage> for SupervisorAgent'"
        );
    }
}
