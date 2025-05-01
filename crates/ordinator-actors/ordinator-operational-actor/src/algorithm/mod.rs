pub mod assert_functions;
pub mod operational_events;
mod operational_interface;
pub mod operational_parameter;
pub mod operational_solution;

use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use anyhow::ensure;
use assert_functions::OperationalAlgorithmAsserts;
use chrono::DateTime;
use chrono::TimeDelta;
use chrono::Utc;
use itertools::Itertools;
use operational_events::OperationalEvents;
use operational_parameter::OperationalParameter;
use operational_parameter::OperationalParameters;
use operational_solution::Assignment;
use operational_solution::OperationalAssignment;
use operational_solution::OperationalFunctions;
use operational_solution::OperationalObjectiveValue;
use operational_solution::OperationalSolution;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::traits::AbLNSUtils;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_actor_core::traits::ActorLinkToSchedulingEnvironment;
use ordinator_actor_core::traits::ObjectiveValueType;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::StrategicInterface;
use ordinator_orchestrator_actor_traits::SupervisorInterface;
use ordinator_orchestrator_actor_traits::SystemSolutionTrait;
use ordinator_orchestrator_actor_traits::TacticalInterface;
use ordinator_orchestrator_actor_traits::delegate::Delegate;
use ordinator_orchestrator_actor_traits::marginal_fitness::MarginalFitness;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::TimeInterval;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::availability::Availability;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use rand::seq::IndexedRandom;
use tracing::Level;
use tracing::event;

use super::OperationalOptions;

pub struct OperationalAlgorithm<Ss>(
    Algorithm<OperationalSolution, OperationalParameters, OperationalNonProductive, Ss>,
)
where
    Ss: SystemSolutionTrait,
    Algorithm<OperationalSolution, OperationalParameters, OperationalNonProductive, Ss>: AbLNSUtils;

#[derive(Clone, Default)]
pub struct OperationalNonProductive(pub Vec<Assignment>);

pub trait OperationalTraitUtils {
    fn determine_next_event_non_productive(
        &mut self,
        current_time: &mut DateTime<Utc>,
        next_operation: Option<OperationalAssignment>,
    ) -> (DateTime<Utc>, OperationalEvents);

    fn determine_time_interval_of_function(
        &mut self,
        next_operation: Option<OperationalAssignment>,
        current_time: &DateTime<Utc>,
        interval: TimeInterval,
    ) -> TimeInterval;

    fn update_marginal_fitness(
        &mut self,
        work_order_activity_previous: (WorkOrderNumber, ActivityNumber),
        time_delta: TimeDelta,
    );
}

// Do you want this on the OperationalAlgorithm?
impl<Ss> OperationalTraitUtils for OperationalAlgorithm<Ss>
where
    Ss: SystemSolutionTrait,
{
    fn determine_next_event_non_productive(
        &mut self,
        current_time: &mut DateTime<Utc>,
        next_operation: Option<OperationalAssignment>,
    ) -> (DateTime<Utc>, OperationalEvents) {
        if self.0.parameters.break_interval.contains(current_time) {
            let time_interval = self.determine_time_interval_of_function(
                next_operation,
                current_time,
                self.0.parameters.break_interval.clone(),
            );
            let new_current_time = *current_time + time_interval.duration();
            (new_current_time, OperationalEvents::Break(time_interval))
        } else if self.0.parameters.off_shift_interval.contains(current_time) {
            let time_interval = self.determine_time_interval_of_function(
                next_operation,
                current_time,
                self.0.parameters.off_shift_interval.clone(),
            );
            let new_current_time = *current_time + time_interval.duration();
            (
                new_current_time,
                OperationalEvents::OffShift(self.0.parameters.off_shift_interval.clone()),
            )
        } else if self.0.parameters.toolbox_interval.contains(current_time) {
            let time_interval = self.determine_time_interval_of_function(
                next_operation,
                current_time,
                self.0.parameters.toolbox_interval.clone(),
            );
            let new_current_time = *current_time + time_interval.duration();
            (
                new_current_time,
                OperationalEvents::Toolbox(self.0.parameters.toolbox_interval.clone()),
            )
        } else {
            let start = *current_time;
            let (time_until_next_event, next_operational_event) =
                self.determine_next_event(current_time);
            let mut new_current_time = *current_time + time_until_next_event;

            if *current_time == new_current_time {
                (
                    new_current_time + next_operational_event.time_delta(),
                    next_operational_event,
                )
            } else {
                if self.0.parameters.availability.finish_date < new_current_time {
                    new_current_time = self.0.parameters.availability.finish_date;
                }
                let time_interval = TimeInterval::new(start.time(), new_current_time.time());
                (
                    new_current_time,
                    OperationalEvents::NonProductiveTime(time_interval),
                )
            }
        }
    }

    // This function makes sure that the created event is adjusted to fit the
    // schedule if there has been any manual intervention in the schedule for
    // the OperationalAgent.
    fn determine_time_interval_of_function(
        &mut self,
        next_operation: Option<OperationalAssignment>,
        current_time: &DateTime<Utc>,
        interval: TimeInterval,
    ) -> TimeInterval {
        // What is this code actually trying to do? I think
        let time_interval: TimeInterval = match next_operation {
            Some(operational_solution) => {
                if operational_solution.start_time().date_naive() == current_time.date_naive() {
                    TimeInterval::new(
                        current_time.time(),
                        interval.end.min(operational_solution.start_time().time()),
                    )
                } else {
                    TimeInterval::new(current_time.time(), interval.end)
                }
            }

            None => TimeInterval::new(current_time.time(), interval.end),
        };
        time_interval
    }

    // This is a problem. What should you do about it? I think that the best thing
    // that you can do is move all this into the `schedule` function and handle
    // it while the code is running. That is probably the best call here. I
    // do not see what other way it could be done in a better way.
    fn update_marginal_fitness(
        &mut self,
        work_order_activity_previous: (WorkOrderNumber, ActivityNumber),
        time_delta: TimeDelta,
    ) {
        let time_delta_usize = time_delta.num_seconds() as u64;

        self.0
            .solution
            .scheduled_work_order_activities
            .iter_mut()
            .find(|oper_sol| oper_sol.0 == work_order_activity_previous)
            .unwrap()
            .1
            .marginal_fitness = MarginalFitness::Scheduled(time_delta_usize);
    }
}

pub enum ContainOrNextOrNone {
    Contain(OperationalAssignment),
    Next(OperationalAssignment),
    None,
}

pub enum Unavailability {
    Beginning,
    End,
}

// FIX
// Some of the methods here should be moved out of the agent. That will be
// crucial. You have one hour to m make this compile again.
// QUESTION
// What should be changed here to make the ABLNS work on the Algorithm again?

impl<Ss> Deref for OperationalAlgorithm<Ss>
where
    Ss: SystemSolutionTrait,
{
    type Target =
        Algorithm<OperationalSolution, OperationalParameters, OperationalNonProductive, Ss>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Ss> DerefMut for OperationalAlgorithm<Ss>
where
    Ss: SystemSolutionTrait,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<Ss> ActorBasedLargeNeighborhoodSearch for OperationalAlgorithm<Ss>
where
    Ss: SystemSolutionTrait<Operational = OperationalSolution>,
    Algorithm<OperationalSolution, OperationalParameters, OperationalNonProductive, Ss>:
        AbLNSUtils<SolutionType = OperationalSolution>,
{
    type Algorithm =
        Algorithm<OperationalSolution, OperationalParameters, OperationalNonProductive, Ss>;
    type Options = OperationalOptions;

    fn incorporate_shared_state(&mut self) -> Result<bool> {
        let operational_shared_solution = self
            .loaded_shared_solution
            .supervisor()
            .delegates_for_agent(&self.id);

        self.solution
            .scheduled_work_order_activities
            // We remain all `OperationalSolution` which are not `Delegate::Drop` where
            // the `Delegate` variant is decided by the
            .retain(|(woa, _)| {
                !operational_shared_solution
                    .get(woa)
                    .unwrap_or_else(|| {
                        assert!(*woa == (WorkOrderNumber(0), 0));
                        &Delegate::Assign
                    })
                    .is_drop()
            });
        Ok(true)
    }

    fn make_atomic_pointer_swap(&mut self) {
        // Performance enhancements:
        // * COW: #[derive(Clone)] struct SharedSolution<'a> { tactical: Cow<'a,
        //   TacticalSolution>, // other fields... }
        //
        // * Reuse the old SharedSolution, cloning only the fields that are needed. let
        //   shared_solution = Arc::new(SharedSolution { tactical:
        //   self.tactical_solution.clone(), // Copy over other fields without cloning
        //   ..(**old).clone() });
        self.arc_swap_shared_solution.rcu(|old| {
            let mut shared_solution = (**old).clone();
            shared_solution.operational_swap(&self.id, self.solution.clone());
            Arc::new(shared_solution)
        });
    }

    // If we are going to implement delta evaluation we should remove this part.
    fn calculate_objective_value(
        &mut self,
        options: &Self::Options,
    ) -> Result<
        ObjectiveValueType<
            <<Self::Algorithm as AbLNSUtils>::SolutionType as Solution>::ObjectiveValue,
        >,
    > {
        let operational_events: Vec<Assignment> = self
            .solution
            .scheduled_work_order_activities
            .iter()
            .flat_map(|(_, os)| os.assignments.iter())
            .cloned()
            .collect();

        event!(Level::DEBUG, operational_events_len = ?operational_events.iter().filter(|val| val.operational_events.is_wrench_time()).collect::<Vec<_>>().len());

        let all_events = operational_events
            .into_iter()
            .chain(self.solution_intermediate.0.clone())
            .sorted_unstable_by_key(|ass| ass.start);

        let mut current_time = self.parameters.availability.start_date;

        let mut wrench_time: TimeDelta = TimeDelta::zero();
        let mut break_time: TimeDelta = TimeDelta::zero();
        let mut off_shift_time: TimeDelta = TimeDelta::zero();
        let mut toolbox_time: TimeDelta = TimeDelta::zero();
        let mut non_productive_time: TimeDelta = TimeDelta::zero();

        let mut prev_fitness: TimeDelta = TimeDelta::zero();
        let mut next_fitness: TimeDelta = TimeDelta::zero();
        let mut first_fitness: bool = true;
        let mut current_work_order_activity: Option<WorkOrderActivity> = None;

        self.assert_no_operation_overlap()
            .context("Operational_events overlap in the operational objective value calculation")?;

        // What is the problem here?
        // If the work order changes you
        for assignment in all_events.clone() {
            match &assignment.operational_events {
                OperationalEvents::WrenchTime((time_interval, work_order_activity)) => {
                    wrench_time += time_interval.duration();
                    current_time += time_interval.duration();
                    current_work_order_activity = match current_work_order_activity {
                        Some(work_order_activity_previous) => {
                            if work_order_activity_previous != *work_order_activity {
                                let marginal_fitness_time_delta = prev_fitness + next_fitness;

                                self.update_marginal_fitness(
                                    work_order_activity_previous,
                                    marginal_fitness_time_delta,
                                );
                                prev_fitness = next_fitness;
                                next_fitness = TimeDelta::zero();
                            }
                            Some(*work_order_activity)
                        }
                        None => {
                            first_fitness = false;
                            Some(*work_order_activity)
                        }
                    }
                }
                OperationalEvents::Break(time_interval) => {
                    break_time += time_interval.duration();
                    current_time += time_interval.duration();
                }
                OperationalEvents::Toolbox(time_interval) => {
                    toolbox_time += time_interval.duration();
                    current_time += time_interval.duration();
                }
                OperationalEvents::OffShift(time_interval) => {
                    off_shift_time += time_interval.duration();
                    current_time += time_interval.duration();
                }
                OperationalEvents::NonProductiveTime(time_interval) => {
                    non_productive_time += time_interval.duration();
                    current_time += time_interval.duration();
                    if first_fitness {
                        prev_fitness += time_interval.duration();
                    } else {
                        next_fitness += time_interval.duration();
                    }
                }
                OperationalEvents::Unavailable(_) => {
                    if !first_fitness {
                        assert!(assignment == all_events.clone().next_back().unwrap());
                        let marginal_fitness_time_delta = prev_fitness + next_fitness;
                        self.update_marginal_fitness(
                            current_work_order_activity
                                .expect("This will happen if there are no work orders scheduled"),
                            marginal_fitness_time_delta,
                        );
                    }
                }
            }
        }

        // assert_eq!(current_time, self.availability.end_date);
        equality_between_time_interval_and_assignments(&all_events.clone().collect::<Vec<_>>());

        assert!(is_assignments_in_bounds(
            &all_events.clone().collect(),
            &self.parameters.availability
        ));

        assert!(no_overlap(&all_events.collect::<Vec<_>>()));

        let total_time =
            wrench_time + break_time + off_shift_time + toolbox_time + non_productive_time;
        assert_eq!(total_time, self.parameters.availability.duration());

        event!(Level::TRACE, wrench_time = ?wrench_time,
        break_time = ?break_time,
        toolbox_time = ?toolbox_time,
        non_productive_time = ?non_productive_time);
        let new_objective_value: OperationalObjectiveValue = (((wrench_time).num_seconds() * 100)
            as u64
            / (wrench_time + break_time + toolbox_time + non_productive_time).num_seconds() as u64)
            .into();

        let old_objective_value = self.solution.objective_value;

        // This should not be set here! This is so disguisting! It is really
        // not the way to make this work! You have to find a different
        // way.
        self.solution.objective_value = new_objective_value.into();
        if self.solution.objective_value > old_objective_value {
            event!(Level::INFO, operational_objective_value_better = ?new_objective_value);
            Ok(ObjectiveValueType::Better(new_objective_value))
        } else {
            event!(Level::INFO, operational_objective_value_worse = ?new_objective_value);
            Ok(ObjectiveValueType::Worse)
        }
    }

    fn schedule(&mut self) -> Result<()> {
        self.solution_intermediate.0.clear();
        // This method should now go into the trait for the supervisor. And its name
        // will provide for the
        // TODO [ ]
        // Move this to the `interface`
        // QUESTION
        // What does this function do?
        // It determines all work order activities that are either `Delegate::Assess` or
        // `Delegate::Assign`
        // it should be named: `task`?
        let work_order_activities = &self
            .loaded_shared_solution
            .supervisor()
            .delegated_tasks(&self.id);

        for work_order_activity in work_order_activities {
            let operational_parameter = match self
                .parameters
                .work_order_parameters
                .get(work_order_activity)
            {
                Some(operational_parameter) => operational_parameter,
                None => continue,
            };

            let start_time = self
                .determine_first_available_start_time(work_order_activity, operational_parameter)
                .with_context(|| format!("{:#?}", work_order_activity))?;

            let assignments = self.determine_wrench_time_assignment(
                *work_order_activity,
                operational_parameter,
                start_time,
            );

            self.solution.try_insert(*work_order_activity, assignments);
        }

        let mut current_time = self.parameters.availability.start_date;

        // Fill the schedule
        loop {
            match self.solution.containing_operational_solution(current_time) {
                ContainOrNextOrNone::Contain(operational_solution) => {
                    current_time = operational_solution.finish_time();
                }
                ContainOrNextOrNone::Next(operational_solution) => {
                    let (new_current_time, operational_event) = self
                        .determine_next_event_non_productive(
                            &mut current_time,
                            Some(operational_solution),
                        );
                    assert!(!operational_event.is_wrench_time());
                    assert_eq!(
                        operational_event.time_delta(),
                        new_current_time - current_time
                    );
                    let assignment =
                        Assignment::new(operational_event, current_time, new_current_time);
                    current_time = new_current_time;
                    self.solution_intermediate.0.push(assignment);
                }
                ContainOrNextOrNone::None => {
                    let (new_current_time, operational_event) =
                        self.determine_next_event_non_productive(&mut current_time, None);
                    assert!(!operational_event.is_wrench_time());
                    assert_eq!(
                        operational_event.time_delta(),
                        new_current_time - current_time
                    );
                    let assignment =
                        Assignment::new(operational_event, current_time, new_current_time);
                    current_time = new_current_time;
                    self.solution_intermediate.0.push(assignment);
                }
            };

            if current_time >= self.parameters.availability.finish_date {
                self.solution_intermediate.0.last_mut().unwrap().finish =
                    self.parameters.availability.finish_date;
                break;
            };
        }

        Ok(())
    }

    fn unschedule(&mut self) -> Result<()> {
        let operational_solutions_len = self.solution.scheduled_work_order_activities.len();

        let operational_solutions_filtered: Vec<WorkOrderActivity> =
            self.solution.scheduled_work_order_activities[1..operational_solutions_len - 1]
                .choose_multiple(
                    &mut self.parameters.options.rng.clone(),
                    self.parameters.options.number_of_removed_activities,
                )
                .map(|operational_solution| operational_solution.0)
                .collect();

        for operational_solution in &operational_solutions_filtered {
            self.unschedule_single_work_order_activity((
                operational_solution.0,
                operational_solution.1,
            ))
            .with_context(|| {
                format!(
                    "{:?} from {:?}\ncould not be unscheduled",
                    operational_solution, &operational_solutions_filtered
                )
            })?
        }
        Ok(())
    }

    fn algorithm_util_methods(&mut self) -> &mut Self::Algorithm {
        &mut self.0
    }

    fn update_based_on_shared_solution(&mut self, options: &Self::Options) -> Result<()> {
        self.algorithm_util_methods().load_shared_solution();

        let state_change = self.incorporate_shared_state()?;

        if state_change {
            self.calculate_objective_value(options)?;
            self.make_atomic_pointer_swap();
        }

        Ok(())
    }

    fn derive_options(configurations: &ActorLinkToSchedulingEnvironment, id: &Id) -> Self::Options {
        Self::Options::from((configurations, id))
    }
}

impl<Ss> OperationalAlgorithm<Ss>
where
    Ss: SystemSolutionTrait,
{
    fn determine_wrench_time_assignment(
        &self,
        work_order_activity: WorkOrderActivity,
        operational_parameter: &OperationalParameter,
        start_time: DateTime<Utc>,
    ) -> Vec<Assignment> {
        assert_ne!(operational_parameter.work, Work::from(0.0));
        assert!(!operational_parameter.operation_time_delta.is_zero());
        let mut assigned_work: Vec<Assignment> = vec![];
        let mut remaining_combined_work = operational_parameter.operation_time_delta;
        let mut current_time = start_time;

        while !remaining_combined_work.is_zero() {
            let next_event = self.determine_next_event(&current_time);

            if next_event.0.is_zero() {
                let finish_time = current_time + next_event.1.time_delta();
                assigned_work.push(Assignment::new(next_event.1, current_time, finish_time));
                current_time = finish_time;
            } else if next_event.0 < remaining_combined_work {
                assigned_work.push(Assignment::new(
                    OperationalEvents::WrenchTime((
                        TimeInterval::new(
                            current_time.time(),
                            (current_time + next_event.0).time(),
                        ),
                        work_order_activity,
                    )),
                    current_time,
                    current_time + next_event.0,
                ));
                current_time += next_event.0;
                remaining_combined_work -= next_event.0;
            } else if next_event.0 >= remaining_combined_work {
                assigned_work.push(Assignment::new(
                    OperationalEvents::WrenchTime((
                        TimeInterval::new(
                            current_time.time(),
                            (current_time + remaining_combined_work).time(),
                        ),
                        work_order_activity,
                    )),
                    current_time,
                    current_time + remaining_combined_work,
                ));
                current_time += next_event.0;
                remaining_combined_work = TimeDelta::zero();
            }
        }
        assert_ne!(assigned_work.len(), 0);
        assigned_work
    }

    fn unschedule_single_work_order_activity(
        &mut self,
        work_order_and_activity_number: WorkOrderActivity,
    ) -> Result<()> {
        ensure!(
            self.solution
                .scheduled_work_order_activities
                .iter()
                .any(|os| os.0 == work_order_and_activity_number)
        );
        dbg!(&self.solution.scheduled_work_order_activities.len());

        self.solution
            .scheduled_work_order_activities
            .retain(|os| os.0 != work_order_and_activity_number);
        dbg!(&self.solution.scheduled_work_order_activities.len());

        ensure!(
            !self
                .solution
                .scheduled_work_order_activities
                .iter()
                .any(|os| os.0 == work_order_and_activity_number)
        );
        Ok(())
    }

    fn determine_next_event(&self, current_time: &DateTime<Utc>) -> (TimeDelta, OperationalEvents) {
        let break_diff = (
            self.parameters.break_interval.start - current_time.time(),
            OperationalEvents::Break(self.parameters.break_interval.clone()),
        );

        let toolbox_diff = (
            self.parameters.toolbox_interval.start - current_time.time(),
            OperationalEvents::Toolbox(self.parameters.toolbox_interval.clone()),
        );

        let off_shift_diff = (
            self.parameters.off_shift_interval.start - current_time.time(),
            OperationalEvents::OffShift(self.parameters.off_shift_interval.clone()),
        );

        [break_diff, toolbox_diff, off_shift_diff]
            .iter()
            .filter(|&diff_event| diff_event.0.num_seconds() >= 0)
            .min_by_key(|&diff_event| diff_event.0.num_seconds())
            .cloned()
            .unwrap()
    }

    fn determine_first_available_start_time(
        &self,
        work_order_activity: &WorkOrderActivity,
        operational_parameter: &OperationalParameter,
    ) -> Result<DateTime<Utc>> {
        // Actually as the guard is always read only does this even make any sense?
        //
        // The issue here is that the components simply are really coupled and what you
        // are wasting you time on is idiocy.
        // Is this a true statement?
        // No! You need to do this to make it scalable. Separate what varies from what
        // that does not.
        // Here we load in the `TacticalSolution` from the `loaded_shared_solution`.
        // This should function to make the code work more seamlessly with the
        // `TacticalSolution`. Are we doing this correctly? I do not think that
        // we are. The thing is that the supervisor can force a work order here and then
        // the Tactical and Strategic Agent has to respect that. That means that
        // initialially this could be None, but we should strive to make this as
        // perfect as possible. If there is a tactical days we should
        // use that. If there is a Strategic period we should use that. If there is none
        // we should check the manual part. The issue here is not that it is not
        // scheduled, the issue is that the entry does not exist. What should
        // you do here?
        // You need to create something that will allow us to make.
        // It cannot be done in this way. You need to simply make the most of
        // the interfaces. You have to understand the tradeoffs.
        // Remember that Option::None always means that a specific actor  not
        // has not scheduled a specific operation.
        // TODO [ ]
        // move this into the trait. This is so difficult to make correctly. I think
        // that the best approach is to... You are exposing the whole of the
        // tactical solution to the operational agent, the is the issue. This is
        // a horrible coding practice and you may be able to see through it now
        // but later on you will not. Only expose what you need. The problem
        // with your current approach is that you cannot control that the `operational`
        // actor does not loop over the `tactical` state and simply insert everything
        // that he wants.
        //
        // This is the fundamental issue that you are trying to solve here. That the
        // access between algorithms should be defined by a contract clearly.
        // Remember a master programmer builds programs that cannot fail.
        // What exactly is this function trying to do?
        let tactical_days_option = self
            .loaded_shared_solution
            // Swap the solution in the `ArcSwap`
            .tactical()
            // FIX [ ]
            // Rename this! It is basically a way of sharing what the
            // `tactical` actor needs to do his scheduling.
            .start_and_finish_dates(work_order_activity);

        // .expect("This should always be present. If this occurs you should check the
        // initialization. The implementation is that the tactical and strategic
        // algorithm always provide a key for each WorkOrderNumber");
        let strategic_period_option = self
            .loaded_shared_solution
            .strategic()
            .scheduled_task(&work_order_activity.0);

        // .expect("This should always be present. If this occurs you should check the
        // initialization. The implementation is that the tactical and strategic
        // algorithm always provide a key for each WorkOrderNumber");

        // This should also be reformulated. The key to ask yourself here is "What
        // exactly is it that you need?" You should not include anything else
        // into the scope here. For a given work order and corresponding activity
        // when can we start? This is the information that the operational actor
        // needs to know to fullfill his duty. This is a crucial insight.
        let (start_window, end_window) = match (strategic_period_option, tactical_days_option) {
            // What is actually happening here?
            (None, None) => (
                &self.parameters.availability.start_date,
                &self.parameters.availability.finish_date,
            ),
            (_, Some(d)) => d,
            (Some(Some(period)), _) => (period.start_date(), period.end_date()),

            _ => bail!(
                "This means that there is no state in either the Tactical or the Strategic agent"
            ),
        };

        for operational_solution in self.solution.scheduled_work_order_activities.windows(2) {
            let start_of_availability = {
                let mut current_time = operational_solution[0].1.assignments.last().unwrap().finish;

                if current_time < *start_window {
                    current_time = *start_window;
                }

                let current_time_option = self.update_current_time_based_on_event(current_time);

                current_time = match current_time_option {
                    Some(new_current_time) => new_current_time,
                    None => current_time,
                };

                loop {
                    let (time_to_next_event, next_event) = self.determine_next_event(&current_time);

                    if time_to_next_event.is_zero() {
                        current_time += next_event.time_delta();
                    } else {
                        break current_time;
                    }
                }
            };

            let end_of_availability = operational_solution[1].1.assignments.first().unwrap().start;

            if (*end_window).min(end_of_availability) - (*start_window).max(start_of_availability)
                > operational_parameter.operation_time_delta
            {
                return Ok(*start_window.max(&start_of_availability));
            }
        }

        let mut current_time = *start_window;

        let current_time_option = self.update_current_time_based_on_event(current_time);

        current_time = match current_time_option {
            Some(new_current_time) => new_current_time,
            None => current_time,
        };

        loop {
            let (time_to_next_event, next_event) = self.determine_next_event(&current_time);

            if time_to_next_event.is_zero() {
                current_time += next_event.time_delta();
            } else {
                break Ok(current_time);
            }
        }
    }

    fn update_current_time_based_on_event(
        &self,
        mut current_time: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        if self.parameters.off_shift_interval.contains(&current_time) {
            let off_shift_interval_end = self.parameters.off_shift_interval.end;
            if off_shift_interval_end < current_time.time() {
                current_time = current_time.with_time(off_shift_interval_end).unwrap();
                current_time += TimeDelta::days(1);
                Some(current_time)
            } else {
                current_time = current_time.with_time(off_shift_interval_end).unwrap();
                Some(current_time)
            }
        } else if self.parameters.break_interval.contains(&current_time) {
            let break_interval_end = self.parameters.break_interval.end;
            if break_interval_end < current_time.time() {
                current_time = current_time.with_time(break_interval_end).unwrap();
                current_time += TimeDelta::days(1);
                Some(current_time)
            } else {
                current_time = current_time.with_time(break_interval_end).unwrap();
                Some(current_time)
            }
        } else if self.parameters.toolbox_interval.contains(&current_time) {
            let toolbox_interval_end = self.parameters.toolbox_interval.end;
            if toolbox_interval_end < current_time.time() {
                current_time = current_time.with_time(toolbox_interval_end).unwrap();
                current_time += TimeDelta::days(1);
                Some(current_time)
            } else {
                current_time = current_time.with_time(toolbox_interval_end).unwrap();
                Some(current_time)
            }
        } else {
            None
        }
    }
}

fn no_overlap(events: &Vec<Assignment>) -> bool {
    for event_1 in events {
        for event_2 in events {
            if event_1 == event_2 {
                continue;
            }

            if (event_1.finish <= event_2.start) || (event_2.finish <= event_1.start) {
                continue;
            } else {
                dbg!(event_1);
                dbg!(event_2);
                return false;
            }
        }
    }
    true
}

fn no_overlap_by_ref(events: Vec<&Assignment>) -> bool {
    for event_1 in &events {
        for event_2 in &events {
            if event_1 == event_2 {
                continue;
            }

            if (event_1.finish <= event_2.start) || (event_2.finish <= event_1.start) {
                continue;
            } else {
                dbg!(event_1);
                dbg!(event_2);
                return false;
            }
        }
    }
    true
}

fn is_assignments_in_bounds(events: &Vec<Assignment>, availability: &Availability) -> bool {
    for event in events {
        if event.start < availability.start_date && !event.operational_events.unavail() {
            dbg!(event, availability);
            return false;
        }
        if availability.finish_date < event.finish && !event.operational_events.unavail() {
            dbg!(event, availability);
            return false;
        }
    }
    true
}

fn equality_between_time_interval_and_assignments(all_events: &Vec<Assignment>) {
    for assignment in all_events {
        assert_eq!(
            assignment.start.time(),
            assignment.operational_events.start_time()
        );
        assert_eq!(
            assignment.finish.time(),
            assignment.operational_events.finish_time()
        );
        assert_eq!(
            assignment.operational_events.time_delta(),
            assignment.finish - assignment.start
        )
    }
}
impl<Ss> From<Algorithm<OperationalSolution, OperationalParameters, OperationalNonProductive, Ss>>
    for OperationalAlgorithm<Ss>
where
    Ss: SystemSolutionTrait,
{
    fn from(
        value: Algorithm<OperationalSolution, OperationalParameters, OperationalNonProductive, Ss>,
    ) -> Self {
        OperationalAlgorithm(value)
    }
}
