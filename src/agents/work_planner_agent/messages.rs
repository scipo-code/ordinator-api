use actix::prelude::*;

use crate::agents::scheduler_agent::SchedulerAgent;

struct ScheduleIteration {}

impl Message for ScheduleIteration {
    type Result = ();
}

impl Handler<ScheduleIteration> for SchedulerAgent {
    type Result = ();

    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut Self::Context) -> Self::Result {
        // let jl = JULIA.lock().unwrap();
        // jl.scope(|_global, scope| {
        //     // Julia code here
        //     Ok(())
        // })?;
        // Ok(42.0)
    }
}