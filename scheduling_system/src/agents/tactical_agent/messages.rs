use actix::prelude::*;
use shared_messages::StatusMessage;

use super::TacticalAgent;

impl Handler<StatusMessage> for TacticalAgent {
    type Result = String;

    fn handle(&mut self, _msg: StatusMessage, _ctx: &mut Self::Context) -> Self::Result {
        format!(
            "Id: {}, Time horizon: {:?}, Objective: {}",
            self.id_tactical,
            self.time_horizon(),
            self.tactical_algorithm.get_objective_value()
        )
    }
}
