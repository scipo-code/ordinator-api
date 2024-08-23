use std::collections::{HashMap, HashSet};

use actix::Addr;
use shared_types::{
    agent_error::AgentError,
    scheduling_environment::{
        work_order::{
            operation::{operation_info::NumberOfPeople, ActivityNumber},
            WorkOrderActivity, WorkOrderNumber,
        },
        worker_environment::resources::{Id, MainResources},
    },
    supervisor::{
        supervisor_response_resources::SupervisorResponseResources,
        supervisor_response_scheduling::SupervisorResponseScheduling,
        supervisor_response_time::SupervisorResponseTime,
    },
};
use tracing::{instrument, span, Level};

use crate::agents::{
    operational_agent::{algorithm::OperationalObjective, OperationalAgent},
    traits::LargeNeighborHoodSearch,
    StateLink, StateLinkWrapper,
};

use super::{Delegate, DelegateAndId, SupervisorAgent, TransitionTypes};

pub struct SupervisorSchedulingRequest;
pub struct SupervisorResourceRequest;
pub struct SupervisorTimeRequest;

pub struct SupervisorAlgorithm {
    objective_value: f64,
    resource: MainResources,
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

    pub fn objective_value(&self) -> f64 {
        self.objective_value
    }

    pub fn is_assigned(&self, work_order_activity: WorkOrderActivity) -> bool {
        self.operational_state
            .0
            .iter()
            .any(|(key, val)| work_order_activity == key.1 && val.0.is_assign())
    }

    pub fn number_woas_for_agent(&self, operational_agent: &Id) -> usize {
        self.operational_state
            .0
            .iter()
            .filter(|id| id.0 .0 == *operational_agent)
            .count()
    }
}

/// This type will contain all the relevant information handles to the operational agents
/// Delegation. This means that the code should... I think that it is simple the code should
/// simply be created in such a way that we only need to change the OperaitonalState and then
/// the correct messages will be sent out.
#[derive(Debug, Default)]
pub struct OperationalState(
    HashMap<(Id, WorkOrderActivity), (Delegate, Option<OperationalObjective>)>,
);

/// This is a fundamental type. Where should we input the OperationalObjective? I think that keeping the
/// code clean of these kind of things is exactly what is needed to make this work.
impl OperationalState {
    pub fn handle_woa(
        &mut self,
        transition_type: TransitionTypes,
        operational_agent: (&Id, &Addr<OperationalAgent>),
        supervisor_id: Id,
    ) {
        match transition_type {
            TransitionTypes::Entering(delegate) => {
                let state_link =
                    StateLink::Supervisor(DelegateAndId(delegate.clone(), supervisor_id));
                self.0.insert(
                    (operational_agent.0.clone(), delegate.get_woa()),
                    (delegate, None),
                );
                let span = span!(Level::DEBUG, "SupervisorSpan.OperationalState");

                let state_link_wrapper = StateLinkWrapper::new(state_link, span.clone());

                operational_agent.1.do_send(state_link_wrapper)
            }
            TransitionTypes::Unchanged(delegate) => {}
            TransitionTypes::Changed(_delegate) => {}
            TransitionTypes::Leaving(delegate) => {
                // This should be a Arc<Delegate>! The operational agent should not be able to change his Delegate only view it
                // I think that the kind of errors that we can introduce into the code are catastrophic by applying these .clone()
                // methods. 
                let state_link = StateLink::Supervisor(DelegateAndId(delegate.clone()))
            }
        }
    }
    pub fn count_unique_woa(&self) -> usize {
        self.0.keys().map(|(_, woa)| woa).len()
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

    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn get_unique_woa(&self) -> HashSet<(WorkOrderNumber, ActivityNumber)> {
        self.0.keys().map(|(_, woa)| woa).cloned().collect()
    }

    fn determine_delegate_assign_and_drop(
        &self,
        number: HashMap<WorkOrderActivity, NumberOfPeople>,
    ) -> Vec<(Id, TransitionTypes)> {
        let mut transition_sequence = Vec::<(Id, TransitionTypes)>::new();

        for (work_order_activity, delegate) in &self
            .0
            .iter()
            .map(|(key, value)| (&key.1, &value.0))
            .collect::<Vec<(&WorkOrderActivity, &Delegate)>>()
        {
            let number = number.get(work_order_activity).unwrap();

            let mut operational_solution_across_ids: Vec<_> =
                self.determine_operational_objectives(**work_order_activity);

            if operational_solution_across_ids
                .iter()
                .all(|objectives| objectives.1.is_some())
            {
                operational_solution_across_ids
                    .sort_by(|a, b| a.1.unwrap().partial_cmp(&b.1.unwrap()).unwrap());

                let operational_solution_across_ids = operational_solution_across_ids.iter().rev();

                let (top_operational_agents, remaining_operational_agents): (Vec<_>, Vec<_>) =
                    operational_solution_across_ids
                        .into_iter()
                        .enumerate()
                        .partition(|&(i, _)| i < *number as usize);

                let operation_solution = match delegate {
                    Delegate::Assess((_, os)) => os,
                    Delegate::Assign((_, os)) => os,
                    Delegate::Drop(_) => panic!("The method that caused this panic should not deal with Delegate::Drop variants"),
                    Delegate::Fixed => todo!(),
                    
                };

                for toa in top_operational_agents {
                    let transition_type = TransitionTypes::Unchanged(Delegate::Assign((
                        **work_order_activity,
                        operation_solution.clone(),
                    )));
                    transition_sequence.push((toa.1 .0.clone(), transition_type));
                }

                for roa in remaining_operational_agents {
                    let transition_type =
                        TransitionTypes::Unchanged(Delegate::Drop(**work_order_activity));
                    transition_sequence.push((roa.1 .0.clone(), transition_type));
                }
            }
        }
        transition_sequence
    }

    pub(crate) fn get_iter(
        &self,
    ) -> std::collections::hash_map::Iter<
        (Id, (WorkOrderNumber, ActivityNumber)),
        (Delegate, Option<f64>),
    > {
        self.0.iter()
    }

    pub fn get(&self, key: &(Id, WorkOrderActivity)) -> Option<&(Delegate, Option<OperationalObjective>)> {
        self.0.get(&(key))
        
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
