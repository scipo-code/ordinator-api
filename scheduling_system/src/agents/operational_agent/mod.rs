pub mod algorithm;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use shared_messages::{
    agent_error::AgentError,
    operational::{
        operational_response_status::OperationalStatusResponse, OperationalConfiguration,
        OperationalInfeasibleCases, OperationalRequestMessage, OperationalResponseMessage,
    },
    scheduling_environment::{
        time_environment::day::Day,
        work_order::{operation::ActivityNumber, WorkOrderNumber},
        worker_environment::resources::Id,
    },
    AlgorithmState, ConstraintState, StatusMessage, StopMessage,
};

use shared_messages::scheduling_environment::{
    work_order::operation::Operation, SchedulingEnvironment,
};

use tracing::{info, warn};

use crate::agents::{operational_agent::algorithm::OperationalParameter, StateLink};

use self::algorithm::{Assignment, OperationalAlgorithm, OperationalSolution};

use super::{
    strategic_agent::ScheduleIteration,
    supervisor_agent::SupervisorAgent,
    tactical_agent::tactical_algorithm::OperationSolution,
    traits::{LargeNeighborHoodSearch, TestAlgorithm},
    SetAddr, UpdateWorkOrderMessage,
};

#[allow(dead_code)]
pub struct OperationalAgent {
    id_operational: Id,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    operational_algorithm: OperationalAlgorithm,
    capacity: Option<f32>,
    assigned: HashSet<(Assigned, WorkOrderNumber, ActivityNumber)>,
    backup_activities: Option<HashMap<u32, Operation>>,
    operational_configuration: OperationalConfiguration,
    supervisor_agent_addr: Addr<SupervisorAgent>,
}

type Assigned = bool;

impl OperationalAgent {
    fn determine_start_and_finish_times(
        &self,
        days: &[(Day, f64)],
    ) -> (DateTime<Utc>, DateTime<Utc>) {
        if days.len() == 1 {
            let start_of_time_window = Utc.from_utc_datetime(&NaiveDateTime::new(
                days.first().unwrap().0.date().date_naive(),
                self.operational_configuration.off_shift_interval.start,
            ));
            let end_of_time_window = Utc.from_utc_datetime(&NaiveDateTime::new(
                days.last().unwrap().0.date().date_naive(),
                self.operational_configuration.off_shift_interval.end,
            ));
            (start_of_time_window, end_of_time_window)
        } else {
            let start_day = days[0].0.date().date_naive();
            let end_day = days.last().unwrap().0.date().date_naive();
            let start_datetime = NaiveDateTime::new(
                start_day,
                self.operational_configuration.off_shift_interval.end
                    - Duration::seconds(3600 * days[0].1.round() as i64),
            );
            let end_datetime = NaiveDateTime::new(
                end_day,
                self.operational_configuration.off_shift_interval.start
                    + Duration::seconds(3600 * days.last().unwrap().1.round() as i64),
            );

            (
                Utc.from_utc_datetime(&start_datetime),
                Utc.from_utc_datetime(&end_datetime),
            )
        }
    }

    fn determine_operation_overlap(
        &self,
        operational_infeasible_cases: &mut OperationalInfeasibleCases,
    ) {
        for (index_1, operational_solution_1) in self
            .operational_algorithm
            .operational_solutions
            .0
            .iter()
            .enumerate()
        {
            for (index_2, operational_solution_2) in self
                .operational_algorithm
                .operational_solutions
                .0
                .iter()
                .enumerate()
            {
                if index_1 == index_2
                    || operational_solution_1.2.is_none()
                    || operational_solution_2.2.is_none()
                {
                    continue;
                }

                if operational_solution_1.2.as_ref().unwrap().start_time()
                    > operational_solution_2.2.as_ref().unwrap().finish_time()
                    && operational_solution_2.2.as_ref().unwrap().finish_time()
                        > operational_solution_1.2.as_ref().unwrap().start_time()
                {
                    operational_infeasible_cases.operation_overlap =
                        ConstraintState::Infeasible(format!(
                            "{:?} : {:?} is overlapping with {:?} : {:?}",
                            operational_solution_1.0,
                            operational_solution_1.1,
                            operational_solution_2.0,
                            operational_solution_2.1
                        ));
                    return;
                }
            }
        }
        operational_infeasible_cases.operation_overlap = ConstraintState::Feasible;
    }
}

impl Actor for OperationalAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.supervisor_agent_addr.do_send(SetAddr::Operational(
            self.id_operational.clone(),
            ctx.address(),
        ));

        let start_event = Assignment::make_unavailable_event(
            algorithm::Unavailability::Beginning,
            &self.operational_configuration.availability,
        );
        let end_event = Assignment::make_unavailable_event(
            algorithm::Unavailability::End,
            &self.operational_configuration.availability,
        );
        let unavailability_start_event = OperationalSolution::new(true, vec![start_event]);
        let unavailability_end_event = OperationalSolution::new(true, vec![end_event]);

        self.operational_algorithm.operational_solutions.0.push((
            WorkOrderNumber(0),
            ActivityNumber(0),
            Some(unavailability_start_event),
        ));

        self.operational_algorithm.operational_solutions.0.push((
            WorkOrderNumber(0),
            ActivityNumber(0),
            Some(unavailability_end_event),
        ));

        ctx.notify(ScheduleIteration {})
    }
}

impl Handler<ScheduleIteration> for OperationalAgent {
    type Result = ();

    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut Self::Context) -> Self::Result {
        let mut rng = rand::thread_rng();

        let mut temporary_schedule: OperationalAlgorithm = self.operational_algorithm.clone();

        ctx.wait(tokio::time::sleep(tokio::time::Duration::from_millis(10)).into_actor(self));
        temporary_schedule.unschedule_random_work_order_activies(&mut rng, 15);

        temporary_schedule.schedule();

        temporary_schedule.calculate_objective_value();

        if temporary_schedule.objective_value > self.operational_algorithm.objective_value {
            self.operational_algorithm = temporary_schedule;

            for operational_solution in &self.operational_algorithm.operational_solutions.0 {
                self.supervisor_agent_addr.do_send(StateLink::Operational((
                    (
                        self.id_operational.clone(),
                        operational_solution.0,
                        operational_solution.1,
                    ),
                    self.operational_algorithm.objective_value,
                )));
            }
            info!(operational_objective = %self.operational_algorithm.objective_value);
        };

        ctx.notify(ScheduleIteration {});
    }
}

impl Handler<OperationSolution> for OperationalAgent {
    type Result = bool;

    fn handle(
        &mut self,
        operation_solution: OperationSolution,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let scheduling_environment = self.scheduling_environment.lock().unwrap();

        let operation: &Operation = scheduling_environment.operation(
            &operation_solution.work_order_number,
            &operation_solution.activity_number,
        );

        let (start_datetime, end_datetime) =
            self.determine_start_and_finish_times(&operation_solution.scheduled);

        let operational_parameter = if operation.work_remaining() > 0.0 {
            OperationalParameter::new(
                operation.work_remaining(),
                operation.operation_analytic.preparation_time,
                start_datetime,
                end_datetime,
            )
        } else {
            return false;
        };
        self.assigned.insert((
            false,
            operation_solution.work_order_number,
            operation_solution.activity_number,
        ));

        self.operational_algorithm.insert_optimized_operation(
            operation_solution.work_order_number,
            operation_solution.activity_number,
            operational_parameter,
        );
        info!(id = ?self.id_operational, operation = ?operation_solution);
        true
    }
}

pub struct OperationalAgentBuilder {
    id_operational: Id,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    operational_algorithm: OperationalAlgorithm,
    capacity: Option<f32>,
    assigned: HashSet<(Assigned, WorkOrderNumber, ActivityNumber)>,
    backup_activities: Option<HashMap<u32, Operation>>,
    operational_configuration: OperationalConfiguration,
    supervisor_agent_addr: Addr<SupervisorAgent>,
}

impl OperationalAgentBuilder {
    pub fn new(
        id_operational: Id,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        operational_configuration: OperationalConfiguration,
        operational_algorithm: OperationalAlgorithm,
        supervisor_agent_addr: Addr<SupervisorAgent>,
    ) -> Self {
        OperationalAgentBuilder {
            id_operational,
            scheduling_environment,
            operational_algorithm,
            capacity: None,
            assigned: HashSet::new(),
            backup_activities: None,
            operational_configuration,
            supervisor_agent_addr,
        }
    }

    #[allow(dead_code)]
    pub fn with_capacity(mut self, capacity: f32) -> Self {
        self.capacity = Some(capacity);
        self
    }

    #[allow(dead_code)]
    pub fn with_assigned(
        mut self,
        assigned: HashSet<(Assigned, WorkOrderNumber, ActivityNumber)>,
    ) -> Self {
        self.assigned = assigned;
        self
    }

    #[allow(dead_code)]
    pub fn with_backup_activities(mut self, backup_activities: HashMap<u32, Operation>) -> Self {
        self.backup_activities = Some(backup_activities);
        self
    }

    pub fn build(self) -> OperationalAgent {
        OperationalAgent {
            id_operational: self.id_operational,
            scheduling_environment: self.scheduling_environment,
            operational_algorithm: self.operational_algorithm,
            capacity: self.capacity,
            assigned: self.assigned,
            backup_activities: self.backup_activities,
            operational_configuration: self.operational_configuration,
            supervisor_agent_addr: self.supervisor_agent_addr,
        }
    }
}

impl Handler<OperationalRequestMessage> for OperationalAgent {
    type Result = Result<OperationalResponseMessage, AgentError>;

    fn handle(
        &mut self,
        request: OperationalRequestMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        match request {
            OperationalRequestMessage::Status(_) => {
                let operational_response_status = OperationalStatusResponse::new(
                    self.id_operational.clone(),
                    self.assigned.len(),
                    self.operational_algorithm.objective_value,
                );
                Ok(OperationalResponseMessage::Status(
                    operational_response_status,
                ))
            }
            OperationalRequestMessage::Scheduling(_) => todo!(),
            OperationalRequestMessage::Resource(_) => todo!(),
            OperationalRequestMessage::Time(_) => todo!(),
            OperationalRequestMessage::Test => {
                let operational_algorithm_state = self.determine_algorithm_state();
                Ok(OperationalResponseMessage::Test(
                    operational_algorithm_state,
                ))
            }
        }
    }
}

impl TestAlgorithm for OperationalAgent {
    type InfeasibleCases = OperationalInfeasibleCases;

    fn determine_algorithm_state(&self) -> shared_messages::AlgorithmState<Self::InfeasibleCases> {
        let mut operational_infeasible_cases = OperationalInfeasibleCases::default();
        self.determine_operation_overlap(&mut operational_infeasible_cases);

        if operational_infeasible_cases.all_feasible() {
            AlgorithmState::Feasible
        } else {
            AlgorithmState::Infeasible(operational_infeasible_cases)
        }
    }
}

impl Handler<StatusMessage> for OperationalAgent {
    type Result = String;

    fn handle(&mut self, _msg: StatusMessage, _ctx: &mut Self::Context) -> Self::Result {
        format!(
            "ID: {}, traits: {}, Objective: {}",
            self.id_operational.0,
            self.id_operational
                .1
                .iter()
                .map(|resource| resource.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.operational_algorithm.objective_value
        )
    }
}

impl Handler<StopMessage> for OperationalAgent {
    type Result = ();

    fn handle(&mut self, _msg: StopMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}

impl Handler<UpdateWorkOrderMessage> for OperationalAgent {
    type Result = ();

    fn handle(
        &mut self,
        _update_work_order: UpdateWorkOrderMessage,

        _ctx: &mut Self::Context,
    ) -> Self::Result {
        // todo!();
        warn!("Update 'impl Handler<UpdateWorkOrderMessage> for SupervisorAgent'");
    }
}
