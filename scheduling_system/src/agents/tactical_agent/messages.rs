use actix::prelude::*;

struct ScheduleIteration {}

impl Message for ScheduleIteration {
    type Result = ();
}
