use serde::{Deserialize, Serialize};

use crate::strategic::requests::strategic_request_scheduling_message::ScheduleChange;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalSchedulingRequest {
    Schedule(ScheduleChange),
    ScheduleMultiple(Vec<ScheduleChange>),
    ExcludeFromDay(ScheduleChange),
}
