use std::collections::HashMap;

use chrono::TimeDelta;
use shared_types::{
    operational::TimeInterval,
    scheduling_environment::{
        work_order::{operation::Work, WorkOrderActivity},
        worker_environment::availability::Availability,
    },
};

#[derive(Clone, Default)]
pub struct OperationalParameters {
    pub work_order_parameters: HashMap<WorkOrderActivity, OperationalParameter>,
    pub availability: Availability,
    pub off_shift_interval: TimeInterval,
    pub break_interval: TimeInterval,
    pub toolbox_interval: TimeInterval,
}
impl OperationalParameters {
    pub(crate) fn new(
        scheduling_environment: Guard,
        operational_configuration: &shared_types::operational::OperationalConfiguration,
    ) -> Self {
        let mut work_order_parameters = HashMap::default();

        for (work_order_number, work_order) in scheduling_environment.work_orders.inner {
            for (activity_number, operation) in work_order.operations() {
                let work_order_activity = (*work_order_number, *activity_number);

                let operational_parameter_option = OperationalParameter::new(
                    operation.work_remaining().unwrap(),
                    operation.operation_analytic.preparation_time,
                );

                let operational_parameter = match operational_parameter_option {
                    Some(operational_parameter) => operational_parameter,
                    None => continue,
                };

                work_order_parameters.insert(work_order_activity, operational_parameter);
            }
        }

        Self {
            work_order_parameters,
            availability: operational_configuration.availability.clone(),
            off_shift_interval: operational_configuration.off_shift_interval.clone(),
            break_interval: operational_configuration.break_interval.clone(),
            toolbox_interval: operational_configuration.toolbox_interval.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OperationalParameter {
    pub work: Work,
    // TODO: INCLUDE PREPARATION
    pub _preparation: Work,
    pub operation_time_delta: TimeDelta,
    // start_window: DateTime<Utc>,
    // end_window: DateTime<Utc>,
    // pub delegated: Delegate,
    // marginal_fitness: MarginalFitness,
}

impl OperationalParameter {
    pub fn new(
        work: Work,
        _preparation: Work,
        // start_window: DateTime<Utc>,
        // end_window: DateTime<Utc>,
        // delegated: Delegate,
        // marginal_fitness: MarginalFitness,
    ) -> Option<Self> {
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
