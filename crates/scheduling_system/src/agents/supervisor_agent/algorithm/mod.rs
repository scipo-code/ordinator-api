pub mod delegate;
pub mod supervisor_parameters;
pub mod supervisor_solution;

use std::{collections::HashSet, sync::Arc};

use anyhow::{Context, Result};
use delegate::Delegate;
use rand::seq::IndexedRandom;
use shared_types::scheduling_environment::work_order::{
    operation::ActivityNumber, WorkOrderNumber,
};

use supervisor_parameters::SupervisorParameters;
#[allow(unused_imports)]
use tracing::{event, Level};

use crate::agents::{
    operational_agent::algorithm::operational_solution::MarginalFitness,
    traits::{ActorBasedLargeNeighborhoodSearch, ObjectiveValueType},
    Algorithm, SupervisorSolution,
};

use super::SupervisorOptions;

impl Algorithm<SupervisorSolution, SupervisorParameters, ()> {
    pub fn unschedule_specific_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
    ) -> Result<()> {
        self.solution
            .turn_work_order_into_delegate_assess(work_order_number);
        Ok(())
    }
}

impl ActorBasedLargeNeighborhoodSearch for Algorithm<SupervisorSolution, SupervisorParameters, ()> {
    type Options = SupervisorOptions;

    fn make_atomic_pointer_swap(&self) {
        // Performance enhancements:
        // * COW:
        //      #[derive(Clone)]
        //      struct SharedSolution<'a> {
        //          tactical: Cow<'a, TacticalSolution>,
        //          // other fields...
        //      }
        //
        // * Reuse the old SharedSolution, cloning only the fields that are needed.
        //     let shared_solution = Arc::new(SharedSolution {
        //             tactical: self.tactical_solution.clone(),
        //             // Copy over other fields without cloning
        //             ..(**old).clone()
        //         });
        self.arc_swap_shared_solution.0.rcu(|old| {
            let mut shared_solution = (**old).clone();
            shared_solution.supervisor = self.solution.clone();
            Arc::new(shared_solution)
        });
    }

    fn calculate_objective_value(&mut self) -> Result<ObjectiveValueType<Self::ObjectiveValue>> {
        let assigned_woas = &self.solution.number_of_assigned_work_orders();

        let all_woas: HashSet<_> = self.solution.get_work_order_activities();

        assert!(is_assigned_part_of_all(assigned_woas, &all_woas));

        let mut intermediate = assigned_woas.len() as f64 / all_woas.len() as f64;
        if intermediate.is_nan() {
            intermediate = 0.0;
        };

        let objective_value = (intermediate * 1000.0) as u64;

        if self.solution.objective_value < objective_value {
            event!(
                Level::INFO,
                supervisor_objective_value_better = objective_value
            );
            Ok(ObjectiveValueType::Better(objective_value))
        } else {
            event!(
                Level::INFO,
                supervisor_objective_value_worse = objective_value
            );
            Ok(ObjectiveValueType::Worse)
        }
    }

    fn schedule(&mut self) -> Result<()> {
        for work_order_activity in &self.solution.get_work_order_activities() {
            let number = self
                .parameters
                .supervisor_work_orders
                .get(&work_order_activity.0)
                .and_then(|activities| activities.get(&work_order_activity.1))
                .expect("The SupervisorParameter should always be available")
                .number;

            let operational_solutions = &self.loaded_shared_solution.operational;

            let mut operational_status_by_work_order_activity =
                self.solution.operational_status_by_work_order_activity(
                    work_order_activity,
                    operational_solutions,
                );

            operational_status_by_work_order_activity
                .retain(|(_, _, mar_fit)| matches!(mar_fit, MarginalFitness::Scheduled(_)));

            operational_status_by_work_order_activity.sort_by_cached_key(
                |(_agent_id, _, mar_fit)| match mar_fit {
                    MarginalFitness::Scheduled(auxillary_operational_objective) => {
                        auxillary_operational_objective
                    }
                    MarginalFitness::None => panic!(),
                },
            );

            if !operational_status_by_work_order_activity.is_empty() {

                // dbg!(operational_status_by_work_order_activity.len());
            };

            let number_of_assigned = operational_status_by_work_order_activity
                .iter()
                .filter(|(_, delegate, _)| *delegate == Delegate::Assign)
                .count() as u64;

            let mut remaining_to_assign = number - number_of_assigned;

            event!(Level::DEBUG, remaining_to_assign = ?remaining_to_assign);
            for (agent_id, delegate_status, _marginal_fitness) in
                &mut operational_status_by_work_order_activity
            {
                if *delegate_status != Delegate::Assess {
                    continue;
                }

                if remaining_to_assign >= 1 {
                    remaining_to_assign -= 1;
                    self.solution
                        .operational_state_machine
                        .get_mut(&(agent_id.clone(), *work_order_activity)).expect("This value should always be present. Check the generation of keys and values if this fails")
                        .state_change_to_assign();
                } else {
                    if *delegate_status == Delegate::Assign {
                        continue;
                    }
                    self.solution
                        .operational_state_machine
                        .get_mut(&(agent_id.clone(), *work_order_activity)).expect("This value should always be present. Check the generation of keys and values if this fails")
                        .state_change_to_unassign();
                }
            }
        }
        Ok(())
    }

    fn unschedule(&mut self, supervisor_options: &mut Self::Options) -> Result<()> {
        let work_order_numbers = self.solution.get_assigned_and_unassigned_work_orders();

        let sampled_work_order_numbers = work_order_numbers
            .choose_multiple(
                &mut supervisor_options.rng,
                supervisor_options.number_of_unassigned_work_orders,
            )
            .collect::<Vec<_>>()
            .clone();

        for work_order_number in sampled_work_order_numbers {
            self.unschedule_specific_work_order(*work_order_number)
                .with_context(|| {
                    format!(
                        "Could not unschedule work_order_number: {:?}",
                        work_order_number
                    )
                })?;
        }
        Ok(())
        // self.algorithm.operational_state.assert_that_operational_state_machine_is_different_from_saved_operational_state_machine(&old_state).unwrap();
    }

    fn incorporate_shared_state(&mut self) -> Result<bool> {
        // List current activities in the `SupervisorAgent`
        let current_activities = self
            .solution
            .operational_state_machine
            .keys()
            .map(|(_, woa)| woa.0)
            .collect::<HashSet<WorkOrderNumber>>();

        // Filter for Strategic scheduled work orders that are inside of the `SupervisorAlgorithm.parameters.strategic_periods`.
        // This can be made cleaner! Much cleaner,
        let strategic_activities_in_supervisor_period = self
            .loaded_shared_solution
            .strategic
            .strategic_scheduled_work_orders
            .iter()
            .filter_map(|(won, opt_str_per)| {
                opt_str_per.as_ref().and_then(|per| {
                    self.parameters
                        .supervisor_periods
                        .contains(per)
                        .then_some((won, per))
                })
            });

        // Select only those that are not part of the `SupervisorAgent` already
        let incoming_activities = strategic_activities_in_supervisor_period
            .clone()
            .filter(|(won, _)| !current_activities.contains(won));

        // Insert all the incoming activities as Delegate::default() for each `OperationalAgent` that
        // has the required skill, `enum Resources`
        for (work_order_number, _) in incoming_activities {
            for activity_number in (self
                .parameters
                .supervisor_work_orders
                .get(work_order_number)
                .context("Missing WorkOrder Parameter in Supervisor")?)
            .keys()
            {
                for operational_id in self.loaded_shared_solution.operational.keys() {
                    let supervisor_parameter_resource = &self
                        .parameters
                        .supervisor_work_orders
                        .get(work_order_number)
                        .context("Missing WorkOrder Parameter in Supervisor")?
                        .get(activity_number)
                        .context("Missing Activity Parameter in Supervisor")?
                        .resource;

                    if operational_id.1.contains(supervisor_parameter_resource) {
                        let work_order_activity = (*work_order_number, *activity_number);
                        let operational_state = (operational_id.clone(), work_order_activity);

                        self.solution
                            .operational_state_machine
                            .insert(operational_state, Delegate::default());
                    }
                }
            }
        }

        let strategic_activities_hash_set = strategic_activities_in_supervisor_period
            .map(|e| e.0)
            .cloned()
            .collect::<HashSet<_>>();

        self.solution
            .operational_state_machine
            .retain(|id_woa, _| strategic_activities_hash_set.contains(&id_woa.1 .0));

        Ok(true)
    }
}

fn is_assigned_part_of_all(
    assigned_woas: &HashSet<(WorkOrderNumber, ActivityNumber)>,
    all_woas: &HashSet<(WorkOrderNumber, ActivityNumber)>,
) -> bool {
    assigned_woas
        .iter()
        .all(|(wo, ac)| all_woas.contains(&(*wo, *ac)))
}
