use std::collections::HashSet;

use shared_types::{
    agent_error::AgentError,
    scheduling_environment::work_order::{operation::ActivityNumber, WorkOrderNumber},
    supervisor::{
        supervisor_response_resources::SupervisorResponseResources,
        supervisor_response_scheduling::SupervisorResponseScheduling,
        supervisor_response_time::SupervisorResponseTime,
    },
};

use crate::agents::traits::LargeNeighborHoodSearch;

use super::SupervisorAgent;

pub struct SupervisorSchedulingRequest;
pub struct SupervisorResourceRequest;
pub struct SupervisorTimeRequest;

impl LargeNeighborHoodSearch for SupervisorAgent {
    type SchedulingRequest = SupervisorSchedulingRequest;
    type SchedulingResponse = SupervisorResponseScheduling;
    type ResourceRequest = SupervisorResourceRequest;
    type ResourceResponse = SupervisorResponseResources;
    type TimeRequest = SupervisorTimeRequest;
    type TimeResponse = SupervisorResponseTime;

    type SchedulingUnit = (WorkOrderNumber, ActivityNumber);

    type Error = AgentError;

    fn calculate_objective_value(&mut self) {
        let assigned_woas = &self.supervisor_algorithm.assigned_to_operational_agents;

        let all_woas: HashSet<_> = self
            .supervisor_algorithm
            .assigned_work_orders
            .iter()
            .map(|(wo, activities, _os)| (wo, activities))
            .collect();

        // So the issue here is that there can be more assign work orders than all_woas. This probably comes from the
        // fact that the woas are updated correctly and the assign is not. This whole setup means that the code should
        // work so that the supervisor updates the state of each of his OperationalAgents in response to the message
        // that he receives from his. Okay we should fix this first and call an assert.
        assert!(assigned_woas
            .iter()
            .map(|(wo, ac)| all_woas.contains(&(wo, ac)))
            .all(|present_woa| present_woa));

        self.supervisor_algorithm.objective_value =
            assigned_woas.len() as f64 / all_woas.len() as f64;
    }

    fn schedule(&mut self) {
        todo!();
    }

    fn unschedule(&mut self, _message: Self::SchedulingUnit) {
        todo!()
    }

    fn update_scheduling_state(
        &mut self,
        _message: Self::SchedulingRequest,
    ) -> Result<Self::SchedulingResponse, Self::Error> {
        todo!()
    }

    fn update_time_state(
        &mut self,
        _message: Self::TimeRequest,
    ) -> Result<Self::TimeResponse, Self::Error> {
        todo!()
    }

    fn update_resources_state(
        &mut self,
        _message: Self::ResourceRequest,
    ) -> Result<Self::ResourceResponse, Self::Error> {
        todo!()
    }
}
