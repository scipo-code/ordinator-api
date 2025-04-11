use std::collections::HashMap;
use std::sync::Arc;
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use arc_swap::Guard;
use chrono::TimeDelta;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::TimeInterval;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::availability::Availability;
use ordinator_scheduling_environment::worker_environment::resources::Id;

use crate::OperationalOptions;

pub struct OperationalParameters
{
    pub work_order_parameters: HashMap<WorkOrderActivity, OperationalParameter>,
    pub availability: Availability,
    pub off_shift_interval: TimeInterval,
    pub break_interval: TimeInterval,
    pub toolbox_interval: TimeInterval,
    pub options: OperationalOptions,
}

// There is something rotten about this function.
impl Parameters for OperationalParameters
{
    type Key = WorkOrderActivity;

    // You should not put it in the Options

    // Do we even want the code to look like this in the first place?
    fn from_source(
        asset: &Id,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
        system_configurations: &Guard<Arc<SystemConfigurations>>,
    ) -> Result<Self>
    {
        let mut work_order_parameters = HashMap::default();

        for (work_order_number, work_order) in &scheduling_environment.work_orders.inner {
            for (activity_number, operation) in &work_order.operations.0 {
                let work_order_activity = (*work_order_number, *activity_number);

                let operational_parameter_option = OperationalParameter::new(
                    operation.operation_info.work_remaining,
                    operation.operation_analytic.preparation_time,
                );

                let operational_parameter = match operational_parameter_option {
                    Some(operational_parameter) => operational_parameter,
                    None => continue,
                };

                work_order_parameters.insert(work_order_activity, operational_parameter);
            }
        }

        let operational_configuration = &scheduling_environment
            .worker_environment
            .agent_environment
            .operational
            .values()
            .find(|oca| asset == &oca.id)
            .with_context(|| format!("{:#?} did not exist", asset.0))?
            .operational_configuration;

        let options = OperationalOptions::from((system_configurations, asset));

        Ok(Self {
            work_order_parameters,
            availability: operational_configuration.availability.clone(),
            off_shift_interval: operational_configuration.off_shift_interval.clone(),
            break_interval: operational_configuration.break_interval.clone(),
            toolbox_interval: operational_configuration.toolbox_interval.clone(),
            options,
        })
    }

    fn create_and_insert_new_parameter(
        &mut self,
        key: Self::Key,
        scheduling_environment: MutexGuard<SchedulingEnvironment>,
    )
    {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct OperationalParameter
{
    pub work: Work,
    // TODO: INCLUDE PREPARATION
    pub _preparation: Work,
    pub operation_time_delta: TimeDelta,
    // start_window: DateTime<Utc>,
    // end_window: DateTime<Utc>,
    // pub delegated: Delegate,
    // marginal_fitness: MarginalFitness,
}

impl OperationalParameter
{
    pub fn new(
        work: Work,
        _preparation: Work,
        // start_window: DateTime<Utc>,
        // end_window: DateTime<Utc>,
        // delegated: Delegate,
        // marginal_fitness: MarginalFitness,
    ) -> Option<Self>
    {
        let combined_time = (work + _preparation).in_seconds();
        let operation_time_delta = TimeDelta::new(combined_time as i64, 0).unwrap();
        if work.to_f64() == 0.0 {
            return None;
        }
        Some(Self {
            work,
            _preparation,
            operation_time_delta,
            // start_window,
            // end_window,
            // delegated,
            // marginal_fitness,
        })
    }
}
