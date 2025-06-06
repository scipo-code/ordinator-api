use std::collections::HashMap;
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use anyhow::ensure;
use chrono::TimeDelta;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::TimeInterval;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::OperationalOptions;
use ordinator_scheduling_environment::worker_environment::availability::Availability;
use ordinator_scheduling_environment::worker_environment::resources::Id;

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
        // This is not needed. It should always be a part of your SchedulingEnvironment.
        // Yes this is the best approach here.
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

                // Are we mutating this function?
                let operational_parameter = match operational_parameter_option.ok() {
                    Some(operational_parameter) => operational_parameter,
                    None => continue,
                };
                ensure!(
                    !operational_parameter.work.is_zero(),
                    "Work for an activity should never be zero in the OperationalActor"
                );

                work_order_parameters.insert(work_order_activity, operational_parameter);
            }
        }

        let operational_configuration = &scheduling_environment
            .worker_environment
            .actor_specification
            .get(asset.asset())
            .unwrap()
            .operational
            .iter()
            .find(|oca| asset == &oca.id)
            .with_context(|| format!("{:#?} did not exist", asset.0))?;

        // What you have been doing is really silly here. You should work on improving
        // this as much as possible.
        Ok(Self {
            work_order_parameters,
            availability: operational_configuration
                .operational_configuration
                .availability
                .clone(),
            off_shift_interval: operational_configuration
                .operational_configuration
                .off_shift_interval
                .clone(),
            break_interval: operational_configuration
                .operational_configuration
                .break_interval
                .clone(),
            toolbox_interval: operational_configuration
                .operational_configuration
                .toolbox_interval
                .clone(),
            options: OperationalOptions {
                number_of_removed_activities: operational_configuration
                    .operational_options
                    .number_of_removed_activities,
            },
        })
    }

    fn create_and_insert_new_parameter(
        &mut self,
        _key: Self::Key,
        _scheduling_environment: MutexGuard<SchedulingEnvironment>,
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
    ) -> Result<Self>
    {
        //
        let combined_time = (work + _preparation).in_seconds();
        let operation_time_delta = TimeDelta::new(combined_time, 0).unwrap();
        ensure!(work.to_f64() > 0.0);
        ensure!(operation_time_delta > TimeDelta::new(0, 0).unwrap());
        Ok(Self {
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
