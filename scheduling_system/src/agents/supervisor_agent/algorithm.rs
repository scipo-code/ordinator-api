use std::collections::{HashMap, HashSet};

use shared_types::{
    agent_error::AgentError,
    scheduling_environment::{
        work_order::{operation::ActivityNumber, WorkOrderActivity, WorkOrderNumber},
        worker_environment::resources::{Id, MainResources},
    },
    supervisor::{
        supervisor_response_resources::SupervisorResponseResources,
        supervisor_response_scheduling::SupervisorResponseScheduling,
        supervisor_response_time::SupervisorResponseTime,
    },
};
use tracing::{event, instrument, Level};

use crate::agents::{
    operational_agent::algorithm::OperationalObjective, traits::LargeNeighborHoodSearch,
};

use super::{Delegate, SupervisorAgent};

pub struct SupervisorSchedulingRequest;
pub struct SupervisorResourceRequest;
pub struct SupervisorTimeRequest;

pub struct SupervisorAlgorithm {
    pub objective_value: f64,
    pub resource: MainResources,
    pub operational_state: OperationalState,
}

impl SupervisorAlgorithm {
    pub fn new(resource: MainResources) -> Self {
        Self {
            objective_value: f64::default(),
            resource,
            operational_state: OperationalState::default(),
        }
    }

    pub fn is_assigned(&self, work_order_activity: WorkOrderActivity) -> bool {
        self.operational_state
            .0
            .iter()
            .any(|(key, val)| work_order_activity == key.1 && val.0.is_assign())
    }
}

/// This type will contain all the relevant information handles to the operational agents
/// Delegation. This means that the code should... I think that it is simple the code should
/// simply be created in such a way that we only need to change the OperaitonalState and then
/// the correct messages will be sent out.
#[derive(Debug, Default)]
pub struct OperationalState(
    pub HashMap<(Id, WorkOrderActivity), (Delegate, Option<OperationalObjective>)>,
);

/// This is a fundamental type. Where should we input the OperationalObjective? I think that keeping the
/// code clean of these kind of things is exactly what is needed to make this work.
impl OperationalState {
    pub fn insert_delegate(
        &mut self,
        key: (Id, WorkOrderActivity),
        delegate: Delegate,
        objective: Option<OperationalObjective>,
    ) {
        let previous_delegate = self.0.insert(key.clone(), (delegate.clone(), objective));

        match previous_delegate {
            Some(delegate_objective) => {
                event!(
                    Level::INFO,
                    delegate_objective = ?delegate_objective.0
                );
                dbg!(&delegate_objective, &delegate);
                assert!(delegate_objective.0.is_drop())
            }
            None => {
                event!(
                    Level::INFO,
                    operational_agent = key.0 .0,
                    "new Delegate::Assess",
                );
            }
        }
    }

    pub fn remove_delegate(&mut self, id_work_order_activity: &(Id, WorkOrderActivity)) {
        let removed_key = self.0.remove(id_work_order_activity);
        assert!(removed_key.is_some());
    }

    fn number_of_assigned_work_orders(&self) -> HashSet<WorkOrderActivity> {
        self.0
            .iter()
            .filter(|(_, val)| val.0.is_assign())
            .map(|(key, _)| key.1)
            .collect()
    }

    pub fn determine_operational_objectives(
        &self,
        work_order_activity: WorkOrderActivity,
    ) -> Vec<(Id, Option<OperationalObjective>)> {
        self.0
            .iter()
            .filter(|(key, _)| key.1 == work_order_activity)
            .map(|(key, val)| (key.0.clone(), val.1))
            .collect()
    }
}

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
        let assigned_woas = &self
            .supervisor_algorithm
            .operational_state
            .number_of_assigned_work_orders();

        let all_woas: HashSet<_> = self
            .supervisor_algorithm
            .operational_state
            .0
            .keys()
            .map(|(_, woa)| woa)
            .cloned()
            .collect();

        assert!(is_assigned_part_of_all(assigned_woas, &all_woas));

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

#[instrument(level = "trace", ret)]
fn is_assigned_part_of_all(
    assigned_woas: &HashSet<(WorkOrderNumber, ActivityNumber)>,
    all_woas: &HashSet<(WorkOrderNumber, ActivityNumber)>,
) -> bool {
    assigned_woas
        .iter()
        .map(|(wo, ac)| all_woas.contains(&(*wo, *ac)))
        .all(|present_woa| present_woa)
}
