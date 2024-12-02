use crate::strategic::strategic_request_scheduling_message::ScheduleChange;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalSchedulingRequest {
    Schedule(ScheduleChange),
    ScheduleMultiple(Vec<ScheduleChange>),
    ExcludeFromDay(ScheduleChange),
}
