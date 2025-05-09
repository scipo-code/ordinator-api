use std::collections::HashMap;

use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use colored::Colorize;
use ordinator_actor_core::traits::ObjectiveValue;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::marginal_fitness::MarginalFitness;
use ordinator_scheduling_environment::time_environment::TimeInterval;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::availability::Availability;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use serde::Serialize;

// This is for the `constracts`, `conversions`, and the `orchstrator` to handle.
use super::ContainOrNextOrNone;
use super::Unavailability;
use super::no_overlap_by_ref;
use super::operational_events::OperationalEvents;
use super::operational_parameter::OperationalParameters;

/// You want this to be a struct so that you can implement methods and
/// formatting and logging.
#[derive(Serialize, Copy, PartialEq, PartialOrd, Ord, Eq, Debug, Default, Clone)]
pub struct OperationalObjectiveValue(pub u64);

impl ObjectiveValue for OperationalObjectiveValue {}

impl From<u64> for OperationalObjectiveValue {
    fn from(value: u64) -> Self {
        OperationalObjectiveValue(value)
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct OperationalSolution {
    pub objective_value: OperationalObjectiveValue,
    pub scheduled_work_order_activities: Vec<(WorkOrderActivity, OperationalAssignment)>,
}

impl Solution for OperationalSolution {
    type ObjectiveValue = OperationalObjectiveValue;
    type Parameters = OperationalParameters;

    fn new(parameters: &Self::Parameters) -> Self {
        let mut scheduled_work_order_activities = Vec::new();

        let start_event =
            Assignment::make_unavailable_event(Unavailability::Beginning, &parameters.availability);

        let end_event =
            Assignment::make_unavailable_event(Unavailability::End, &parameters.availability);

        let unavailability_start_event = OperationalAssignment::new(vec![start_event]);

        let unavailability_end_event = OperationalAssignment::new(vec![end_event]);

        scheduled_work_order_activities.push(((WorkOrderNumber(0), 0), unavailability_start_event));

        scheduled_work_order_activities.push(((WorkOrderNumber(0), 0), unavailability_end_event));

        Self {
            objective_value: OperationalObjectiveValue(0),
            scheduled_work_order_activities,
        }
    }

    fn update_objective_value(&mut self, other_objective_value: Self::ObjectiveValue) {
        self.objective_value = other_objective_value;
    }
}

#[allow(dead_code)]
pub trait GetMarginalFitness {
    fn marginal_fitness(
        &self,
        operational_agent: &Id,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<&MarginalFitness>;
}
impl GetMarginalFitness for HashMap<Id, OperationalSolution> {
    fn marginal_fitness(
        &self,
        operational_agent: &Id,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<&MarginalFitness> {
        self.get(operational_agent)
            .with_context(|| {
                format!(
                    "Could not find {} for operational agent: {:#?}",
                    std::any::type_name::<MarginalFitness>(),
                    operational_agent,
                )
            })?
            .scheduled_work_order_activities
            .iter()
            .find(|woa_os| woa_os.0 == *work_order_activity)
            .map(|os| &os.1.marginal_fitness)
            .with_context(|| {
                format!(
                    "{} did not have\n{:#?}",
                    operational_agent.to_string().bright_blue(),
                    format!("{:#?}", work_order_activity).bright_yellow()
                )
            })
    }
}

// I think that we should have a Generic solution struct.
impl OperationalSolution {
    pub fn is_operational_solution_already_scheduled(
        &self,
        work_order_activity: WorkOrderActivity,
    ) -> bool {
        self.scheduled_work_order_activities
            .iter()
            .any(|(woa, _)| *woa == work_order_activity)
    }
}

pub trait OperationalFunctions {
    type Key;
    type Sequence;

    fn try_insert(&mut self, key: Self::Key, sequence: Self::Sequence);

    fn containing_operational_solution(&self, time: DateTime<Utc>) -> ContainOrNextOrNone;
}

impl OperationalFunctions for OperationalSolution {
    type Key = WorkOrderActivity;
    type Sequence = Vec<Assignment>;

    fn try_insert(&mut self, key: Self::Key, assignments: Self::Sequence) {
        for (index, operational_solution) in self
            .scheduled_work_order_activities
            .iter()
            .map(|os| os.1.clone())
            .collect::<Vec<_>>()
            .windows(2)
            .map(|x| (&x[0], &x[1]))
            .enumerate()
        {
            let start_of_solution_window = operational_solution.0.finish_time();

            let end_of_solution_window = operational_solution.1.start_time();

            if start_of_solution_window
                < assignments
                    .first()
                    .expect("No Assignment in the OperationalSolution")
                    .start
                && assignments.last().unwrap().finish < end_of_solution_window
            {
                let operational_solution = OperationalAssignment::new(assignments);

                if !self.is_operational_solution_already_scheduled(key) {
                    self.scheduled_work_order_activities
                        .insert(index + 1, (key, operational_solution));
                    let assignments = self
                        .scheduled_work_order_activities
                        .iter()
                        .flat_map(|(_, os)| &os.assignments)
                        .collect();

                    assert!(no_overlap_by_ref(assignments));
                }
                break;
            }
        }
    }

    fn containing_operational_solution(&self, time: DateTime<Utc>) -> ContainOrNextOrNone {
        let containing: Option<OperationalAssignment> = self
            .scheduled_work_order_activities
            .iter()
            .find(|operational_solution| operational_solution.1.contains(time))
            .map(|(_, os)| os)
            .cloned();

        match containing {
            Some(containing) => ContainOrNextOrNone::Contain(containing),
            None => {
                let next: Option<OperationalAssignment> = self
                    .scheduled_work_order_activities
                    .iter()
                    .map(|os| os.1.clone())
                    .find(|start| start.start_time() > time);

                match next {
                    Some(operational_solution) => ContainOrNextOrNone::Next(operational_solution),
                    None => ContainOrNextOrNone::None,
                }
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct OperationalAssignment {
    // This is an auxilliary objective value. Where should it lie to solve this issue? You
    // need one per `WorkOrderActivity` so removing it does not really make that much sense
    // I think that you have to store them in the solution.
    pub marginal_fitness: MarginalFitness,
    pub assignments: Vec<Assignment>,
}

impl OperationalAssignment {
    pub fn new(assignments: Vec<Assignment>) -> Self {
        Self {
            assignments,
            marginal_fitness: MarginalFitness::default(),
        }
    }

    /// Start time of the Whole Assignment Vec
    pub fn start_time(&self) -> DateTime<Utc> {
        self.assignments.first().unwrap().start
    }

    pub fn finish_time(&self) -> DateTime<Utc> {
        self.assignments.last().unwrap().finish
    }

    pub fn contains(&self, time: DateTime<Utc>) -> bool {
        self.start_time() <= time && time < self.finish_time()
    }
}

// This kind of behavior should be part of the `SharedSolutionTrait`
// The issue here is that the code is not ready for use. We have to
// change the different
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Assignment {
    pub operational_events: OperationalEvents,
    pub start: DateTime<Utc>,
    pub finish: DateTime<Utc>,
}

impl Assignment {
    pub fn new(event_type: OperationalEvents, start: DateTime<Utc>, finish: DateTime<Utc>) -> Self {
        assert_eq!(event_type.time_delta(), finish - start);
        assert!(start < finish);
        assert_eq!(event_type.start_time(), start.time());
        assert_eq!(event_type.finish_time(), finish.time());
        Self {
            operational_events: event_type,
            start,
            finish,
        }
    }

    pub fn make_unavailable_event(kind: Unavailability, availability: &Availability) -> Self {
        match kind {
            Unavailability::Beginning => {
                let event_start_time = availability
                    .start_date
                    .clone()
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                let event_finish_time = availability.start_date;

                Assignment::new(
                    OperationalEvents::Unavailable(TimeInterval::from_date_times(
                        event_start_time,
                        event_finish_time,
                    )),
                    event_start_time,
                    event_finish_time,
                )
            }
            Unavailability::End => {
                let event_start_time = availability.finish_date;
                let event_finish_time = availability
                    .finish_date
                    .clone()
                    .date_naive()
                    .and_hms_opt(23, 59, 59)
                    .unwrap()
                    .and_utc();

                Assignment::new(
                    OperationalEvents::Unavailable(TimeInterval::from_date_times(
                        event_start_time,
                        event_finish_time,
                    )),
                    event_start_time,
                    event_finish_time,
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ordinator_orchestrator_actor_traits::marginal_fitness::MarginalFitness;

    #[test]
    fn test_marginal_fitness_debug() {
        let marginal_fitness = MarginalFitness::Scheduled(3600);

        let formatted_marginal_fitness = format!("{:?}", marginal_fitness);

        assert_eq!(
            formatted_marginal_fitness,
            "MarginalFitness::Scheduled(3600, 1, 0)"
        );
    }
}
