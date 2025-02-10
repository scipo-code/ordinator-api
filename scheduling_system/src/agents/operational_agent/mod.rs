pub mod algorithm;
pub mod assert_functions;
pub mod message_handlers;

use algorithm::operational_parameter::OperationalParameter;
use anyhow::Result;

use rand::{rngs::StdRng, SeedableRng};
use shared_types::operational::{
    OperationalConfiguration, OperationalRequestMessage, OperationalResponseMessage,
};
use shared_types::scheduling_environment::work_order::{operation::Work, WorkOrderActivity};

use shared_types::scheduling_environment::work_order::operation::Operation;

use self::algorithm::OperationalAlgorithm;

use super::Agent;

impl Agent<OperationalAlgorithm, OperationalRequestMessage, OperationalResponseMessage> {
    pub fn create_operational_parameter(
        &mut self,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<()> {
        let scheduling_environment = self.scheduling_environment.lock().unwrap();

        let operation: &Operation = scheduling_environment.operation(work_order_activity);

        assert!(
            operation.work_remaining() > &Some(Work::from(0.0))
                || self
                    .algorithm
                    .loaded_shared_solution
                    .supervisor
                    .operational_state_machine
                    .get(&(self.agent_id.clone(), *work_order_activity))
                    .unwrap()
                    .is_done()
        );

        // TODO: move this around
        let operational_parameter = OperationalParameter::new(
            operation.work_remaining().unwrap(),
            operation.operation_analytic.preparation_time,
        );

        self.algorithm
            .insert_operational_parameter(*work_order_activity, operational_parameter);

        self.algorithm
            .history_of_dropped_operational_parameters
            .insert(*work_order_activity);

        Ok(())
    }
}

pub struct OperationalOptions {
    number_of_activities: usize,
    rng: StdRng,
}

impl Default for OperationalOptions {
    fn default() -> Self {
        Self {
            number_of_activities: 50,
            rng: StdRng::from_os_rng(),
        }
    }
}
