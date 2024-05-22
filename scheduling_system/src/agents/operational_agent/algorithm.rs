use std::collections::HashMap;

use chrono::{DateTime, Utc};
use shared_messages::{
    agent_error::AgentError,
    models::{
        work_order::{operation::ActivityNumber, WorkOrderNumber},
        worker_environment::availability::Availability,
    },
    operational::{
        operational_request_resource::OperationalResourceRequest,
        operational_request_scheduling::OperationalSchedulingRequest,
        operational_request_time::OperationalTimeRequest,
        operational_response_resource::OperationalResourceResponse,
        operational_response_scheduling::OperationalSchedulingResponse,
        operational_response_time::OperationalTimeResponse,
    },
};

use crate::agents::traits::LargeNeighborHoodSearch;

pub struct OperationalAlgorithm {
    pub objective_value: f64,
    pub time_window:
        HashMap<(WorkOrderNumber, ActivityNumber, DateTime<Utc>), OperationalParameters>,
    pub availability: Option<Availability>,
}

impl OperationalAlgorithm {
    pub fn new() -> Self {
        Self {
            objective_value: f64::INFINITY,
            time_window: HashMap::new(),
            availability: None,
        }
    }
}

pub struct OperationalParameters {
    start_window: DateTime<Utc>,
    end_window: DateTime<Utc>,
}

impl LargeNeighborHoodSearch for OperationalAlgorithm {
    type SchedulingRequest = OperationalSchedulingRequest;

    type SchedulingResponse = OperationalSchedulingResponse;

    type ResourceRequest = OperationalResourceRequest;

    type ResourceResponse = OperationalResourceResponse;

    type TimeRequest = OperationalTimeRequest;

    type TimeResponse = OperationalTimeResponse;

    type Error = AgentError;

    fn calculate_objective_value(&mut self) {
        todo!()
    }

    fn schedule(&mut self) {
        todo!()
    }

    fn unschedule(&mut self, message: WorkOrderNumber) {
        todo!()
    }

    fn update_scheduling_state(
        &mut self,
        message: Self::SchedulingRequest,
    ) -> Result<Self::SchedulingResponse, Self::Error> {
        todo!()
    }

    fn update_time_state(
        &mut self,
        message: Self::TimeRequest,
    ) -> Result<Self::TimeResponse, Self::Error> {
        todo!()
    }

    fn update_resources_state(
        &mut self,
        message: Self::ResourceRequest,
    ) -> Result<Self::ResourceResponse, Self::Error> {
        todo!()
    }
}
