pub mod assert_functions;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use actix::Addr;
use shared_types::scheduling_environment::{
    work_order::{WorkOrderActivity, WorkOrderNumber},
    worker_environment::resources::Id,
};
use tracing::{event, span, Level};

use crate::agents::{
    operational_agent::{InitialMessage, OperationalAgent},
    StateLink, StateLinkWrapper,
};

use super::{
    algorithm::MarginalFitness,
    delegate::{AtomicDelegate, Delegate},
    CapturedSupervisorState, TransitionTypes,
};

#[derive(Debug, Default)]
pub struct OperationalStateMachine(
    HashMap<(Id, WorkOrderActivity), (Arc<AtomicDelegate>, MarginalFitness)>,
);

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
                if !value.0.load(std::sync::atomic::Ordering::SeqCst).is_drop() {
                    event!(
                        Level::ERROR,
                        value_in_atomic_delegate =
                            ?value.0.load(std::sync::atomic::Ordering::SeqCst)
                    );
                    panic!("You tried to remove a delegate that was not Delegate::Drop, doing this could lead to a situation where the remaining state could be wrong");
                }
            }
            None => {
                panic!("You tried to remove an entry of the SupervisorAlgorithm OperationalState, which did not exist. This is a major violation of the internal consistency of the SupervisorAgent and all its OperationalAgents")
            }
        }
    }

    pub fn number_of_assigned_work_orders(&self) -> HashSet<WorkOrderActivity> {
        self.0
            .iter()
            .filter(|(_, val)| val.0.load(std::sync::atomic::Ordering::SeqCst).is_assign())
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

    pub fn set_operational_state(&mut self, captured_supervisor_state: CapturedSupervisorState) {
        for (id_work_order_activity, delegate) in captured_supervisor_state.state_of_each_agent {
            let self_delegate = self
                .0
                .get_mut(&id_work_order_activity)
                .unwrap()
                .0
                .swap(delegate, std::sync::atomic::Ordering::SeqCst);
        }
    }

    pub fn get(
        &self,
        key: &(Id, WorkOrderActivity),
    ) -> Option<&(Arc<AtomicDelegate>, MarginalFitness)> {
        self.0.get(&(key))
    }

    pub(crate) fn get_iter(
        &self,
    ) -> std::collections::hash_map::Iter<
        (Id, WorkOrderActivity),
        (Arc<AtomicDelegate>, MarginalFitness),
    > {
        self.0.iter()
    }

    pub(crate) fn get_assigned_and_unassigned_work_orders(&self) -> Vec<WorkOrderNumber> {
        self.0
            .iter()
            .filter(|(_, del_fit)| {
                let delegate = del_fit.0.load(std::sync::atomic::Ordering::SeqCst);
                delegate == Delegate::Assign || delegate == Delegate::Unassign
            })
            .map(|(id_woa, _)| id_woa.1 .0)
            .collect()
    }

    pub(crate) fn get_work_order_activities(&self) -> HashSet<WorkOrderActivity> {
        self.0.keys().map(|(_, woa)| woa).cloned().collect()
    }

    pub(crate) fn get_work_order_numbers(&self) -> HashSet<WorkOrderNumber> {
        self.0.iter().map(|(id_woa, _)| (id_woa.1 .0)).collect()
    }
}
