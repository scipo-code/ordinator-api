<<<<<<< HEAD
use chrono::{DateTime, Utc};
use shared_types::{
    operational::{operational_response_scheduling::ApiAssignmentEvents, TimeInterval},
    scheduling_environment::{
        work_order::WorkOrderActivity, worker_environment::availability::Availability,
    },
};
use strum_macros::AsRefStr;
use tracing::{event, Level};

use crate::agents::{operational_agent::algorithm::no_overlap_by_ref, OperationalSolution};

use super::{operational_events::OperationalEvents, ContainOrNextOrNone, Unavailability};

impl OperationalSolution {
    pub fn new(work_order_activities: Vec<(WorkOrderActivity, OperationalAssignment)>) -> Self {
        Self {
            objective_value: 0,
            work_order_activities_assignment: work_order_activities,
        }
    }

    pub fn is_operational_solution_already_scheduled(
        &self,
        work_order_activity: WorkOrderActivity,
    ) -> bool {
        self.work_order_activities_assignment
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
            .work_order_activities_assignment
            .iter()
            .map(|os| os.1.clone())
            .collect::<Vec<_>>()
            .windows(2)
            .map(|x| (&x[0], &x[1]))
            .enumerate()
        {
            let start_of_solution_window = operational_solution.0.finish_time();

            let end_of_solution_window = operational_solution.1.start_time();

            event!(Level::TRACE, start_window = ?start_of_solution_window);
            event!(Level::TRACE,
                first_assignment = ?assignments
                    .first()
                    .expect("No Assignment in the OperationalSolution")
                    .start,
            );
            event!(Level::TRACE, last_assignment = ?assignments.last().unwrap().finish);
            event!(Level::TRACE, end_window = ?end_of_solution_window);

            if start_of_solution_window
                < assignments
                    .first()
                    .expect("No Assignment in the OperationalSolution")
                    .start
                && assignments.last().unwrap().finish < end_of_solution_window
            {
                let operational_solution = OperationalAssignment::new(assignments);

                if !self.is_operational_solution_already_scheduled(key) {
                    self.work_order_activities_assignment
                        .insert(index + 1, (key, operational_solution));
                    let assignments = self
                        .work_order_activities_assignment
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
            .work_order_activities_assignment
            .iter()
            .find(|operational_solution| operational_solution.1.contains(time))
            .map(|(_, os)| os)
            .cloned();

        match containing {
            Some(containing) => ContainOrNextOrNone::Contain(containing),
            None => {
                let next: Option<OperationalAssignment> = self
                    .work_order_activities_assignment
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

impl From<Assignment> for ApiAssignmentEvents {
    fn from(_value: Assignment) -> Self {
        todo!()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Assignment {
    pub event_type: OperationalEvents,
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
            event_type,
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
                let event_start_time = availability.end_date;
                let event_finish_time = availability
                    .end_date
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

#[derive(AsRefStr, Eq, PartialEq, PartialOrd, Ord, Clone, Default)]
pub enum MarginalFitness {
    Scheduled(u64),
    #[default]
    None,
}

impl std::fmt::Debug for MarginalFitness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarginalFitness::Scheduled(time) => write!(
                f,
                "{}::{}({}, {}, {})",
                std::any::type_name::<MarginalFitness>()
                    .split("::")
                    .last()
                    .unwrap(),
                self.as_ref(),
                time,
                time / 3600,
                time / 3600 / 24,
            ),
            MarginalFitness::None => write!(f, "{}", self.as_ref()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::agents::operational_agent::algorithm::operational_solution::MarginalFitness;

    #[test]
    fn test_marginal_fitness_debug() {
        let marginal_fitness = MarginalFitness::Scheduled(3600);

        let formatted_marginal_fitness = format!("{:?}", marginal_fitness);

        assert_eq!(
            formatted_marginal_fitness,
            "Scheduled(Seconds: 3600, Hours: 1)"
        );
    }
}
||||||| 83bbf8b
=======
use chrono::{DateTime, Utc};
use shared_types::{
    operational::{operational_response_scheduling::ApiAssignmentEvents, TimeInterval},
    scheduling_environment::{
        work_order::WorkOrderActivity, worker_environment::availability::Availability,
    },
};
use tracing::{event, Level};

use crate::agents::{operational_agent::algorithm::no_overlap_by_ref, OperationalSolution};

use super::{operational_events::OperationalEvents, ContainOrNextOrNone, Unavailability};

impl OperationalSolution {
    pub fn new(work_order_activities: Vec<(WorkOrderActivity, OperationalAssignment)>) -> Self {
        Self {
            objective_value: 0,
            work_order_activities_assignment: work_order_activities,
        }
    }

    pub fn is_operational_solution_already_scheduled(
        &self,
        work_order_activity: WorkOrderActivity,
    ) -> bool {
        self.work_order_activities_assignment
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
            .work_order_activities_assignment
            .iter()
            .map(|os| os.1.clone())
            .collect::<Vec<_>>()
            .windows(2)
            .map(|x| (&x[0], &x[1]))
            .enumerate()
        {
            let start_of_solution_window = operational_solution.0.finish_time();

            let end_of_solution_window = operational_solution.1.start_time();

            event!(Level::TRACE, start_window = ?start_of_solution_window);
            event!(Level::TRACE,
                first_assignment = ?assignments
                    .first()
                    .expect("No Assignment in the OperationalSolution")
                    .start,
            );
            event!(Level::TRACE, last_assignment = ?assignments.last().unwrap().finish);
            event!(Level::TRACE, end_window = ?end_of_solution_window);

            if start_of_solution_window
                < assignments
                    .first()
                    .expect("No Assignment in the OperationalSolution")
                    .start
                && assignments.last().unwrap().finish < end_of_solution_window
            {
                let operational_solution = OperationalAssignment::new(assignments);

                if !self.is_operational_solution_already_scheduled(key) {
                    self.work_order_activities_assignment
                        .insert(index + 1, (key, operational_solution));
                    let assignments = self
                        .work_order_activities_assignment
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
            .work_order_activities_assignment
            .iter()
            .find(|operational_solution| operational_solution.1.contains(time))
            .map(|(_, os)| os)
            .cloned();

        match containing {
            Some(containing) => ContainOrNextOrNone::Contain(containing),
            None => {
                let next: Option<OperationalAssignment> = self
                    .work_order_activities_assignment
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

impl From<Assignment> for ApiAssignmentEvents {
    fn from(_value: Assignment) -> Self {
        todo!()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Assignment {
    pub event_type: OperationalEvents,
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
            event_type,
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
                let event_start_time = availability.end_date;
                let event_finish_time = availability
                    .end_date
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

#[derive(Eq, PartialEq, PartialOrd, Ord, Debug, Clone, Default)]
pub enum MarginalFitness {
    Fitness(u64),
    #[default]
    None,
}
>>>>>>> d3bf08ac4751bc3e07ab1bec22efb19272c0fba9
