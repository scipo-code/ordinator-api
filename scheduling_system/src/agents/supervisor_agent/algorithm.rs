use std::ops::DerefMut;

use shared_messages::{
    agent_error::AgentError,
    supervisor::{
        supervisor_response_resources::SupervisorResponseResources,
        supervisor_response_scheduling::SupervisorResponseScheduling,
        supervisor_response_time::SupervisorResponseTime,
    },
};

use crate::agents::traits::LargeNeighborHoodSearch;

use super::SupervisorAgent;

pub struct SupervisorSchedulingMessage;
pub struct SupervisorResourceMessage;
pub struct SupervisorTimeMessage;

impl LargeNeighborHoodSearch for SupervisorAgent {
    type SchedulingMessage = SupervisorSchedulingMessage;
    type SchedulingResponse = SupervisorResponseScheduling;
    type ResourceMessage = SupervisorResourceMessage;
    type ResourceResponse = SupervisorResponseResources;
    type TimeMessage = SupervisorTimeMessage;
    type TimeResponse = SupervisorResponseTime;

    type Error = AgentError;

    fn calculate_objective_value(&mut self) {
        todo!()
    }

    fn schedule(&mut self) {
        for (work_order_number, operations) in &self.assigned_work_orders {
            for (activity_number, operation) in operations {
                if operation.resource == *self.id_supervisor.1.first().unwrap() {
                    // self.operational_agent_addrs;
                }
            }
        }
    }

    fn unschedule(&mut self, message: shared_messages::models::work_order::WorkOrderNumber) {
        todo!()
    }

    fn update_scheduling_state(
        &mut self,
        message: Self::SchedulingMessage,
    ) -> Result<Self::SchedulingResponse, Self::Error> {
        todo!()
    }

    fn update_time_state(
        &mut self,
        message: Self::TimeMessage,
    ) -> Result<Self::TimeResponse, Self::Error> {
        todo!()
    }

    fn update_resources_state(
        &mut self,
        message: Self::ResourceMessage,
    ) -> Result<Self::ResourceResponse, Self::Error> {
        todo!()
    }
}
