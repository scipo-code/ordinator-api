use std::{collections::HashMap};


use shared_messages::{
    agent_error::AgentError,
    models::{
        work_order::{operation::ActivityNumber, WorkOrderNumber},
        worker_environment::resources::Id,
    },
    supervisor::{
        supervisor_response_resources::SupervisorResponseResources,
        supervisor_response_scheduling::SupervisorResponseScheduling,
        supervisor_response_time::SupervisorResponseTime,
    },
};

use crate::agents::{
    traits::LargeNeighborHoodSearch,
};

use super::SupervisorAgent;

pub struct SupervisorSchedulingRequest;
pub struct SupervisorResourceRequest;
pub struct SupervisorTimeRequest;

pub struct SupervisorAlgorithm {
    pub objective_value: f64,
    pub assigned_activities_by_agent: HashMap<Id, Vec<(WorkOrderNumber, ActivityNumber)>>,
}

impl SupervisorAlgorithm {
    pub fn new() -> Self {
        Self {
            objective_value: f64::INFINITY,
            assigned_activities_by_agent: HashMap::new(),
        }
    }

    pub(crate) fn clone(&self) -> SupervisorAlgorithm {
        todo!()
    }
}

impl LargeNeighborHoodSearch for SupervisorAgent {
    type SchedulingRequest = SupervisorSchedulingRequest;
    type SchedulingResponse = SupervisorResponseScheduling;
    type ResourceRequest = SupervisorResourceRequest;
    type ResourceResponse = SupervisorResponseResources;
    type TimeRequest = SupervisorTimeRequest;
    type TimeResponse = SupervisorResponseTime;

    type Error = AgentError;

    fn calculate_objective_value(&mut self) {
        todo!()
    }

    fn schedule(&mut self) {
        todo!();
    }

    fn unschedule(&mut self, _message: shared_messages::models::work_order::WorkOrderNumber) {
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
