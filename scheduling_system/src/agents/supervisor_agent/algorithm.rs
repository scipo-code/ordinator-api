use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicUsize, Arc},
};

use shared_types::{
    agent_error::AgentError,
    scheduling_environment::{
        work_order::{operation::ActivityNumber, WorkOrderActivity, WorkOrderNumber},
        worker_environment::resources::{Id, Resources},
    },
    supervisor::{
        supervisor_response_resources::SupervisorResponseResources,
        supervisor_response_scheduling::SupervisorResponseScheduling,
        supervisor_response_time::SupervisorResponseTime,
    },
    TomlSupervisor,
};
use tracing::{event, instrument, span, Level};

use crate::agents::{
    operational_agent::{algorithm::OperationalObjective, InitialMessage, OperationalAgent},
    tactical_agent::tactical_algorithm::TacticalOperation,
    traits::LargeNeighborHoodSearch,
    StateLink, StateLinkWrapper,
};

use super::{
    delegate::{AtomicDelegate, Delegate},
    operational_state_machine::OperationalStateMachine,
    SupervisorAgent, TransitionTypes,
};

#[derive(Debug, Clone)]
pub struct MarginalFitness(pub Arc<AtomicUsize>);

impl MarginalFitness {
    pub fn is_scheduled(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::SeqCst) == usize::MAX
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        let self_value = self.0.load(std::sync::atomic::Ordering::Acquire);
        let other_value = other.0.load(std::sync::atomic::Ordering::Acquire);

        if self_value == other_value {
            return Ordering::Equal;
        } else if self_value > other_value {
            return Ordering::Greater;
        } else {
            return Ordering::Less;
        }
    }
}

impl Default for MarginalFitness {
    fn default() -> Self {
        MarginalFitness(Arc::new(AtomicUsize::new(usize::MAX)))
    }
}

pub struct SupervisorSchedulingRequest;
pub struct SupervisorResourceRequest;
pub struct SupervisorTimeRequest;

pub struct SupervisorAlgorithm {
    pub objective_value: f64,
    _resource: TomlSupervisor,
    pub operational_state: OperationalStateMachine,
    pub operational_agent_objectives: HashMap<Id, OperationalObjective>,
    pub tactical_operations: HashMap<WorkOrderActivity, Arc<TacticalOperation>>,
}

impl SupervisorAlgorithm {
    pub fn new(supervisor: TomlSupervisor) -> Self {
        Self {
            objective_value: f64::default(),
            _resource: supervisor,
            operational_state: OperationalStateMachine::default(),
            operational_agent_objectives: HashMap::default(),
            tactical_operations: HashMap::default(),
        }
    }

    pub fn objective_value(&self) -> f64 {
        self.objective_value
    }

    #[allow(dead_code)]
    pub fn is_assigned(&self, work_order_activity: WorkOrderActivity) -> bool {
        self.operational_state.get_iter().any(|(key, val)| {
            work_order_activity == key.1
                && val.0.load(std::sync::atomic::Ordering::Relaxed) == Delegate::Assign
        })
    }

    #[allow(dead_code)]
    pub fn number_woas_for_agent(&self, operational_agent: &Id) -> usize {
        self.operational_state
            .get_iter()
            .filter(|id| id.0 .0 == *operational_agent)
            .count()
    }
}

/// This type will contain all the relevant information handles to the operational agents
/// Delegation. This means that the code should... I think that it is simple the code should
/// simply be created in such a way that we only need to change the OperaitonalState and then
/// the correct messages will be sent out.

impl LargeNeighborHoodSearch for SupervisorAgent {
    type BetterSolution = ();
    type SchedulingRequest = SupervisorSchedulingRequest;
    type SchedulingResponse = SupervisorResponseScheduling;
    type ResourceRequest = SupervisorResourceRequest;
    type ResourceResponse = SupervisorResponseResources;
    type TimeRequest = SupervisorTimeRequest;
    type TimeResponse = SupervisorResponseTime;

    type SchedulingUnit = WorkOrderNumber;

    type Error = AgentError;

    fn calculate_objective_value(&mut self) -> Self::BetterSolution {
        let assigned_woas = &self
            .supervisor_algorithm
            .operational_state
            .number_of_assigned_work_orders();

        let all_woas: HashSet<_> = self
            .supervisor_algorithm
            .operational_state
            .get_work_order_activities();

        assert!(is_assigned_part_of_all(assigned_woas, &all_woas));

        self.supervisor_algorithm.objective_value =
            assigned_woas.len() as f64 / all_woas.len() as f64;
    }

    fn schedule(&mut self) {
        'next_work_order_activity: for work_order_activity in &self
            .supervisor_algorithm
            .operational_state
            .get_work_order_activities()
        {
            let number = self
                .supervisor_algorithm
                .tactical_operations
                .get(work_order_activity)
                .expect("The Tactical Operation should be in present if present in the s.s.operational_state")
                .number;

            let mut operational_status_by_woa = self
                .supervisor_algorithm
                .operational_state
                .operational_status_by_woa(&work_order_activity);

            operational_status_by_woa.sort_by(|a, b| a.2.compare(&b.2));

            let mut number_of_assigned: u64 = 0;
            for operational_agent in &operational_status_by_woa {
                if operational_agent
                    .1
                    .load(std::sync::atomic::Ordering::Acquire)
                    == Delegate::Assign
                {
                    number_of_assigned += 1;
                }
            }

            let mut remaining_work_order_activities_to_be_state_changed_to_delegate_assign =
                number - number_of_assigned;

            for operational_agent in &operational_status_by_woa {
                if operational_agent
                    .1
                    .load(std::sync::atomic::Ordering::Acquire)
                    != Delegate::Assess
                {
                    continue;
                }

                if operational_agent
                    .2
                     .0
                    .load(std::sync::atomic::Ordering::Acquire)
                    == usize::MAX
                {
                    continue 'next_work_order_activity;
                }

                if remaining_work_order_activities_to_be_state_changed_to_delegate_assign >= 1 {
                    remaining_work_order_activities_to_be_state_changed_to_delegate_assign -= 1;
                    operational_agent.1.state_change_to_assign();
                } else if remaining_work_order_activities_to_be_state_changed_to_delegate_assign
                    == 0
                {
                    if operational_agent
                        .1
                        .load(std::sync::atomic::Ordering::Acquire)
                        == Delegate::Assign
                    {
                        continue;
                    }

                    operational_agent.1.state_change_to_unassign()
                } else {
                    panic!();
                }
            }
        }
    }

    fn unschedule(&mut self, work_order_number: Self::SchedulingUnit) {
        self.supervisor_algorithm
            .operational_state
            .turn_work_order_into_delegate_assess(work_order_number);
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
