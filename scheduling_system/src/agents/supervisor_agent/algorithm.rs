use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    iter::Map,
    sync::{atomic::AtomicUsize, Arc},
};

use actix::Addr;
use proptest::strategy::statics::MapFn;
use shared_types::{
    agent_error::AgentError,
    scheduling_environment::{
        work_order::{
            operation::{operation_info::NumberOfPeople, ActivityNumber},
            WorkOrderActivity, WorkOrderNumber,
        },
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
    SupervisorAgent, TransitionTypes,
};

#[derive(Debug, Clone)]
pub struct MarginalFitness(pub Arc<AtomicUsize>);

impl MarginalFitness {
    fn is_scheduled(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::SeqCst) == usize::MAX
    }

    fn compare(&self, other: &Self) -> Ordering {
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
        self.operational_state.0.iter().any(|(key, val)| {
            work_order_activity == key.1
                && val.0.load(std::sync::atomic::Ordering::Relaxed) == Delegate::Assign
        })
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
    HashMap<(Id, WorkOrderActivity), (Arc<AtomicDelegate>, MarginalFitness)>,
);

/// This is a fundamental type. Where should we input the OperationalObjective? I think that keeping the
/// code clean of these kind of things is exactly what is needed to make this work.
impl OperationalStateMachine {
    pub fn update_operational_state(
        &mut self,
        transition_type: TransitionTypes,
        operational_agent: (&Id, &Addr<OperationalAgent>),
        supervisor_id: Id,
    ) {
        match transition_type {
            TransitionTypes::Entering((work_order_activity, tactical_operation)) => {
                let delegate = Arc::new(AtomicDelegate::new(Delegate::new()));
                let marginal_fitness = MarginalFitness::default();

                self.0.insert(
                    (operational_agent.0.clone(), work_order_activity),
                    (Arc::clone(&delegate), marginal_fitness.clone()),
                );

                let span = span!(
                    Level::DEBUG,
                    "SupervisorSpan.OperationalState.TransitionType::Entering"
                );

                let _entered = span.enter();

                let state_link = StateLink::Supervisor(InitialMessage::new(
                    work_order_activity,
                    delegate.clone(),
                    tactical_operation,
                    marginal_fitness,
                    supervisor_id,
                ));
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

                delegate.state_change_to_drop();

                self.remove_an_operational_state(woa, operational_agent.0.clone());
            }
            TransitionTypes::Done(_) => {
                panic!("You should never send a done request to an OperationalAgent");
            }
        }
    }

    pub fn turn_work_order_into_delegate_assess(&mut self, work_order_number: WorkOrderNumber) {
        let id_and_work_order_activities_to_turn_into_delegate_assess =
            self.0.keys().filter(|(_, woa)| woa.0 == work_order_number);

        for id_work_order_activity in id_and_work_order_activities_to_turn_into_delegate_assess {
            self.0
                .get(&id_work_order_activity)
                .unwrap()
                .0
                .store(Delegate::Assess, std::sync::atomic::Ordering::SeqCst)
        }
    }

    pub fn is_work_order_activity_present(&self, work_order_activity: &WorkOrderActivity) -> bool {
        self.0
            .keys()
            .map(|key| key.1)
            .collect::<HashSet<_>>()
            .contains(work_order_activity)
    }

    pub fn count_unique_woa(&self) -> usize {
        self.0.keys().map(|(_, woa)| woa).len()
    }

    pub fn remove_an_operational_state(
        &mut self,
        work_order_activity: WorkOrderActivity,
        operational_id: Id,
    ) {
        let value_option = self.0.remove(&(operational_id, work_order_activity));

        match value_option {
            Some(value) => {
                if !value.0.load(std::sync::atomic::Ordering::Acquire).is_drop() {
                    event!(
                        Level::ERROR,
                        value_in_atomic_delegate =
                            ?value.0.load(std::sync::atomic::Ordering::Acquire)
                    );
                    panic!("You tried to remove a delegate that was not Delegate::Drop, doing this could lead to a situation where the remaining state could be wrong");
                }
            }
            None => {
                panic!("You tried to remove an entry of the SupervisorAlgorithm OperationalState, which did not exist. This is a major violation of the internal consistency of the SupervisorAgent and all its OperationalAgents")
            }
        }
    }

    // TODO Rewrite this.
    pub fn are_unassigned_woas_valid(&self) -> bool {
        for work_order_activity in self.0.keys().map(|(_, woa)| woa).collect::<Vec<_>>() {
            // What is it that is mutable here?
            let mut delegates_by_woa = self
                .0
                .iter()
                .filter(|(key, _)| key.1 == *work_order_activity)
                .map(|(_, (delegates, _))| delegates);

            let is_all_assess = delegates_by_woa.all(|delegate| {
                delegate
                    .load(std::sync::atomic::Ordering::Acquire)
                    .is_assess()
                    || delegate
                        .load(std::sync::atomic::Ordering::Acquire)
                        .is_done()
            });

            let is_all_drop = delegates_by_woa.all(|delegate| {
                delegate
                    .load(std::sync::atomic::Ordering::Acquire)
                    .is_drop()
                    || delegate
                        .load(std::sync::atomic::Ordering::Acquire)
                        .is_done()
            });

            if !(is_all_drop || is_all_assess) {
                event!(Level::ERROR, delegate_by_woa = ?delegates_by_woa);
                return false;
            }
        }
        true
    }

    fn number_of_assigned_work_orders(&self) -> HashSet<WorkOrderActivity> {
        self.0
            .iter()
            .filter(|(_, val)| val.0.load(std::sync::atomic::Ordering::Acquire).is_assign())
            .map(|(key, _)| key.1)
            .collect()
    }

    pub fn operational_status_by_woa(
        &self,
        work_order_activity: &WorkOrderActivity,
    ) -> Vec<(Id, Arc<AtomicDelegate>, MarginalFitness)> {
        self.0
            .iter()
            .filter(|(key, _)| key.1 == *work_order_activity)
            .map(|(key, val)| (key.0.clone(), val.0.clone(), val.1.clone()))
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
            .collect::<Vec<(&WorkOrderActivity, &Arc<AtomicDelegate>)>>()
        {
            let number = number.get(work_order_activity).unwrap();

            let mut operational_marginal_fitnai: Vec<_> =
                self.operational_status_by_woa(*work_order_activity);

            if operational_marginal_fitnai
                .iter()
                .all(|objectives| objectives.2.is_scheduled())
            {
                operational_marginal_fitnai.sort_by(|a, b| a.2.compare(&b.2));

                let operational_solution_across_ids = operational_marginal_fitnai.iter().rev();

                let (top_operational_agents, remaining_operational_agents): (Vec<_>, Vec<_>) =
                    operational_solution_across_ids
                        .into_iter()
                        .enumerate()
                        .partition(|&(i, _)| i < *number as usize);

                // let tactical_operation = match delegate.load(std::sync::atomic::Ordering::Acquire) {
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
                    let transition_type = TransitionTypes::Unchanged(**work_order_activity);
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
        (Arc<AtomicDelegate>, MarginalFitness),
    > {
        self.0.iter()
    }

    pub fn get(
        &self,
        key: &(Id, WorkOrderActivity),
    ) -> Option<&(Arc<AtomicDelegate>, MarginalFitness)> {
        self.0.get(&(key))
    }

    pub(crate) fn get_assigned_and_unassigned_work_orders(&self) -> Vec<WorkOrderNumber> {
        self.0
            .iter()
            .filter(|(_, del_fit)| {
                let delegate = del_fit.0.load(std::sync::atomic::Ordering::Acquire);
                delegate == Delegate::Assign || delegate == Delegate::Unassign
            })
            .map(|(id_woa, _)| id_woa.1 .0)
            .collect()
    }
}

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
        'next_work_order_activity: for work_order_activity in &self
            .supervisor_algorithm
            .operational_state
            .0
            .keys()
            .map(|(_, woa)| woa)
            .collect::<HashSet<_>>()
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

pub fn assert_that_operational_state_machine_for_each_work_order_is_either_delegate_assign_and_unassign_or_all_assess(
    operational_state_machine: &OperationalStateMachine,
) {
    let work_order_numbers: HashSet<WorkOrderNumber> = operational_state_machine
        .0
        .iter()
        .map(|(id_woa, _)| (id_woa.1 .0))
        .collect();

    for work_order_number in work_order_numbers {
        let mut assess_work_orders: HashSet<WorkOrderNumber> = HashSet::new();
        let mut assign_unassign_work_orders: HashSet<WorkOrderNumber> = HashSet::new();

        operational_state_machine
            .0
            .iter()
            .filter(|(id_woa, _)| id_woa.1 .0 == work_order_number)
            .for_each(|osm| {
                let delegate = osm.1 .0.load(std::sync::atomic::Ordering::Acquire);

                if delegate == Delegate::Assess {
                    assess_work_orders.insert(work_order_number);
                } else if delegate == Delegate::Assign || delegate == Delegate::Unassign {
                    assign_unassign_work_orders.insert(work_order_number);
                }
            });

        assert!(!assess_work_orders.is_empty() || !assign_unassign_work_orders.is_empty());
    }
}
