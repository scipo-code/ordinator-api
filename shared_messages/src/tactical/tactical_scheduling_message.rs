use crate::strategic::strategic_request_scheduling_message::SingleWorkOrder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalSchedulingMessage {
    Schedule(SingleWorkOrder),
    ScheduleMultiple(Vec<SingleWorkOrder>),
    ExcludeFromDay(SingleWorkOrder),
}
