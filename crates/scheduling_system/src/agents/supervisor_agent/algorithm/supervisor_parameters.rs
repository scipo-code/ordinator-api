use std::{collections::HashMap, sync::MutexGuard};

use anyhow::{Context, Result};
use shared_types::scheduling_environment::{
    time_environment::period::Period,
    work_order::{
        operation::{operation_info::NumberOfPeople, ActivityNumber, Operation},
        WorkOrderActivity, WorkOrderNumber,
    },
    worker_environment::resources::Resources,
    SchedulingEnvironment,
};

use crate::agents::{supervisor_agent::SupervisorOptions, traits::Parameters};

pub struct SupervisorParameters {
    pub supervisor_work_orders:
        HashMap<WorkOrderNumber, HashMap<ActivityNumber, SupervisorParameter>>,
    pub supervisor_periods: Vec<Period>,
    pub resources: Vec<Resources>,
}

impl Parameters for SupervisorParameters {
    type Key = WorkOrderActivity;
    type Options = SupervisorOptions;

    fn new(
        asset: &shared_types::Asset,
        options: Self::Options,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self> {
        let supervisor_periods = &scheduling_environment.time_environment.supervisor_periods;

        // What is it that you want to do here? I think that the best approach will be to make the system
        //
        //

        let mut supervisor_parameters = HashMap::new();
        for (work_order_number, work_order) in &scheduling_environment.work_orders.inner {
            let inner_map = work_order.operations.iter().map(|(acn, op)| {
                (
                    *acn,
                    SupervisorParameter::new(op.resource, op.operation_info.number),
                )
            });

            let _assert_option = supervisor_parameters.insert(*work_order_number, inner_map);

            assert!(_assert_option.is_none());
        }

        Ok(Self {
            supervisor_work_orders: HashMap::new(),
            supervisor_periods: supervisor_periods.clone(),
            resources: vec![],
        })
    }

    fn create_and_insert_new_parameter(
        &mut self,
        key: Self::Key,
        scheduling_environment: std::sync::MutexGuard<
            shared_types::scheduling_environment::SchedulingEnvironment,
        >,
    ) {
        todo!()
    }
}

impl SupervisorParameters {
    pub(crate) fn supervisor_parameter(
        &self,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<&SupervisorParameter> {
        let supervisor_parameter = self.supervisor_work_orders
            .get(&work_order_activity.0)
            .context(format!("WorkOrderNumber: {:?} was not part of the SupervisorParameters", work_order_activity.0))?
            .get(&work_order_activity.1)
            .context(format!("WorkOrderNumber: {:?} with ActivityNumber: {:?} was not part of the SupervisorParameters", work_order_activity.0, work_order_activity.1))?;

        Ok(supervisor_parameter)
    }

    pub(crate) fn create_and_insert_supervisor_parameter(
        &mut self,
        operation: &Operation,
        work_order_activity: &WorkOrderActivity,
    ) {
        // DEBUG: Make assertions here!
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
