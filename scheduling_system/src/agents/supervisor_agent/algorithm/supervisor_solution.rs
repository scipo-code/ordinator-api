use std::collections::HashSet;

use actix::Addr;
use anyhow::Result;
use shared_types::scheduling_environment::{
    work_order::{WorkOrderActivity, WorkOrderNumber},
    worker_environment::resources::Id,
};

use crate::agents::{operational_agent::OperationalAgent, SupervisorSolution};

use super::delegate::Delegate;

/// The SupervisorSolution is a state machine that keeps track of all the
/// states of the operational agents. It is a solution representation of
/// a **iterative combinatorial auction algorithms**.
impl SupervisorSolution {
    pub fn len(&self) -> usize {
        self.operational_state_machine.len()
    }

    pub fn insert_supervisor_solution(
        &mut self,
        operational_agent: (&Id, &Addr<OperationalAgent>),
        delegate: Delegate,
        work_order_activity: WorkOrderActivity,
    ) -> Result<()> {
        self.operational_state_machine
            .insert((operational_agent.0.clone(), work_order_activity), delegate);
        Ok(())
    }

    pub fn turn_work_order_into_delegate_assess(&mut self, work_order_number: WorkOrderNumber) {
        self.operational_state_machine
            .iter_mut()
            .filter(|(key, _)| key.1 .0 == work_order_number)
            .for_each(|(_, delegate)| *delegate = Delegate::Assess)
    }

    pub fn count_unique_woa(&self) -> usize {
        self.operational_state_machine
            .keys()
            .map(|(_, woa)| woa)
            .len()
    }

    pub fn number_of_assigned_work_orders(&self) -> HashSet<WorkOrderActivity> {
        self.operational_state_machine
            .iter()
            .filter(|(_, val)| val.is_assign())
            .map(|(key, _)| key.1)
            .collect()
    }

    pub fn operational_status_by_work_order_activity(
        &self,
        work_order_activity: &WorkOrderActivity,
    ) -> Vec<(Id, Delegate)> {
        self.operational_state_machine
            .iter()
            .filter(|(key, _)| key.1 == *work_order_activity)
            .map(|(key, val)| (key.0.clone(), val.clone()))
            .collect()
    }

    pub(crate) fn get_iter(
        &self,
    ) -> std::collections::hash_map::Iter<(Id, WorkOrderActivity), Delegate> {
        self.operational_state_machine.iter()
    }

    pub(crate) fn get_assigned_and_unassigned_work_orders(&self) -> Vec<WorkOrderNumber> {
        self.operational_state_machine
            .iter()
            .filter(|(_, delegate)| {
                **delegate == Delegate::Assign || **delegate == Delegate::Unassign
            })
            .map(|(id_woa, _)| id_woa.1 .0)
            .collect()
    }

    pub(crate) fn get_work_order_activities(&self) -> HashSet<WorkOrderActivity> {
        self.operational_state_machine
            .keys()
            .map(|(_, woa)| woa)
            .cloned()
            .collect()
    }
}