pub mod delegate;
pub mod supervisor_parameters;
pub mod supervisor_solution;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::{Context, Result};
use arc_swap::Guard;
use delegate::Delegate;
use rand::seq::IndexedRandom;
use shared_types::{
    scheduling_environment::{
        time_environment::period::Period,
        work_order::{
            operation::{operation_info::NumberOfPeople, ActivityNumber, Operation},
            WorkOrderActivity, WorkOrderNumber,
        },
        worker_environment::resources::Resources,
    },
    supervisor::{SupervisorObjectiveValue, SupervisorRequestMessage, SupervisorResponseMessage},
};

#[allow(unused_imports)]
use tracing::{event, Level};

use crate::agents::{
    operational_agent::algorithm::operational_solution::MarginalFitness,
    traits::{ActorBasedLargeNeighborhoodSearch, ObjectiveValueType},
    ArcSwapSharedSolution, SharedSolution, SupervisorSolution,
};

use super::SupervisorOptions;

pub struct SupervisorAlgorithm {
    pub objective_value: SupervisorObjectiveValue,
    pub resources: Vec<Resources>,
    pub supervisor_parameters: SupervisorParameters,
    pub supervisor_solution: SupervisorSolution,
    arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
    pub loaded_shared_solution: Guard<Arc<SharedSolution>>,
}

pub struct SupervisorParameters {
    pub supervisor_work_orders:
        HashMap<WorkOrderNumber, HashMap<ActivityNumber, SupervisorParameter>>,
    pub supervisor_periods: Vec<Period>,
}

impl SupervisorParameters {
    pub fn new(supervisor_periods: Vec<Period>) -> Self {
        Self {
            supervisor_work_orders: HashMap::new(),
            supervisor_periods,
        }
    }

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

impl SupervisorAlgorithm {
    pub fn new(
        resources: Vec<Resources>,
        arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
        supervisor_periods: &[Period],
    ) -> Self {
        let loaded_shared_solution = arc_swap_shared_solution.0.load();

        Self {
            objective_value: SupervisorObjectiveValue::default(),
            resources,
            supervisor_parameters: SupervisorParameters::new(supervisor_periods.to_vec()),
            supervisor_solution: SupervisorSolution::default(),
            arc_swap_shared_solution,
            loaded_shared_solution,
        }
    }
    fn unschedule_specific_work_order(&mut self, work_order_number: WorkOrderNumber) -> Result<()> {
        self.supervisor_solution
            .turn_work_order_into_delegate_assess(work_order_number);
        Ok(())
    }
}

impl ActorBasedLargeNeighborhoodSearch for SupervisorAlgorithm {
    type MessageRequest = SupervisorRequestMessage;
    type MessageResponse = SupervisorResponseMessage;
    type Options = SupervisorOptions;
    type Solution = SupervisorSolution;

    fn clone_algorithm_solution(&self) -> Self::Solution {
        self.supervisor_solution.clone()
    }

    fn swap_solution(&mut self, solution: Self::Solution) {
        self.supervisor_solution = solution;
    }

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
            shared_solution.supervisor = self.supervisor_solution.clone();
            Arc::new(shared_solution)
        });
    }

    fn load_shared_solution(&mut self) {
        self.loaded_shared_solution = self.arc_swap_shared_solution.0.load();
    }

    fn calculate_objective_value(&mut self) -> Result<ObjectiveValueType> {
        let assigned_woas = &self.supervisor_solution.number_of_assigned_work_orders();

        let all_woas: HashSet<_> = self.supervisor_solution.get_work_order_activities();

        assert!(is_assigned_part_of_all(assigned_woas, &all_woas));

        let mut intermediate = assigned_woas.len() as f64 / all_woas.len() as f64;
        if intermediate.is_nan() {
            intermediate = 0.0;
        };

        let objective_value = (intermediate * 1000.0) as u64;

        self.objective_value = objective_value;
        if self.objective_value < objective_value {
            event!(Level::INFO, supervisor_objective_value = objective_value);
            Ok(ObjectiveValueType::Better)
        } else {
            event!(Level::INFO, supervisor_objective_value = objective_value);
            Ok(ObjectiveValueType::Worse)
        }
    }

    fn schedule(&mut self) -> Result<()> {
        'next_work_order_activity: for work_order_activity in
            &self.supervisor_solution.get_work_order_activities()
        {
            let number = self
                .supervisor_parameters
                .supervisor_work_orders
                .get(&work_order_activity.0)
                .and_then(|activities| activities.get(&work_order_activity.1))
                .expect("The SupervisorParameter should always be available")
                .number;

            let operational_solutions = &self.loaded_shared_solution.operational;

            let mut operational_status_by_work_order_activity = self
                .supervisor_solution
                .operational_status_by_work_order_activity(
                    work_order_activity,
                    operational_solutions,
                );

            // dbg!(operational);

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
                    self.supervisor_solution
                        .operational_state_machine
                        .get_mut(&(agent_id.clone(), *work_order_activity)).expect("This value should always be present. Check the generation of keys and values if this fails")
                        .state_change_to_assign();
                } else {
                    if *delegate_status == Delegate::Assign {
                        continue;
                    }
                    self.supervisor_solution
                        .operational_state_machine
                        .get_mut(&(agent_id.clone(), *work_order_activity)).expect("This value should always be present. Check the generation of keys and values if this fails")
                        .state_change_to_unassign();
                }
            }
        }
        Ok(())
    }

    fn unschedule(&mut self, supervisor_options: &mut Self::Options) -> Result<()> {
        let work_order_numbers = self
            .supervisor_solution
            .get_assigned_and_unassigned_work_orders();

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

    // WARNING
    // THIS IS A GOOD WAY OF FILTERING THE WORK ORDERS
    // fn remove_leaving_work_order_activities(
    //     &mut self,
    //     entering_work_orders_from_strategic: &HashSet<WorkOrderNumber>,
    // ) {
    //     let supervisor_work_orders: HashSet<WorkOrderNumber> = self
    //         .operational_state_machine
    //         .iter()
    //         .map(|((_, woa), _)| woa.0)
    //         .collect();

    //     let leaving: HashSet<_> = supervisor_work_orders
    //         .difference(entering_work_orders_from_strategic)
    //         .collect();

    //     self.operational_state_machine
    //         .retain(|(_, woa), _| !leaving.contains(&woa.0));
    // }
    // WARNING
    fn update_based_on_shared_solution(&mut self) -> Result<()> {
        // List current activities in the `SupervisorAgent`
        let current_activities = self
            .supervisor_solution
            .operational_state_machine
            .keys()
            .map(|(_, woa)| woa.0)
            .collect::<HashSet<_>>();

        // Filter for Strategic scheduled work orders that are inside of the `SupervisorAlgorithm.parameters.strategic_periods`
        let activities_in_supervisor_period = self
            .loaded_shared_solution
            .strategic
            .strategic_scheduled_work_orders
            .iter()
            .filter_map(|(won, opt_str_per)| {
                opt_str_per.as_ref().and_then(|per| {
                    self.supervisor_parameters
                        .supervisor_periods
                        .contains(per)
                        .then_some((won, per))
                })
            });

        // Select only those that are not part of the `SupervisorAgent` already
        let incoming_activities = activities_in_supervisor_period
            .clone()
            .filter(|(won, _)| current_activities.contains(won));

        // Insert all the incoming activities as Delegate::default() for each `OperationalAgent` that
        // has the required skill, `enum Resources`
        for (work_order_number, _) in incoming_activities {
            for activity_number in (self
                .supervisor_parameters
                .supervisor_work_orders
                .get(work_order_number)
                .context("Missing WorkOrder Parameter in Supervisor")?)
            .keys()
            {
                for operational_id in self.loaded_shared_solution.operational.keys() {
                    if operational_id.1.contains(
                        &self
                            .supervisor_parameters
                            .supervisor_work_orders
                            .get(work_order_number)
                            .context("Missing WorkOrder Parameter in Supervisor")?
                            .get(activity_number)
                            .context("Missing Activity Parameter in Supervisor")?
                            .resource,
                    ) {
                        self.supervisor_solution.operational_state_machine.insert(
                            (
                                operational_id.clone(),
                                (*work_order_number, *activity_number),
                            ),
                            Delegate::default(),
                        );
                    }
                }
            }
        }

        // Now we have to remove the the activities that are not part of the SharedSolution::strategic
        for (strategic_work_order_number, _) in activities_in_supervisor_period {
            self.supervisor_solution
                .operational_state_machine
                .retain(|id_woa, _| id_woa.1 .0 == *strategic_work_order_number)
        }

        Ok(())
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
