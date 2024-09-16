use std::{collections::{HashMap, HashSet}, sync::{Arc, RwLock}};

use actix::Addr;
use chrono::TimeDelta;
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
use tracing::{event, instrument, span, Level};

use crate::agents::{
    operational_agent::{algorithm::OperationalObjective, OperationalAgent},
    traits::LargeNeighborHoodSearch,
    StateLink, StateLinkWrapper,
};

use super::{delegate::Delegate, delegate::DelegateAndId, SupervisorAgent, TransitionTypes};
pub type MarginalFitness = TimeDelta;

pub struct SupervisorSchedulingRequest;
pub struct SupervisorResourceRequest;
pub struct SupervisorTimeRequest;

pub struct SupervisorAlgorithm {
    objective_value: f64,
    _resource: MainResources,
    pub operational_state: OperationalStateMachine,
}

impl SupervisorAlgorithm {
    pub fn new(resource: MainResources) -> Self {
        Self {
            objective_value: f64::default(),
            _resource: resource,
            operational_state: OperationalStateMachine::default(),
        }
    }

    pub fn objective_value(&self) -> f64 {
        self.objective_value
    }


    #[allow(dead_code)]
    pub fn is_assigned(&self, work_order_activity: WorkOrderActivity) -> bool {
        self.operational_state
            .0
            .iter()
            .any(|(key, val)| work_order_activity == key.1 && val.0.read().unwrap().is_assign())
    }

    #[allow(dead_code)]
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
pub struct OperationalStateMachine(
    HashMap<(Id, WorkOrderActivity), (Arc<RwLock<Delegate>>, Option<OperationalObjective>, Option<MarginalFitness>)>,
);

/// This is a fundamental type. Where should we input the OperationalObjective? I think that keeping the
/// code clean of these kind of things is exactly what is needed to make this work.
impl OperationalStateMachine {
    pub fn update_operaitonal_state(
        &mut self,
        transition_type: TransitionTypes,
        operational_agent: (&Id, &Addr<OperationalAgent>),
        supervisor_id: Id,
    ) {
        match transition_type {
            TransitionTypes::Entering((work_order_activity, tactical_operation)) => {
                let delegate = Arc::new(RwLock::new(Delegate::new(work_order_activity, tactical_operation)));

                self.0.insert(
                    (operational_agent.0.clone(), work_order_activity),
                    (Arc::clone(&delegate), None, None),
                );

                let span = span!(Level::DEBUG, "SupervisorSpan.OperationalState.TransitionType::Entering");

                let _entered = span.enter();

                let state_link =
                    StateLink::Supervisor(DelegateAndId(delegate.clone(), supervisor_id));
                let state_link_wrapper = StateLinkWrapper::new(state_link, span.clone());

                operational_agent.1.do_send(state_link_wrapper)
            }
            TransitionTypes::Unchanged(_delegate) => {}
            TransitionTypes::Changed(_delegate) => {}
            TransitionTypes::Leaving(woa) => {
                let delegate_option = self.0.get(&(operational_agent.0.clone(), woa));

                let delegate = delegate_option
                    .expect("Cannot Delegate::Drop a WOA that is not in the already in the OperationalState")
                    .0
                    .clone();

                delegate.write().unwrap().state_change_to_drop();

                self.remove_an_operational_state(
                    woa, operational_agent.0.clone()
                );
            }
            TransitionTypes::Done(work_order_activity) => {
                let delegate = Arc::new(RwLock::new(Delegate::Done(work_order_activity)));

                self.0.insert(
                    (operational_agent.0.clone(), work_order_activity),
                    (delegate, None, None),
                );
                
            }
        }
    }
    
    pub fn count_unique_woa(&self) -> usize {
        self.0.keys().map(|(_, woa)| woa).len()
    }

    pub fn remove_an_operational_state(&mut self, work_order_activity: WorkOrderActivity, operational_id: Id) {
        let value_option = self.0.remove(&(operational_id, work_order_activity));

        match value_option {
            Some(value) => {
                if !value.0.read().unwrap().is_drop() {
                    panic!("You tried to remove a delegate that was not Delegate::Drop, doing this could lead to a situation where the remaining state could be wrong");
                }
            }
            None => {
                panic!("You tried to remove an entry of the SupervisorAlgorithm OperationalState, which did not exist. This is a major violation of the internal consistency of the SupervisorAgent and all its OperationalAgents")
            }    
        }
    }

    pub fn are_unassigned_woas_valid(&self) -> bool {
        for work_order_activity in self.0.keys().map(|(_, woa)| woa).collect::<Vec<_>>() {
           
            // What is it that is mutable here? 
            let mut delegates_by_woa = self.0.iter().filter(|(key, _)| {
                key.1 == *work_order_activity
            }).map(|(_,(delegates, _, _))| delegates );

            let is_all_assess = delegates_by_woa.all(|delegate| {
                 delegate.read().unwrap().is_assess() || delegate.read().unwrap().is_done() 
            });

            let is_all_drop = delegates_by_woa.all(|delegate| {
                delegate.read().unwrap().is_drop() || delegate.read().unwrap().is_done() 
            });

            if !(is_all_drop || is_all_assess) {
                event!(Level::ERROR, delegate_by_woa = ?delegates_by_woa);
                return false
            }
        }
        true
    }

    fn number_of_assigned_work_orders(&self) -> HashSet<WorkOrderActivity> {
        self.0
            .iter()
            .filter(|(_, val)| val.0.read().unwrap().is_assign())
            .map(|(key, _)| key.1)
            .collect()
    }

    #[allow(dead_code)]
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

    pub fn get_unique_woa(&self) -> HashSet<(WorkOrderNumber, ActivityNumber)> {
        self.0.keys().map(|(_, woa)| woa).cloned().collect()
    }

    #[allow(dead_code)]
    fn determine_delegate_assign_and_drop(
        &self,
        number: HashMap<WorkOrderActivity, NumberOfPeople>,
    ) -> Vec<(Id, TransitionTypes)> {
        let mut transition_sequence = Vec::<(Id, TransitionTypes)>::new();

        for (work_order_activity, _delegate) in &self
            .0
            .iter()
            .map(|(key, value)| (&key.1, &value.0))
            .collect::<Vec<(&WorkOrderActivity, &Arc<RwLock<Delegate>>)>>()
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

                // let tactical_operation = match delegate.read().unwrap() {
                //     Delegate::Assess((_, ref os)) => os,
                //     Delegate::Assign((_, ref os)) => os,
                //     Delegate::Drop(_) => panic!("The method that caused this panic should not deal with Delegate::Drop variants"),
                //     Delegate::Done(_) => panic!("Delegate::Done TacticalOperations should not be propagated through the system"),
                //     Delegate::Fixed => todo!(),
                    
                // };

                for toa in top_operational_agents {
                    let transition_type = TransitionTypes::Unchanged(**work_order_activity);
                    transition_sequence.push((toa.1 .0.clone(), transition_type));
                }

                for roa in remaining_operational_agents {
                    let transition_type =
                        TransitionTypes::Unchanged(**work_order_activity);
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
        (Arc<RwLock<Delegate>>, Option<OperationalObjective>, Option<MarginalFitness>),
    > {
        self.0.iter()
    }

    pub fn get(&self, key: &(Id, WorkOrderActivity)) -> Option<&(Arc<RwLock<Delegate>>, Option<OperationalObjective>, Option<MarginalFitness>)> {
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
