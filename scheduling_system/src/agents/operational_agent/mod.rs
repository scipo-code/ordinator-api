pub mod algorithm;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use shared_types::{
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

use shared_types::scheduling_environment::{
    work_order::operation::Operation, SchedulingEnvironment,
};

use tracing::{error, info, instrument, span, warn, Level};

use crate::agents::{
    operational_agent::algorithm::OperationalParameter, StateLink, StateLinkWrapper,
};

use self::algorithm::{Assignment, OperationalAlgorithm, OperationalSolution};

use super::supervisor_agent::{Delegate, SupervisorAgent};
use super::traits::LargeNeighborHoodSearch;
use super::traits::TestAlgorithm;
use super::ScheduleIteration;
use super::SetAddr;
use super::StateLinkError;
use super::UpdateWorkOrderMessage;

#[allow(dead_code)]
pub struct OperationalAgent {
    id_operational: Id,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    operational_algorithm: OperationalAlgorithm,
    capacity: Option<f32>,
    // assigned: HashMap<(WorkOrderNumber, ActivityNumber), Assigned>,
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

        // ctx.notify(ScheduleIteration {})
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

            for operational_solution in &self
                .operational_algorithm
                .operational_solutions
                .0
                .iter()
                .filter(|vec| vec.2.is_some())
                .collect::<Vec<_>>()
            {
                let state_link = StateLink::Operational((
                    (
                        self.id_operational.clone(),
                        operational_solution.0,
                        operational_solution.1,
                    ),
                    self.operational_algorithm.objective_value,
                ));

                let span = span!(Level::INFO, "operational_agent_span", state_link = ?state_link);

                let state_link_wrapper = StateLinkWrapper::new(state_link, span);

                self.supervisor_agent_addr.do_send(state_link_wrapper);
            }
            info!(operational_objective = %self.operational_algorithm.objective_value);
        };

        ctx.notify(ScheduleIteration {});
    }
}

pub struct OperationalAgentBuilder(OperationalAgent);

impl OperationalAgentBuilder {
    pub fn new(
        id_operational: Id,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        operational_configuration: OperationalConfiguration,
        operational_algorithm: OperationalAlgorithm,
        supervisor_agent_addr: Addr<SupervisorAgent>,
    ) -> Self {
        Self(OperationalAgent {
            id_operational,
            scheduling_environment,
            operational_algorithm,
            capacity: None,
            backup_activities: None,
            operational_configuration,
            supervisor_agent_addr,
        })
    }

    #[allow(dead_code)]
    pub fn with_capacity(mut self, capacity: f32) -> Self {
        self.0.capacity = Some(capacity);
        self
    }

    #[allow(dead_code)]
    pub fn with_backup_activities(mut self, backup_activities: HashMap<u32, Operation>) -> Self {
        self.0.backup_activities = Some(backup_activities);
        self
    }

    pub fn build(self) -> OperationalAgent {
        OperationalAgent {
            id_operational: self.0.id_operational,
            scheduling_environment: self.0.scheduling_environment,
            operational_algorithm: self.0.operational_algorithm,
            capacity: self.0.capacity,
            backup_activities: self.0.backup_activities,
            operational_configuration: self.0.operational_configuration,
            supervisor_agent_addr: self.0.supervisor_agent_addr,
        }
    }
}

type StrategicMessage = ();
type TacticalMessage = ();
type SupervisorMessage = Delegate;
type OperationalMessage = ();

impl
    Handler<
        StateLinkWrapper<StrategicMessage, TacticalMessage, SupervisorMessage, OperationalMessage>,
    > for OperationalAgent
{
    type Result = Result<(), StateLinkError>;

    #[instrument(skip_all, fields(state_link = ?state_link_wrapper.state_link))]
    fn handle(
        &mut self,
        state_link_wrapper: StateLinkWrapper<
            StrategicMessage,
            TacticalMessage,
            SupervisorMessage,
            OperationalMessage,
        >,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let state_link = state_link_wrapper.state_link;
        let span = state_link_wrapper.span;
        let _enter = span.enter();
        match state_link {
            StateLink::Strategic(_) => todo!(),
            StateLink::Tactical(_) => todo!(),
            StateLink::Supervisor(delegate) => match delegate {
                Delegate::Assign((work_order_number, activity_number)) => {
                    let operational_solutions =
                        &mut self.operational_algorithm.operational_solutions;

                    if let Some(operational_solution) = operational_solutions
                        .0
                        .iter_mut()
                        .find(|os| os.0 == work_order_number && os.1 == activity_number)
                    {
                        operational_solution.2.as_mut().unwrap().assigned = true;
                    }
                    Ok(())
                }
                Delegate::Drop((work_order_number, activity_number)) => {
                    if self
                        .operational_algorithm
                        .operational_parameters
                        .keys()
                        .any(|(won, acn)| *won == work_order_number && *acn == activity_number)
                    {
                        error!(work_order_number = ?work_order_number, activity_number = ?activity_number, id_operational = ?self.id_operational);
                        // panic!();
                    }
                    let number_of_os = self.operational_algorithm.operational_parameters.len();
                    self.operational_algorithm
                        .operational_solutions
                        .0
                        .retain(|element| {
                            !(element.0 == work_order_number && element.1 == activity_number)
                        });
                    self.operational_algorithm
                        .operational_parameters
                        .remove(&(work_order_number, activity_number));
                    assert_eq!(
                        self.operational_algorithm.operational_parameters.len(),
                        number_of_os - 1
                    );
                    Ok(())
                }
                Delegate::Assess(operation_solution) => {
                    let scheduling_environment = self.scheduling_environment.lock().unwrap();

                    let operation: &Operation =
                        scheduling_environment.operation(&operation_solution.work_order_activity);

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
                        error!("Actor did not incorporate the right state, but supervisor thought that it did");
                        return Err(StateLinkError(
                            Some(self.id_operational.clone()),
                            Some(operation_solution.work_order_activity),
                        ));
                    };

                    self.operational_algorithm.insert_optimized_operation(
                        operation_solution.work_order_activity,
                        operational_parameter,
                    );
                    info!(id = ?self.id_operational, operation = ?operation_solution);
                    Ok(())
                }
            },
            StateLink::Operational(_) => todo!(),
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
                    self.operational_algorithm.operational_parameters.len(),
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

    fn determine_algorithm_state(&self) -> shared_types::AlgorithmState<Self::InfeasibleCases> {
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
