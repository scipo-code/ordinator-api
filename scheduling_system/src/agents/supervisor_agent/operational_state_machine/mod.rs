pub mod assert_functions;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use actix::Addr;
use anyhow::Result;
use shared_types::scheduling_environment::{
    work_order::{WorkOrderActivity, WorkOrderNumber},
    worker_environment::resources::Id,
};

use crate::agents::{operational_agent::OperationalAgent, SupervisorSolution};

use super::{
    algorithm::MarginalFitness,
    delegate::{AtomicDelegate, Delegate},
};

#[derive(Debug, Default)]
/// This is the StateMachine for the SupervisorAgent.
/// Here we will hold all the relevant state for the
pub struct OperationalStateMachine(
    HashMap<(Id, WorkOrderActivity), (Arc<AtomicDelegate>, MarginalFitness)>,
);

impl OperationalStateMachine {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn insert_supervisor_solution(
        &mut self,
        operational_agent: (&Id, &Addr<OperationalAgent>),
        work_order_activity: WorkOrderActivity,
    ) -> Result<()> {
        let delegate = Arc::new(AtomicDelegate::new(Delegate::new()));
        let marginal_fitness = MarginalFitness::default();

        self.0.insert(
            (operational_agent.0.clone(), work_order_activity),
            (Arc::clone(&delegate), marginal_fitness.clone()),
        );
        Ok(())
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

    pub fn count_unique_woa(&self) -> usize {
        self.0.keys().map(|(_, woa)| woa).len()
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

    pub fn set_operational_state(&mut self, captured_supervisor_state: SupervisorSolution) {
        for (id_work_order_activity, delegate) in captured_supervisor_state.state_of_each_agent {
            let self_delegate = self
                .0
                .get_mut(&id_work_order_activity)
                .unwrap()
                .0
                .swap(delegate, std::sync::atomic::Ordering::SeqCst);
            debug_assert_ne!(self_delegate, delegate);
        }
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
