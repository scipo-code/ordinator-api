use std::collections::HashMap;
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Operation;
use ordinator_scheduling_environment::work_order::operation::operation_info::NumberOfPeople;
use ordinator_scheduling_environment::worker_environment::SupervisorOptions;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use ordinator_scheduling_environment::worker_environment::resources::Resources;

pub struct SupervisorParameters {
    pub supervisor_work_orders:
        HashMap<WorkOrderNumber, HashMap<ActivityNumber, SupervisorParameter>>,
    pub supervisor_periods: Vec<Period>,
    pub operational_ids: Vec<Id>,
    pub options: SupervisorOptions,
}

impl Parameters for SupervisorParameters {
    type Key = WorkOrderActivity;

    fn from_source(
        id: &Id,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self> {
        let supervisor_periods = &scheduling_environment.time_environment.supervisor_periods;

        let mut supervisor_parameters = HashMap::new();

        for (work_order_number, work_order) in scheduling_environment
            .work_orders
            .inner
            .iter()
            .filter(|(_, wo)| {
                &wo.functional_location().asset
                    == id
                        .2
                        .first()
                        .expect("TODO: Implement multi-asset technicians")
            })
        {
            let inner_map = work_order
                .operations
                .0
                .iter()
                .map(|(acn, op)| {
                    (
                        *acn,
                        SupervisorParameter::new(op.resource, op.operation_info.number),
                    )
                })
                .collect();

            let _assert_option = supervisor_parameters.insert(*work_order_number, inner_map);

            assert!(_assert_option.is_none());
        }

        // FIX
        // You should not select all agents. You should instead pick the ones that fit
        // the correct supervisor. WARN
        // You made a huge mistake here! The types in the `SchedulingEnvironment` was
        // wrong and then you created state duplication to fix the issue.
        let operational_ids: Vec<Id> = scheduling_environment
            .worker_environment
            .actor_specification
            .get(id.asset())
            .unwrap()
            .operational
            .iter()
            // TODO [ ] - Start here.
            .map(|e| e.id)
            .collect();

        Ok(Self {
            supervisor_work_orders: supervisor_parameters,
            supervisor_periods: supervisor_periods.clone(),
            operational_ids,
            options,
        })
    }

    fn create_and_insert_new_parameter(
        &mut self,
        key: Self::Key,
        scheduling_environment: MutexGuard<SchedulingEnvironment>,
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

#[derive(Clone)]
pub struct SupervisorParameter {
    pub resource: Resources,
    pub number: NumberOfPeople,
}

impl SupervisorParameter {
    pub fn new(resource: Resources, number: NumberOfPeople) -> Self {
        Self { resource, number }
    }
}
