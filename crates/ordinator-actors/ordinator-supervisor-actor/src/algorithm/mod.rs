pub mod assert_functions;
pub mod supervisor_interface;
pub mod supervisor_parameters;
pub mod supervisor_solution;

use std::collections::HashSet;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::traits::AbLNSUtils;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_actor_core::traits::ObjectiveValueType;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::StrategicInterface;
use ordinator_orchestrator_actor_traits::SwapSolution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::delegate::Delegate;
use ordinator_orchestrator_actor_traits::marginal_fitness::MarginalFitness;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::worker_environment::SupervisorOptions;
use rand::rng;
use rand::seq::IndexedRandom;
use supervisor_parameters::SupervisorParameters;
use supervisor_solution::SupervisorSolution;
#[allow(unused_imports)]
use tracing::Level;
#[allow(unused_imports)]
use tracing::event;

pub struct SupervisorAlgorithm<Ss>(Algorithm<SupervisorSolution, SupervisorParameters, (), Ss>)
where
    Ss: SystemSolutions;

impl<Ss> SupervisorAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    pub fn unschedule_specific_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
    ) -> Result<()>
    {
        self.solution
            .turn_work_order_into_delegate_assess(work_order_number);
        Ok(())
    }
}

impl<Ss> ActorBasedLargeNeighborhoodSearch for SupervisorAlgorithm<Ss>
where
    Algorithm<SupervisorSolution, SupervisorParameters, (), Ss>:
        AbLNSUtils<SolutionType = SupervisorSolution>,
    SupervisorSolution: Solution,
    SupervisorParameters: Parameters,
    Ss: SystemSolutions<Supervisor = SupervisorSolution>,
{
    type Algorithm = Algorithm<SupervisorSolution, SupervisorParameters, (), Ss>;
    type Options = SupervisorOptions;

    // I think that we can move this out
    fn make_atomic_pointer_swap(&mut self)
    {
        // Performance enhancements:
        // * COW: #[derive(Clone)] struct SharedSolution<'a> { tactical: Cow<'a,
        //   TacticalSolution>, // other fields... }
        //
        // * Reuse the old SharedSolution, cloning only the fields that are needed. let
        //   shared_solution = Arc::new(SharedSolution { tactical:
        //   self.tactical_solution.clone(), // Copy over other fields without cloning
        //   ..(**old).clone() });
        // NOTE
        // Every actor will have to specify how to make this work
        // on its own. There is no other way of doing it I think.
        //
        // Yes you have to make it like that. Also I can sense that
        // you will have to make the code work more efficiently with the
        // `SchedulingEnvironment` in the future.
        //
        // I do not see what other way we could make this work. The best
        // approach would possibly be
        self.arc_swap_shared_solution.rcu(|old| {
            let mut system_solutions = (**old).clone();
            // You have to invert the dependency here.
            // I cannot see how to make this function in a correct
            // manner. The best possible way here is to make the system work with
            // the required,
            SwapSolution::swap(&self.id, self.solution.clone(), &mut system_solutions);
            // <SupervisorSolution as SwapSolution>::swap(self.id, self.solution,
            // system_solutions) swap(self.id, self.solution.clone(),
            // shared_solution)
            system_solutions.supervisor_swap(&self.id, self.solution.clone());
            Arc::new(system_solutions)
        });
    }

    fn calculate_objective_value(
        &mut self,
    ) -> Result<
        ObjectiveValueType<
            <<Self::Algorithm as AbLNSUtils>::SolutionType as Solution>::ObjectiveValue,
        >,
    >
    {
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

    fn schedule(&mut self) -> Result<()>
    {
        for work_order_activity in &self.solution.get_work_order_activities() {
            let number = self
                .parameters
                .supervisor_work_orders
                .get(&work_order_activity.0)
                .and_then(|activities| activities.get(&work_order_activity.1))
                .expect("The SupervisorParameter should always be available")
                .number;

            // And this comes in as a close second.
            let mut operational_status_by_work_order_activity =
                self.solution.operational_status_by_work_order_activity(
                    work_order_activity,
                    &self.loaded_shared_solution,
                )?;

            operational_status_by_work_order_activity
                .retain(|(_, _, mar_fit)| matches!(mar_fit, MarginalFitness::Scheduled(_)));

            operational_status_by_work_order_activity.sort_by_key(|(_agent_id, _, mar_fit)| {
                match mar_fit {
                    MarginalFitness::Scheduled(auxillary_operational_objective) => {
                        *auxillary_operational_objective
                    }
                    MarginalFitness::None => panic!(),
                }
            });

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
                operational_status_by_work_order_activity.clone()
            {
                if delegate_status != Delegate::Assess {
                    continue;
                }

                let solution =
                    self.solution
                        .operational_state_machine
                        .get_mut(&(agent_id.clone(), *work_order_activity)).expect("This value should always be present. Check the generation of keys and values if this fails");

                if remaining_to_assign >= 1 {
                    remaining_to_assign -= 1;
                    solution.state_change_to_assign();
                } else {
                    if delegate_status == Delegate::Assign {
                        continue;
                    }
                    solution.state_change_to_unassign();
                }
            }
        }
        Ok(())
    }

    fn unschedule(&mut self) -> Result<()>
    {
        let mut rng = rng();
        let work_order_numbers = self.solution.get_assigned_and_unassigned_work_orders();

        let sampled_work_order_numbers = work_order_numbers
            .choose_multiple(
                &mut rng,
                self.parameters.options.number_of_unassigned_work_orders,
            )
            .collect::<Vec<_>>()
            .clone();

        for work_order_number in sampled_work_order_numbers {
            self.unschedule_specific_work_order(*work_order_number)
                .with_context(|| {
                    format!("Could not unschedule work_order_number: {work_order_number:?}")
                })?;
        }
        Ok(())
        // self.algorithm.operational_state.
        // assert_that_operational_state_machine_is_different_from_saved_operational_state_machine(&
        // old_state).unwrap();
    }

    fn incorporate_shared_state(&mut self) -> Result<bool>
    {
        // List current activities in the `SupervisorAgent`
        let current_activities = self
            .solution
            .operational_state_machine
            .keys()
            .map(|(_, woa)| woa.0)
            .collect::<HashSet<WorkOrderNumber>>();

        // Filter for Strategic scheduled work orders that are inside of the
        // `SupervisorAlgorithm.parameters.strategic_periods`. This can be made
        // cleaner! Much cleaner,
        let strategic_activities_in_supervisor_period = self
            .loaded_shared_solution
            .strategic()?
            .supervisor_tasks(&self.parameters.supervisor_periods);

        // Select only those that are not part of the `SupervisorAgent` already
        let incoming_activities = strategic_activities_in_supervisor_period
            .iter()
            .filter(|(won, _)| !current_activities.contains(won));

        // Insert all the incoming activities as Delegate::default() for each
        // `OperationalAgent` that has the required skill, `enum Resources`
        // QUESTION
        // Why does this happen here? I do not really know why and that is an
        // issue. You should find out now.
        //
        // TODO [ ]
        // determine exactly how to fix this.
        let work_order_parameters = self.parameters.supervisor_work_orders.clone();
        let all_operational_actors = self.loaded_shared_solution.all_operational().clone();

        for (work_order_number, _) in incoming_activities {
            let activity_number = work_order_parameters
                .get(work_order_number)
                .context("Missing WorkOrder Parameter in Supervisor")?
                .keys()
                .cloned();

            for activity_number in activity_number {
                for operational_id in &all_operational_actors {
                    let supervisor_parameter_resource = &self
                        .parameters
                        .supervisor_work_orders
                        .get(work_order_number)
                        .context("Missing WorkOrder Parameter in Supervisor")?
                        .get(&activity_number)
                        .context("Missing Activity Parameter in Supervisor")?
                        .resource;

                    if operational_id.1.contains(supervisor_parameter_resource) {
                        let work_order_activity = (*work_order_number, activity_number);
                        let operational_state = ((*operational_id).clone(), work_order_activity);

                        self.solution
                            .operational_state_machine
                            .insert(operational_state, Delegate::default());
                    }
                }
            }
        }

        let strategic_activities_hash_set = strategic_activities_in_supervisor_period
            .iter()
            .map(|e| e.0)
            .cloned()
            .collect::<HashSet<_>>();

        self.solution
            .operational_state_machine
            .retain(|id_woa, _| strategic_activities_hash_set.contains(&id_woa.1.0));

        Ok(true)
    }

    fn algorithm_util_methods(&mut self) -> &mut Self::Algorithm
    {
        &mut self.0
    }
}

fn is_assigned_part_of_all(
    assigned_woas: &HashSet<(WorkOrderNumber, ActivityNumber)>,
    all_woas: &HashSet<(WorkOrderNumber, ActivityNumber)>,
) -> bool
{
    assigned_woas
        .iter()
        .all(|(wo, ac)| all_woas.contains(&(*wo, *ac)))
}
impl<Ss> Deref for SupervisorAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    type Target = Algorithm<SupervisorSolution, SupervisorParameters, (), Ss>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss> DerefMut for SupervisorAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}
impl<Ss> From<Algorithm<SupervisorSolution, SupervisorParameters, (), Ss>>
    for SupervisorAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    fn from(value: Algorithm<SupervisorSolution, SupervisorParameters, (), Ss>) -> Self
    {
        SupervisorAlgorithm(value)
    }
}
