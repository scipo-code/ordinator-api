use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicUsize, Arc, MutexGuard},
};

use anyhow::{Context, Result};
use arc_swap::Guard;
use shared_types::{
    scheduling_environment::{
        time_environment::period::Period,
        work_order::{
            operation::{operation_info::NumberOfPeople, ActivityNumber},
            WorkOrderActivity, WorkOrderNumber,
        },
        worker_environment::resources::{Id, Resources},
        SchedulingEnvironment,
    },
    supervisor::{
        supervisor_response_resources::SupervisorResponseResources,
        supervisor_response_scheduling::SupervisorResponseScheduling,
        supervisor_response_time::SupervisorResponseTime, SupervisorObjectiveValue,
    },
};
use tracing::{event, Level};

use crate::agents::{
    operational_agent::algorithm::OperationalObjectiveValue, traits::LargeNeighborHoodSearch,
    ArcSwapSharedSolution, SharedSolution,
};

use super::{delegate::Delegate, operational_state_machine::OperationalStateMachine};

#[derive(Debug, Clone)]
pub struct MarginalFitness(Arc<AtomicUsize>);
// pub struct MarginalFitness(pub Arc<AtomicUsize>);

impl MarginalFitness {
    pub fn inner(&self) -> usize {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn store(&self, value: usize) {
        self.0.store(value, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        let self_value = self.inner();
        let other_value = other.inner();

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
    pub objective_value: SupervisorObjectiveValue,
    pub resources: Vec<Resources>,
    pub supervisor_parameters: SupervisorParameters,
    pub operational_state_machine: OperationalStateMachine,
    arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
    pub loaded_shared_solution: Guard<Arc<SharedSolution>>,
    pub operational_agent_objectives: HashMap<Id, OperationalObjectiveValue>,
}

pub struct SupervisorParameters {
    pub supervisor_work_orders:
        HashMap<WorkOrderNumber, HashMap<ActivityNumber, SupervisorParameter>>,
    pub supervisor_periods: Vec<Period>,
}

impl SupervisorParameters {
    pub fn new(supervisor_periods: Vec<Period>) -> Self {
        Self {
            supervisor_work_orders: HashMap::new(),
            supervisor_periods,
        }
    }

    pub(crate) fn supervisor_parameter(
        &self,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<&SupervisorParameter> {
        Ok(self.supervisor_work_orders
            .get(&work_order_activity.0)
            .context(format!("WorkOrderNumber: {:?} was not part of the SupervisorParameters", work_order_activity.0))?
            .get(&work_order_activity.1)
            .context(format!("WorkOrderNumber: {:?} with ActivityNumber: {:?} was not part of the SupervisorParameters", work_order_activity.0, work_order_activity.1))?)
    }

    pub(crate) fn create(
        &mut self,
        scheduling_environment_lock: &MutexGuard<SchedulingEnvironment>,
        work_order_activity: &WorkOrderActivity,
    ) {
        let operation = scheduling_environment_lock.operation(work_order_activity);

        let supervisor_parameter =
            SupervisorParameter::new(operation.resource.clone(), operation.operation_info.number);
        self.supervisor_work_orders
            .entry(work_order_activity.0)
            .or_insert_with(HashMap::new)
            .insert(work_order_activity.1, supervisor_parameter);
    }
}

pub struct SupervisorParameter {
    pub resource: Resources,
    pub number: NumberOfPeople,
}

impl SupervisorParameter {
    pub fn new(resource: Resources, number: NumberOfPeople) -> Self {
        Self { resource, number }
    }
}

impl SupervisorAlgorithm {
    pub fn new(
        resources: Vec<Resources>,
        arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
        supervisor_periods: &[Period],
    ) -> Self {
        let loaded_shared_solution = arc_swap_shared_solution.0.load();
        Self {
            objective_value: SupervisorObjectiveValue::default(),
            resources,
            supervisor_parameters: SupervisorParameters::new(supervisor_periods.to_vec()),
            operational_state_machine: OperationalStateMachine::default(),
            operational_agent_objectives: HashMap::default(),
            arc_swap_shared_solution,
            loaded_shared_solution,
        }
    }

    pub fn load_shared_solution(&mut self) {
        self.loaded_shared_solution = self.arc_swap_shared_solution.0.load();
    }
}

impl LargeNeighborHoodSearch for SupervisorAlgorithm {
    type BetterSolution = SupervisorObjectiveValue;
    type SchedulingRequest = SupervisorSchedulingRequest;
    type SchedulingResponse = SupervisorResponseScheduling;
    type ResourceRequest = SupervisorResourceRequest;
    type ResourceResponse = SupervisorResponseResources;
    type TimeRequest = SupervisorTimeRequest;
    type TimeResponse = SupervisorResponseTime;

    type SchedulingUnit = WorkOrderNumber;

    fn calculate_objective_value(&mut self) -> Self::BetterSolution {
        let assigned_woas = &self
            .operational_state_machine
            .number_of_assigned_work_orders();

        let all_woas: HashSet<_> = self.operational_state_machine.get_work_order_activities();

        assert!(is_assigned_part_of_all(assigned_woas, &all_woas));

        let mut intermediate = assigned_woas.len() as f64 / all_woas.len() as f64;
        if intermediate.is_nan() {
            intermediate = 0.0;
        };

        let objective_value = (intermediate * 1000.0) as u64;

        self.objective_value = objective_value;
        objective_value
    }

    fn schedule(&mut self) {
        'next_work_order_activity: for work_order_activity in
            &self.operational_state_machine.get_work_order_activities()
        {
            event!(Level::WARN, "DETERMINE FLOW");
            let number = self
                .supervisor_parameters
                .supervisor_work_orders
                .get(&work_order_activity.0)
                .expect("The supervisor parameter should always be available")
                .get(&work_order_activity.1)
                .expect("The SupervisorParameter should always be available")
                .number;

            event!(Level::WARN, "DETERMINE FLOW");
            let mut operational_status_by_woa = self
                .operational_state_machine
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

            event!(Level::WARN, "DETERMINE FLOW");
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
        self.operational_state_machine
            .turn_work_order_into_delegate_assess(work_order_number);
        Ok(())
    }

    fn update_scheduling_state(
        &mut self,
        _message: Self::SchedulingRequest,
    ) -> Result<Self::SchedulingResponse> {
        todo!()
    }

    fn update_time_state(&mut self, _message: Self::TimeRequest) -> Result<Self::TimeResponse> {
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
