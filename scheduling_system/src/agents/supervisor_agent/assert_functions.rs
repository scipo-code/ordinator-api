use std::collections::HashSet;

use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use tracing::{event, Level};

use super::SupervisorAgent;

#[allow(dead_code)]
pub trait SupervisorAssertions {
    fn test_symmetric_difference_between_tactical_operations_and_operational_state_machine(&self);
    fn assert_that_operational_state_machine_woas_are_a_subset_of_tactical_operations(&self);
}

impl SupervisorAssertions for SupervisorAgent {
    fn test_symmetric_difference_between_tactical_operations_and_operational_state_machine(&self) {
        let tactical_operation_woas: HashSet<WorkOrderNumber> = self
            .supervisor_algorithm
            .loaded_shared_solution
            .strategic
            .supervisor_activities(
                &self
                    .supervisor_algorithm
                    .supervisor_parameters
                    .supervisor_periods,
            );

        let operational_state_woas: HashSet<WorkOrderNumber> = self
            .supervisor_algorithm
            .operational_state_machine
            .get_iter()
            .map(|(woa, _)| woa.1 .0)
            .collect();
        // What would it mean to schedule these work
        let symmetric_difference = tactical_operation_woas
            .symmetric_difference(&operational_state_woas)
            .cloned()
            .collect::<HashSet<WorkOrderNumber>>();

        if !symmetric_difference.is_empty() {
            event!(Level::ERROR,
                non_corresponding_work_order_activities = ? symmetric_difference,
                in_the_tactical_operations = ?symmetric_difference.intersection(&tactical_operation_woas),
                in_the_operational_state_woas = ?symmetric_difference.intersection(&operational_state_woas),
            );
            panic!();
        }
    }
    fn assert_that_operational_state_machine_woas_are_a_subset_of_tactical_operations(&self) {
        let tactical_operation_woas: HashSet<WorkOrderNumber> = self
            .supervisor_algorithm
            .loaded_shared_solution
            .strategic
            .supervisor_activities(
                &self
                    .supervisor_algorithm
                    .supervisor_parameters
                    .supervisor_periods,
            );

        let operational_state_woas: HashSet<WorkOrderNumber> = self
            .supervisor_algorithm
            .operational_state_machine
            .get_iter()
            .map(|(woa, _)| woa.1 .0)
            .collect();

        if !operational_state_woas.is_subset(&tactical_operation_woas) {
            event!(
                Level::ERROR,
                operational_difference_with_tactical_operations = ?operational_state_woas
                    .difference(&tactical_operation_woas)
                    .cloned()
                    .collect::<HashSet<_>>()
            );
            panic!("The tactical_operations should always hold all the work_order_activities of the operational_state_machine");
        }
    }
}
