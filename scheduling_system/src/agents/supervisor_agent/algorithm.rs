use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicUsize, Arc},
};

use anyhow::Result;
use arc_swap::Guard;
use shared_types::{
    agent_error::AgentError,
    scheduling_environment::{
        work_order::{operation::ActivityNumber, WorkOrderActivity, WorkOrderNumber},
        worker_environment::resources::Id,
    },
    supervisor::{
        supervisor_response_resources::SupervisorResponseResources,
        supervisor_response_scheduling::SupervisorResponseScheduling,
        supervisor_response_time::SupervisorResponseTime,
    },
    TomlSupervisor,
};

use crate::agents::{
    operational_agent::algorithm::OperationalObjective, tactical_agent::tactical_algorithm::TacticalOperation, traits::LargeNeighborHoodSearch, ArcSwapSharedSolution, SharedSolution
};

use super::{
    delegate::Delegate, operational_state_machine::OperationalStateMachine
    
};

#[derive(Debug, Clone)]
pub struct MarginalFitness(pub Arc<AtomicUsize>);

impl MarginalFitness {

    pub fn compare(&self, other: &Self) -> Ordering {
        let self_value = self.0.load(std::sync::atomic::Ordering::SeqCst);
        let other_value = other.0.load(std::sync::atomic::Ordering::SeqCst);

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
    arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
    pub loaded_shared_solution: Guard<Arc<SharedSolution>>,
    pub operational_agent_objectives: HashMap<Id, OperationalObjective>,
    pub tactical_operations: HashMap<WorkOrderActivity, Arc<TacticalOperation>>,
}

impl SupervisorAlgorithm {
    pub fn new(supervisor: TomlSupervisor, arc_swap_shared_solution: Arc<ArcSwapSharedSolution>) -> Self {

        let loaded_shared_solution = arc_swap_shared_solution.0.load();
        Self {
            objective_value: f64::default(),
            _resource: supervisor,
            operational_state: OperationalStateMachine::default(),
            operational_agent_objectives: HashMap::default(),
            arc_swap_shared_solution,
            tactical_operations: HashMap::default(),
            loaded_shared_solution,
        }
    }

    pub fn load_shared_solution(&mut self) {
        self.loaded_shared_solution = self.arc_swap_shared_solution.0.load();

    }

}

impl LargeNeighborHoodSearch for SupervisorAlgorithm {
    type BetterSolution = f64;
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
            
            .operational_state
            .number_of_assigned_work_orders();

        let all_woas: HashSet<_> = self
            
            .operational_state
            .get_work_order_activities();

        assert!(is_assigned_part_of_all(assigned_woas, &all_woas));

        let mut objective_value = assigned_woas.len() as f64 / all_woas.len() as f64;

        if objective_value.is_nan() {
            objective_value = 0.0
        }
        self.objective_value = objective_value;
        objective_value
    }

    fn schedule(&mut self) {
        'next_work_order_activity: for work_order_activity in &self
            
            .operational_state
            .get_work_order_activities()
        {
            let number = self
                
                .tactical_operations
                .get(work_order_activity)
                .expect("The Tactical Operation should be in present if present in the s.s.operational_state")
                .number;

            let mut operational_status_by_woa = self
                
                .operational_state
                .operational_status_by_woa(&work_order_activity);

            operational_status_by_woa.sort_by(|a, b| a.2.compare(&b.2));

            let mut number_of_assigned: u64 = 0;
            for operational_agent in &operational_status_by_woa {
                if operational_agent
                    .1
                    .load(std::sync::atomic::Ordering::SeqCst)
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
                    .load(std::sync::atomic::Ordering::SeqCst)
                    != Delegate::Assess
                {
                    continue;
                }

                if operational_agent
                    .2
                     .0
                    .load(std::sync::atomic::Ordering::SeqCst)
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
                        .load(std::sync::atomic::Ordering::SeqCst)
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

    fn unschedule(&mut self, work_order_number: Self::SchedulingUnit) -> Result<()> {
        self
            .operational_state
            .turn_work_order_into_delegate_assess(work_order_number);
        Ok(())
    }

    fn update_scheduling_state(
        &mut self,
        _message: Self::SchedulingRequest,
    ) -> Result<Self::SchedulingResponse> {
        todo!()
    }

    fn update_time_state(
        &mut self,
        _message: Self::TimeRequest,
    ) -> Result<Self::TimeResponse> {
        todo!()
    }

    fn update_resources_state(
        &mut self,
        _message: Self::ResourceRequest,
    ) -> Result<Self::ResourceResponse> {
        todo!()
    }
}

fn is_assigned_part_of_all(
    assigned_woas: &HashSet<(WorkOrderNumber, ActivityNumber)>,
    all_woas: &HashSet<(WorkOrderNumber, ActivityNumber)>,
) -> bool {
    assigned_woas
        .iter()
        .map(|(wo, ac)| all_woas.contains(&(*wo, *ac)))
        .all(|present_woa| present_woa)
}
