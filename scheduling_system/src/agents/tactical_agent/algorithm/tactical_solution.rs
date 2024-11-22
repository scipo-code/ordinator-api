use actix::Message;
use serde::Serialize;
use shared_types::scheduling_environment::{
    time_environment::day::Day,
    work_order::{
        operation::{operation_info::NumberOfPeople, ActivityNumber, Work},
        WorkOrderActivity, WorkOrderNumber,
    },
    worker_environment::resources::Resources,
};

#[derive(Hash, PartialEq, PartialOrd, Ord, Eq, Clone, Debug, Serialize)]
pub struct TacticalOperation {
    pub scheduled: Vec<(Day, Work)>,
    pub resource: Resources,
    pub number: NumberOfPeople,
    pub work_remaining: Work,
    pub work_order_activity: WorkOrderActivity,
}

impl TacticalOperation {
    pub fn new(
        scheduled: Vec<(Day, Work)>,
        resource: Resources,
        number: NumberOfPeople,
        work_remaining: Work,
        work_order_number: WorkOrderNumber,
        activity_number: ActivityNumber,
    ) -> TacticalOperation {
        TacticalOperation {
            scheduled,
            resource,
            number,
            work_remaining,
            work_order_activity: (work_order_number, activity_number),
        }
    }
}

impl Message for TacticalOperation {
    type Result = bool;
}
