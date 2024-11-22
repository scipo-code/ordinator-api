use actix::Handler;
use anyhow::{bail, Result};
use shared_types::{
    scheduling_environment::{work_order::WorkOrderActivity, worker_environment::resources::Id},
    supervisor::{
        supervisor_response_scheduling::SupervisorResponseScheduling,
        supervisor_response_status::SupervisorResponseStatus, SupervisorRequestMessage,
        SupervisorResponseMessage,
    },
};
use tracing::{event, Level};

use crate::agents::{
    operational_agent::algorithm::OperationalObjectiveValue, SetAddr, StateLink, StateLinkWrapper,
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

type StrategicMessage = ();
type TacticalMessage = ();
type SupervisorMessage = ();
// Why do we send this message? I am not really sure?
type OperationalMessage = ((Id, WorkOrderActivity), OperationalObjectiveValue);

impl
    Handler<
        StateLinkWrapper<StrategicMessage, TacticalMessage, SupervisorMessage, OperationalMessage>,
    > for SupervisorAgent
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

        match state_link {
            StateLink::Strategic(_) => Ok(()),
            StateLink::Tactical(_) => Ok(()),
            StateLink::Supervisor(_) => Ok(()),
            StateLink::Operational(_operational_solution) => Ok(()),
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
