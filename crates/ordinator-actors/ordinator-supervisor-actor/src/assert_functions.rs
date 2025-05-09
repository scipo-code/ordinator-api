use std::collections::HashSet;

use anyhow::Result;
use anyhow::bail;
use ordinator_actor_core::Actor;
use ordinator_orchestrator_actor_traits::StrategicInterface;
use ordinator_orchestrator_actor_traits::SystemSolutionTrait;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use tracing::Level;
use tracing::event;

use crate::algorithm::SupervisorAlgorithm;
use crate::algorithm::supervisor_solution::SupervisorSolution;

#[allow(dead_code)]
pub trait SupervisorAssertions {
    fn test_symmetric_difference_between_tactical_operations_and_operational_state_machine(
        &self,
    ) -> Result<()>;
    fn assert_operational_state_machine_woas_is_subset_of_tactical_shared_solution(
        &self,
    ) -> Result<()>;
}

impl<MessageRequest, MessageResponse, Ss> SupervisorAssertions
    for Actor<MessageRequest, MessageResponse, SupervisorAlgorithm<Ss>>
where
    Ss: SystemSolutionTrait<Supervisor = SupervisorSolution>,
{
    fn test_symmetric_difference_between_tactical_operations_and_operational_state_machine(
        &self,
    ) -> Result<()> {
        let tactical_operation_woas: HashSet<WorkOrderNumber> = self
            .algorithm
            .loaded_shared_solution
            .strategic()
            .supervisor_tasks(&self.algorithm.parameters.supervisor_periods)
            .iter()
            .map(|f| *f.0)
            .collect();

        let operational_state_woas: HashSet<WorkOrderNumber> = self
            .algorithm
            .solution
            .get_iter()
            .map(|(woa, _)| woa.1.0)
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
            bail!(
                "If the symmetric difference is empty it means that there are state inconsistencies"
            );
        }
        Ok(())
    }

    // This assertion tests that
    fn assert_operational_state_machine_woas_is_subset_of_tactical_shared_solution(
        &self,
    ) -> Result<()> {
        let strategic_work_orders: HashSet<WorkOrderNumber> = self
            .algorithm
            .loaded_shared_solution
            .strategic()
            .supervisor_tasks(&self.algorithm.parameters.supervisor_periods)
            .iter()
            .map(|f| *f.0)
            .collect();

        let operational_state_work_order_activities: HashSet<WorkOrderNumber> = self
            .algorithm
            .solution
            .get_iter()
            .map(|(woa, _)| woa.1.0)
            .collect();

        if !operational_state_work_order_activities.is_subset(&strategic_work_orders) {
            event!(
                Level::ERROR,
                operational_difference_with_tactical_operations = ?operational_state_work_order_activities
                    .difference(&strategic_work_orders)
                    .cloned()
                    .collect::<HashSet<_>>()
            );
            bail!(
                "The tactical_operations should always hold all the work_order_activities of the operational_state_machine"
            );
        }
        Ok(())
    }
}
