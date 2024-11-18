use actix::prelude::*;
use shared_types::StatusMessage;

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
