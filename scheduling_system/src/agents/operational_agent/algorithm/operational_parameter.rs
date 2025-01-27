use std::collections::HashMap;

use chrono::TimeDelta;
use shared_types::scheduling_environment::work_order::{operation::Work, WorkOrderActivity};

#[derive(Clone, Default)]
pub struct OperationalParameters {
    pub work_order_parameters: HashMap<WorkOrderActivity, OperationalParameter>,
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
    ) -> Self {
        let combined_time = (work + _preparation).in_seconds();
        let operation_time_delta = TimeDelta::new(combined_time as i64, 0).unwrap();
        assert_ne!(work.to_f64(), 0.0);
        assert!(!operation_time_delta.is_zero());
        assert_eq!(combined_time, work.in_seconds() + _preparation.in_seconds());
        Self {
            work,
            _preparation,
            operation_time_delta,
            // start_window,
            // end_window,
            // delegated,
            // marginal_fitness,
        }
    }
}
