use actix::prelude::*;
use anyhow::{bail, Result};
use shared_types::{
    tactical::{TacticalRequestMessage, TacticalResponseMessage},
    StatusMessage,
};

use crate::agents::{traits::LargeNeighborHoodSearch, SetAddr, StateLink};

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
            StateLink::Strategic(_strategic_state) => Ok(()),
            StateLink::Tactical => {
                todo!()
            }
            StateLink::Supervisor => {
                todo!()
            }
            StateLink::Operational => {
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
