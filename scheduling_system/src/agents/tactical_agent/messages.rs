use actix::prelude::*;
use shared_messages::StatusMessage;

use super::TacticalAgent;

struct ScheduleIteration {}

impl Message for ScheduleIteration {
    type Result = ();
}

impl Handler<StatusMessage> for TacticalAgent {
    type Result = String;

    fn handle(&mut self, _msg: StatusMessage, _ctx: &mut Self::Context) -> Self::Result {
        format!(
            "Id: {}, Time horizon: {}, Objective: {}",
            self.id,
            self.time_horizon(),
            self.tactical_algorithm.get_objective_value()
        )
    }
}
