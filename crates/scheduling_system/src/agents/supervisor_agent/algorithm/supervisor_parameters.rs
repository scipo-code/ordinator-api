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

        for (work_order_number, work_order) in &scheduling_environment.work_orders.inner {
            for (activity_number, operation) in &work_order.operations {
                let work_order_activity = &(*work_order_number, *activity_number);
                supervisor_algorithm
                    .supervisor_parameters
                    .create_and_insert_supervisor_parameter(operation, work_order_activity);

                for operational_agent in supervisor_algorithm
                    // This should run on the `SchedulingEnvironment::worker_environment`
                    .loaded_shared_solution
                    .operational
                    .keys()
                {
                    if operational_agent.1.contains(
                        &supervisor_algorithm
                            .supervisor_parameters
                            .supervisor_parameter(work_order_activity)
                            .context("The SupervisorParameter was not found")?
                            .resource,
                    ) {
                        let operation = scheduling_environment_guard.operation(work_order_activity);
                        let delegate = Delegate::build(operation);
                    }
                }
            }
        }

        Ok(Self {
            supervisor_work_orders: HashMap::new(),
            supervisor_periods: supervisor_periods.clone(),
            resources,
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
        let supervisor_parameter =
            SupervisorParameter::new(operation.resource, operation.operation_info.number);
        let _assert_option = self
            .supervisor_work_orders
            .entry(work_order_activity.0)
            .or_default()
            .insert(work_order_activity.1, supervisor_parameter);
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
