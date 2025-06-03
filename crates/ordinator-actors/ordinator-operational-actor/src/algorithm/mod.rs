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
use ordinator_actor_core::traits::ObjectiveValueType;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::StrategicInterface;
use ordinator_orchestrator_actor_traits::SupervisorInterface;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::TacticalInterface;
use ordinator_orchestrator_actor_traits::delegate::Delegate;
use ordinator_orchestrator_actor_traits::marginal_fitness::MarginalFitness;
use ordinator_scheduling_environment::time_environment::TimeInterval;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::OperationalOptions;
use ordinator_scheduling_environment::worker_environment::availability::Availability;
use rand::seq::IndexedRandom;
use tracing::Level;
use tracing::event;

pub struct OperationalAlgorithm<Ss>(
    Algorithm<OperationalSolution, OperationalParameters, FillinOperationalEvents, Ss>,
)
where
    Ss: SystemSolutions,
    Algorithm<OperationalSolution, OperationalParameters, FillinOperationalEvents, Ss>: AbLNSUtils;

// This has the wrong name, it is simply the fill out.
#[derive(Clone, Default, Debug)]
pub struct FillinOperationalEvents(pub Vec<Assignment>);

pub trait OperationalTraitUtils
{
    fn determine_next_event_non_productive(
        &mut self,
        current_time: &mut DateTime<Utc>,
        next_operation: Option<&OperationalAssignment>,
    ) -> Result<(DateTime<Utc>, OperationalEvents)>;

    fn determine_time_interval_of_function(
        &mut self,
        next_operation: Option<&OperationalAssignment>,
        current_time: &DateTime<Utc>,
        interval: TimeInterval,
    ) -> Result<TimeInterval>;

    fn update_marginal_fitness(
        &mut self,
        work_order_activity_previous: (WorkOrderNumber, ActivityNumber),
        time_delta: TimeDelta,
    );
}

// Do you want this on the OperationalAlgorithm?
impl<Ss> OperationalTraitUtils for OperationalAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    fn determine_next_event_non_productive(
        &mut self,
        current_time: &mut DateTime<Utc>,
        next_operation: Option<&OperationalAssignment>,
    ) -> Result<(DateTime<Utc>, OperationalEvents)>
    {
        // So the error is in here now. That means that we should strive for
        // making this
        if self.0.parameters.break_interval.contains(current_time) {
            let time_interval = self.determine_time_interval_of_function(
                next_operation,
                current_time,
                self.0.parameters.break_interval.clone(),
            )?;
            let new_current_time = *current_time + time_interval.duration();
            Ok((new_current_time, OperationalEvents::Break(time_interval)))
        } else if self.0.parameters.off_shift_interval.contains(current_time) {
            let time_interval = self.determine_time_interval_of_function(
                next_operation,
                current_time,
                self.0.parameters.off_shift_interval.clone(),
            )?;
            let new_current_time = *current_time + time_interval.duration();
            Ok((
                new_current_time,
                OperationalEvents::OffShift(self.0.parameters.off_shift_interval.clone()),
            ))
        } else if self.0.parameters.toolbox_interval.contains(current_time) {
            let time_interval = self.determine_time_interval_of_function(
                next_operation,
                current_time,
                self.0.parameters.toolbox_interval.clone(),
            )?;
            let new_current_time = *current_time + time_interval.duration();
            Ok((
                new_current_time,
                OperationalEvents::Toolbox(self.0.parameters.toolbox_interval.clone()),
            ))
        } else {
            // All this is much more complex than it needs to be. I can feel it.
            //
            let start = *current_time;
            let (time_until_next_event, next_operational_event) =
                self.determine_next_event(current_time).with_context(|| {
                    format!(
                        "Could not determine the next event\n{}:{}",
                        file!(),
                        line!()
                    )
                })?;
            // We should think of this in
            let mut new_current_time = *current_time + time_until_next_event;

            if *current_time == new_current_time {
                Ok((
                    new_current_time + next_operational_event.time_delta(),
                    next_operational_event,
                ))
            } else {
                // TODO [x] BREAK
                if self.0.parameters.availability.finish_date < new_current_time {
                    new_current_time = self.0.parameters.availability.finish_date;
                }
                let time_interval = TimeInterval::new(start.time(), new_current_time.time())?;
                Ok((
                    new_current_time,
                    // So the issue is here. This is the only place that the
                    // `OperationalEvents::NonProductiveTime` is instantiated.
                    // Should you simply continue here? You should take a break now. Meditate
                    // is the best course of action.
                    OperationalEvents::NonProductiveTime(time_interval),
                ))
            }
        }
    }

    // This function makes sure that the created event is adjusted to fit the
    // schedule if there has been any manual intervention in the schedule for
    // the OperationalAgent.
    //
    // You should feel good about this! Maturity comes here from accepting your
    // own weaknesses.
    fn determine_time_interval_of_function(
        &mut self,
        next_operation: Option<&OperationalAssignment>,
        current_time: &DateTime<Utc>,
        interval: TimeInterval,
    ) -> Result<TimeInterval>
    {
        // What is this code actually trying to do? I think
        let time_interval: TimeInterval = match next_operation {
            Some(operational_solution) => {
                if operational_solution.start_time().date_naive() == current_time.date_naive() {
                    TimeInterval::new(
                        current_time.time(),
                        interval.end.min(operational_solution.start_time().time()),
                    )?
                } else {
                    TimeInterval::new(current_time.time(), interval.end)?
                }
            }

            None => TimeInterval::new(current_time.time(), interval.end)?,
        };
        Ok(time_interval)
    }

    // This is a problem. What should you do about it? I think that the best thing
    // that you can do is move all this into the `schedule` function and handle
    // it while the code is running. That is probably the best call here. I
    // do not see what other way it could be done in a better way.
    fn update_marginal_fitness(
        &mut self,
        work_order_activity_previous: (WorkOrderNumber, ActivityNumber),
        time_delta: TimeDelta,
    )
    {
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

pub enum ContainOrNextOrNone
{
    Contain(OperationalAssignment),
    Next(OperationalAssignment),
    None,
}

pub enum Unavailability
{
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
    Ss: SystemSolutions,
{
    type Target =
        Algorithm<OperationalSolution, OperationalParameters, FillinOperationalEvents, Ss>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss> DerefMut for OperationalAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

impl<Ss> ActorBasedLargeNeighborhoodSearch for OperationalAlgorithm<Ss>
where
    Ss: SystemSolutions<Operational = OperationalSolution>,
    Algorithm<OperationalSolution, OperationalParameters, FillinOperationalEvents, Ss>:
        AbLNSUtils<SolutionType = OperationalSolution>,
{
    type Algorithm =
        Algorithm<OperationalSolution, OperationalParameters, FillinOperationalEvents, Ss>;
    type Options = OperationalOptions;

    fn incorporate_system_solution(&mut self) -> Result<bool>
    {
        let operational_shared_solution = self
            .loaded_shared_solution
            .supervisor_actor_solutions()
            .with_context(|| {
                format!(
                    "SupervisorSolution not available to the OperationalActor:\n{}",
                    self.id
                )
            })?
            // The fact that this error was not propagated with `with_context` caused you a 5 minute
            // delay and significant redirections.
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
        // This should be abstracted away at some point.
        self.arc_swap_shared_solution.rcu(|old| {
            let mut shared_solution = (**old).clone();
            shared_solution.operational_swap(&self.id, self.solution.clone());
            Arc::new(shared_solution)
        });
    }

    // If we are going to implement delta evaluation we should remove this part.
    fn calculate_objective_value(
        &mut self,
    ) -> Result<
        ObjectiveValueType<
            <<Self::Algorithm as AbLNSUtils>::SolutionType as Solution>::ObjectiveValue,
        >,
    >
    {
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

        no_overlap(&all_events.clone().collect::<Vec<_>>()).with_context(|| {
            format!(
                "Overlap between work order activities\n{}:{}",
                file!(),
                line!()
            )
        })?;

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

        ensure!(is_assignments_in_bounds(
            &all_events.clone().collect(),
            &self.parameters.availability
        ));

        no_overlap(&all_events.collect::<Vec<_>>())
            .with_context(|| "Overlap between work order activities".to_string())?;

        let total_time =
            wrench_time + break_time + off_shift_time + toolbox_time + non_productive_time;

        ensure!(
            total_time == self.parameters.availability.duration(),
            self.solution_intermediate.0.len()
        );
        assert!(total_time == self.parameters.availability.duration());

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
        self.solution.objective_value = new_objective_value;
        if self.solution.objective_value > old_objective_value {
            event!(Level::INFO, operational_objective_value_better = ?new_objective_value);
            Ok(ObjectiveValueType::Better(new_objective_value))
        } else {
            event!(Level::INFO, operational_objective_value_worse = ?new_objective_value);
            Ok(ObjectiveValueType::Worse)
        }
    }

    fn schedule(&mut self) -> Result<()>
    {
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
            .supervisor_actor_solutions()
            .with_context(|| "SupervisorSolution is not initialized for the OperationalActor")?
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
            ensure!(!operational_parameter.work.is_zero());

            let start_time = self
                .determine_first_available_start_time(work_order_activity, operational_parameter)
                .with_context(|| format!("{work_order_activity:#?}"))?;

            let assignments = self
                .determine_wrench_time_assignment(
                    *work_order_activity,
                    operational_parameter,
                    start_time,
                )
                .with_context(|| {
                    format!(
                        "Error in determining the wrench time assignment for OperationalActor\n{}\n{work_order_activity:#?}\nstart_time: {start_time}\navailability: {:#?}",
                        self.id,
                        self.parameters.availability,
                    )
                })?;

            self.solution.try_insert(*work_order_activity, assignments);
        }

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
        no_overlap(&all_events.collect::<Vec<_>>())
            .with_context(|| "Overlap between work order activities".to_string())?;
        let mut current_time = self.parameters.availability.start_date;

        // Fill the schedule
        // What does `ContainOrNextOrNone` even mean here? The fact that you do not know
        // is a serious issue.
        loop {
            match self.solution.containing_operational_solution(current_time) {
                ContainOrNextOrNone::Contain(individual_operational_assignment) => {
                    current_time = individual_operational_assignment.finish_time();
                }

                // You are having trouble here because you have not named things correctly. That is
                // the main issue here.
                ContainOrNextOrNone::Next(individual_operational_assignment) => {
                    let (new_current_time, operational_event) = self
                        .determine_next_event_non_productive(
                            &mut current_time,
                            Some(&individual_operational_assignment),
                        )?;
                    ensure!(!operational_event.is_wrench_time());
                    ensure!(operational_event.time_delta() == new_current_time - current_time);
                    // The amount of business logic that has to go into all of this is enourmous.
                    let assignment =
                        Assignment::new(operational_event, current_time, new_current_time)
                            .with_context(|| format!("Could not create the a work assignment\ncurrent_time: {current_time}\n new_current_time: {new_current_time}"))?;

                    current_time = new_current_time;

                    // You need to assign the reason for the error every where. This is crucial.
                    // Okay so what should happen to these two? I think that the best approach
                    // is to simply.
                    //
                    // The issue with using a debugger is that you would have to note down
                    // where everything lies in order for it to work correctly
                    // What do we want to test now?
                    // ensure!(
                    //     (event_1.finish <= event_2.start) || (event_2.finish <= event_1.start),
                    //     "event_1: {event_1:#?}\nevent_2: {event_2:#?}"
                    // );
                    // Remain calm. That is crucial.
                    ensure!(
                        individual_operational_assignment
                            .assignments
                            .first()
                            .context("operational_solution should not be empty")?
                            .start
                            >= assignment.finish,
                        "{:<30}: {current_time}\n\n{:<30}: {:#?}\n\n{:<30}: {:#?}\n\n{:<30}: {:#?}",
                        "current_time",
                        "operational_solution_finish",
                        // Why is there only a single element in this one? That is a
                        // mistake that should be fixed.
                        individual_operational_assignment.assignments,
                        "assignment",
                        assignment,
                        "non_productive",
                        self.solution_intermediate,
                    );
                    // This is the error. Hmm... The question then becomes who
                    // should handle the hjjk
                    self.solution_intermediate.0.push(assignment);
                }
                // I think that this should be renamed.
                ContainOrNextOrNone::None => {
                    let (new_current_time, operational_event) = self
                        .determine_next_event_non_productive(&mut current_time, None)
                        .with_context(|| {
                            "Could not determine the next non-productive event".to_string()
                        })?;
                    ensure!(!operational_event.is_wrench_time());
                    ensure!(operational_event.time_delta() == new_current_time - current_time);
                    let assignment =
                        Assignment::new(operational_event, current_time, new_current_time)?;
                    current_time = new_current_time;
                    self.solution_intermediate.0.push(assignment);
                }
            };

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

            no_overlap(&all_events.collect::<Vec<_>>())
                .with_context(|| "Overlap between work order activities".to_string())?;
            if current_time >= self.parameters.availability.finish_date {
                self.solution_intermediate.0.last_mut().unwrap().finish =
                    self.parameters.availability.finish_date;
                break;
            };
        }
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
        no_overlap(&all_events.collect::<Vec<_>>())
            .with_context(|| "Overlap between work order activities".to_string())?;

        Ok(())
    }

    fn unschedule(&mut self) -> Result<()>
    {
        let mut rng = rand::rng();
        let operational_solutions_len = self.solution.scheduled_work_order_activities.len();

        let operational_solutions_filtered: Vec<WorkOrderActivity> =
            self.solution.scheduled_work_order_activities[1..operational_solutions_len - 1]
                .choose_multiple(
                    &mut rng,
                    self.parameters.options.number_of_removed_activities,
                )
                .map(|operational_solution| operational_solution.0)
                .collect();

        ensure!(
            (self
                .solution
                .scheduled_work_order_activities
                .first()
                .unwrap()
                .0
                .0
                == WorkOrderNumber(0))
        );
        ensure!(
            (self
                .solution
                .scheduled_work_order_activities
                .last()
                .unwrap()
                .0
                .0
                == WorkOrderNumber(0))
        );
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

    fn algorithm_util_methods(&mut self) -> &mut Self::Algorithm
    {
        &mut self.0
    }
}

impl<Ss> OperationalAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    // Okay, you got an error here, but you do not understand where it came from.
    // That means that you did the error handling incorrectly.
    fn determine_wrench_time_assignment(
        &self,
        work_order_activity: WorkOrderActivity,
        // You have to handle the case where the
        operational_parameter: &OperationalParameter,
        start_time: DateTime<Utc>,
    ) -> Result<Vec<Assignment>>
    {
        ensure!(operational_parameter.work != Work::from(0.0));
        // WARN
        // There is a much better way of expressing the this.
        ensure!(!operational_parameter.operation_time_delta.is_zero());
        let mut assigned_work: Vec<Assignment> = vec![];
        let mut remaining_combined_work = operational_parameter.operation_time_delta;
        let mut current_time = start_time;

        while !remaining_combined_work.is_zero() {
            let next_event = self.determine_next_event(&current_time).with_context(|| {
                format!("Next event was not created correctly\ncurrent time: {current_time}\nremaining_work: {remaining_combined_work}\nassigned_work: {assigned_work:#?}")
            })?;

            if next_event.0.is_zero() {
                let finish_time = current_time + next_event.1.time_delta();
                assigned_work.push(
                    Assignment::new(next_event.1, current_time, finish_time)
                        .with_context(|| "Could not create the a work assignment".to_string())?,
                );
                current_time = finish_time;
            } else if next_event.0 < remaining_combined_work {
                assigned_work.push(
                    Assignment::new(
                        OperationalEvents::WrenchTime((
                            TimeInterval::new(
                                current_time.time(),
                                (current_time + next_event.0).time(),
                            )
                            .with_context(|| "Could not create a valid TimeInterval".to_string())?,
                            work_order_activity,
                        )),
                        current_time,
                        current_time + next_event.0,
                    )
                    .with_context(|| format!("{}:{}", file!(), line!()))?,
                );
                current_time += next_event.0;
                remaining_combined_work -= next_event.0;
                ensure!(remaining_combined_work >= TimeDelta::new(0, 0).unwrap());
            } else if next_event.0 >= remaining_combined_work {
                ensure!(
                    remaining_combined_work
                        > TimeDelta::new(0, 0).context("Could not create a valid TimeDelta")?,
                    format!(
                        "next_event: {next_event:#?}\nremaining_combined_work: {remaining_combined_work:#?}\ncurrent_time: {current_time}"
                    )
                );
                assigned_work.push(
                    Assignment::new(
                        OperationalEvents::WrenchTime((
                            TimeInterval::new(
                                current_time.time(),
                                (current_time + remaining_combined_work).time(),
                            )
                            .with_context(|| "Could not create a valid TimeInterval".to_string())?,
                            work_order_activity,
                        )),
                        current_time,
                        current_time + remaining_combined_work,
                    )
                    .with_context(|| format!("{}:{}", file!(), line!()))?,
                );
                current_time += next_event.0;
                remaining_combined_work = TimeDelta::zero();
            }
        }
        assert_ne!(assigned_work.len(), 0);
        Ok(assigned_work)
    }

    fn unschedule_single_work_order_activity(
        &mut self,
        work_order_and_activity_number: WorkOrderActivity,
    ) -> Result<()>
    {
        ensure!(
            self.solution
                .scheduled_work_order_activities
                .iter()
                .any(|os| os.0 == work_order_and_activity_number)
        );

        self.solution
            .scheduled_work_order_activities
            .retain(|os| os.0 != work_order_and_activity_number);

        ensure!(
            !self
                .solution
                .scheduled_work_order_activities
                .iter()
                .any(|os| os.0 == work_order_and_activity_number)
        );
        Ok(())
    }

    /// Determining the next event in the operational sequencing.
    /// The TimeDelta return is the time until the next event and
    /// `OperationalEvents` is the next upcoming event. This means that
    /// the TimeDelta here is the effective time that can be used
    /// for wrench time and preparation time.
    fn determine_next_event(
        &self,
        current_time: &DateTime<Utc>,
    ) -> Result<(TimeDelta, OperationalEvents)>
    {
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

        // So the current time is wrong. I think tha
        // ensure!(
        //     break_diff.0 > TimeDelta::new(0, 0).unwrap(),
        //     format!(
        //         "Current time: {current_time}\nbreak_diff: {break_diff:#?}\n{}:{}",
        //         file!(),
        //         line!()
        //     )
        // );
        // ensure!(
        //     toolbox_diff.0 > TimeDelta::new(0, 0).unwrap(),
        //     format!(
        //         "Current time: {current_time}\ntoolbox_diff:
        // {toolbox_diff:#?}\n{}:{}",         file!(),
        //         line!()
        //     )
        // );
        // ensure!(
        //     off_shift_diff.0 > TimeDelta::new(0, 0).unwrap(),
        //     format!(
        //         "Current time: {current_time}\noff_shift_diff:
        // {off_shift_diff:#?}\n{}:{}",         file!(),
        //         line!()
        //     )
        // );
        // FIX
        // This is completely idiotic! We should never duplicate the state
        // This is so fucking dumb. NEVER EVER DO THIS AGAIN!
        // This is wrong, the time delta is the time until you hit
        // the next event.
        // ensure!(
        //     break_diff.1.time_delta() == break_diff.0,
        //     format!(
        //         "Current time: {current_time}\nbreak_diff: {break_diff:#?}\n{}:{}",
        //         file!(),
        //         line!()
        //     )
        // );
        // ensure!(
        //     toolbox_diff.1.time_delta() == break_diff.0,
        //     format!(
        //         "Current time: {current_time}\ntoolbox_diff:
        // {toolbox_diff:#?}\n{}:{}",         file!(),
        //         line!()
        //     )
        // );
        // ensure!(
        //     off_shift_diff.1.time_delta() == break_diff.0,
        //     format!(
        //         "Current time: {current_time}\noff_shift_diff:
        // {off_shift_diff:#?}\n{}:{}",         file!(),
        //         line!()
        //     )
        // );

        Ok([break_diff, toolbox_diff, off_shift_diff]
            .iter()
            .filter(|&diff_event| diff_event.0.num_seconds() >= 0)
            .min_by_key(|&diff_event| diff_event.0.num_seconds())
            .cloned()
            .unwrap())
    }

    fn determine_first_available_start_time(
        &self,
        work_order_activity: &WorkOrderActivity,
        operational_parameter: &OperationalParameter,
    ) -> Result<DateTime<Utc>>
    {
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
            .tactical_actor_solution()?
            // FIX [ ]
            // Rename this! It is basically a way of sharing what the
            // `tactical` actor needs to do his scheduling.
            .start_and_finish_dates(work_order_activity);

        // .expect("This should always be present. If this occurs you should check the
        // initialization. The implementation is that the tactical and strategic
        // algorithm always provide a key for each WorkOrderNumber");
        let strategic_period_option = self
            .loaded_shared_solution
            .strategic()?
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
            (Some(None), _) => (
                &self.parameters.availability.start_date,
                &self.parameters.availability.finish_date,
            ),
            // WARN
            // This kind of code should be made with `AppError`. You should have a centralized error
            // strategy aimed at making quick iterations on the scheduling logic.
            // _ => bail!(
            //     "This means that there is no state in either the Tactical or the Strategic agent.
            // This should not be possible as the \     OperationalActor gets its state
            // from either of those. An exception is if the the WorkOrder has left the
            // StrategicActor or \     the TacticalActor, and the supervisor still have
            // the WorkOrderActivity in his state.\nSupervisorActor state: \
            //     \nIs WorkOrder {:#?} present in StrategicActor: {:?} \
            //     \nIs WorkOrder {:#?} present in TacticalActor : {:?} \
            //     \nIs WorkOrderActivity {:?} present in SupervisorActor: {:?} \
            //     \n{}:{}",
            //     work_order_activity.0,
            //     self.loaded_shared_solution
            //         .strategic()
            //         .unwrap()
            //         .scheduled_task(&work_order_activity.0),
            //     work_order_activity.0,
            //     self.loaded_shared_solution
            //         .tactical_actor_solution()
            //         .unwrap()
            //         .start_and_finish_dates(work_order_activity),
            //     work_order_activity,
            //     self.loaded_shared_solution
            //         .supervisor_actor_solutions()
            //         .unwrap()
            //         .delegates_for_agent(&self.id)
            //         .contains_key(work_order_activity),
            //     file!(),
            //     line!()
            // ),
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
                    let (time_to_next_event, next_event) =
                        self.determine_next_event(&current_time).with_context(|| {
                            format!(
                                "Could not determine the next event\n{}:{}",
                                file!(),
                                line!()
                            )
                        })?;

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
            let (time_to_next_event, next_event) =
                self.determine_next_event(&current_time).with_context(|| {
                    format!(
                        "Could not determine the next event\n{}:{}",
                        file!(),
                        line!()
                    )
                })?;

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
    ) -> Option<DateTime<Utc>>
    {
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

fn no_overlap(events: &Vec<Assignment>) -> Result<()>
{
    for event_1 in events {
        for event_2 in events {
            if event_1 == event_2 {
                continue;
            }

            ensure!(
                (event_1.finish <= event_2.start) || (event_2.finish <= event_1.start),
                "event_1: {event_1:#?}\nevent_2: {event_2:#?}"
            );
        }
    }
    Ok(())
}

fn no_overlap_by_ref(events: Vec<&Assignment>) -> bool
{
    for event_1 in &events {
        for event_2 in &events {
            if event_1 == event_2 {
                continue;
            }

            if (event_1.finish <= event_2.start) || (event_2.finish <= event_1.start) {
                continue;
            } else {
                return false;
            }
        }
    }
    true
}

fn is_assignments_in_bounds(events: &Vec<Assignment>, availability: &Availability) -> bool
{
    for event in events {
        if event.start < availability.start_date && !event.operational_events.unavail() {
            return false;
        }
        if availability.finish_date < event.finish && !event.operational_events.unavail() {
            return false;
        }
    }
    true
}

fn equality_between_time_interval_and_assignments(all_events: &Vec<Assignment>)
{
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
impl<Ss> From<Algorithm<OperationalSolution, OperationalParameters, FillinOperationalEvents, Ss>>
    for OperationalAlgorithm<Ss>
where
    Ss: SystemSolutions,
{
    fn from(
        value: Algorithm<OperationalSolution, OperationalParameters, FillinOperationalEvents, Ss>,
    ) -> Self
    {
        OperationalAlgorithm(value)
    }
}
