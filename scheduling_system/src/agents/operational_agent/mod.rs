pub mod algorithm;
pub mod operational_events;

use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc, Mutex},
};

use actix::prelude::*;
use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use shared_types::{
    agent_error::AgentError,
    operational::{
        operational_request_scheduling::OperationalSchedulingRequest,
        operational_response_scheduling::{
            EventInfo, JsonAssignment, JsonAssignmentEvents, OperationalSchedulingResponse,
        },
        operational_response_status::OperationalStatusResponse,
        OperationalConfiguration, OperationalInfeasibleCases, OperationalRequestMessage,
        OperationalResponseMessage,
    },
    scheduling_environment::{
        time_environment::day::Day,
        work_order::{
            operation::{ActivityNumber, Work},
            WorkOrderActivity, WorkOrderNumber,
        },
        worker_environment::resources::Id,
    },
    AlgorithmState, ConstraintState, StatusMessage, StopMessage,
};

use shared_types::scheduling_environment::{
    work_order::operation::Operation, SchedulingEnvironment,
};

use tracing::{event, info, instrument, warn, Level};

use crate::agents::{
    operational_agent::algorithm::OperationalParameter, supervisor_agent::delegate::Delegate,
    StateLink, StateLinkWrapper,
};

use self::algorithm::{Assignment, OperationalAlgorithm, OperationalSolution};

use super::traits::TestAlgorithm;
use super::ScheduleIteration;
use super::SetAddr;
use super::StateLinkError;
use super::UpdateWorkOrderMessage;
use super::{supervisor_agent::delegate::AtomicDelegate, traits::LargeNeighborHoodSearch};
use super::{
    supervisor_agent::{algorithm::MarginalFitness, SupervisorAgent},
    tactical_agent::tactical_algorithm::TacticalOperation,
};

#[allow(dead_code)]
pub struct OperationalAgent {
    id_operational: Id,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    operational_algorithm: OperationalAlgorithm,
    capacity: Option<f32>,
    // assigned: HashMap<(WorkOrderNumber, ActivityNumber), Assigned>,
    backup_activities: Option<HashMap<u32, Operation>>,
    operational_configuration: OperationalConfiguration,
    main_supervisor: Option<Addr<SupervisorAgent>>,
    supervisor_agent_addr: HashMap<Id, Addr<SupervisorAgent>>,
}

impl OperationalAgent {
    fn determine_start_and_finish_times(
        &self,
        days: &[(Day, Work)],
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
                    - Duration::seconds(days[0].1.in_seconds() as i64),
            );
            let end_datetime = NaiveDateTime::new(
                end_day,
                self.operational_configuration.off_shift_interval.start
                    + Duration::seconds(days.last().unwrap().1.in_seconds() as i64),
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
        for (_, operational_solution_1) in self
            .operational_algorithm
            .operational_solutions
            .0
            .iter()
            .enumerate()
        {
            for (_, operational_solution_2) in self
                .operational_algorithm
                .operational_solutions
                .0
                .iter()
                .enumerate()
            {
                if operational_solution_1.1.start_time() > operational_solution_2.1.finish_time()
                    && operational_solution_2.1.finish_time()
                        > operational_solution_1.1.start_time()
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
        self.supervisor_agent_addr.iter().for_each(|(_, addr)| {
            addr.do_send(SetAddr::Operational(
                self.id_operational.clone(),
                ctx.address(),
            ));
        });

        let start_event = Assignment::make_unavailable_event(
            algorithm::Unavailability::Beginning,
            &self.operational_configuration.availability,
        );

        let end_event = Assignment::make_unavailable_event(
            algorithm::Unavailability::End,
            &self.operational_configuration.availability,
        );

        let unavailability_start_event = OperationalSolution::new(vec![start_event]);

        let unavailability_end_event = OperationalSolution::new(vec![end_event]);

        self.operational_algorithm.operational_solutions.0.push((
            (WorkOrderNumber(0), ActivityNumber(0)),
            unavailability_start_event,
        ));

        self.operational_algorithm.operational_solutions.0.push((
            (WorkOrderNumber(0), ActivityNumber(0)),
            unavailability_end_event,
        ));

        ctx.notify(ScheduleIteration {})
    }
}

impl Handler<ScheduleIteration> for OperationalAgent {
    type Result = ();

    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut Self::Context) -> Self::Result {
        let mut rng = rand::thread_rng();

        self.operational_algorithm.remove_delegate_drop();
        // This is for testing only. There is a small chance that the supervisor
        // sets a Delegate::Drop in the short time span between the line above
        // and the assert! below
        // assert!(self
        //     .operational_algorithm
        //     .operational_parameters
        //     .no_delegate_drop_or_delegate_done());

        let mut temporary_schedule: OperationalAlgorithm = self.operational_algorithm.clone();

        temporary_schedule.unschedule_random_work_order_activies(&mut rng, 15);

        temporary_schedule.schedule();

        let is_better_schedule = temporary_schedule.calculate_objective_value();

        event!(
            Level::ERROR,
            temp_obj = temporary_schedule.objective_value.load(Ordering::Acquire)
        );

        event!(
            Level::ERROR,
            temp_obj = self
                .operational_algorithm
                .objective_value
                .load(Ordering::Acquire)
        );
        if is_better_schedule {
            self.operational_algorithm = temporary_schedule;
            info!(operational_objective = ?self.operational_algorithm.objective_value);
        };

        ctx.wait(
            tokio::time::sleep(tokio::time::Duration::from_millis(
                dotenvy::var("OPERATIONAL_THROTTLING")
                    .expect("The OPERATIONAL_THROTTLING environment variable should always be set")
                    .parse::<u64>()
                    .expect("The OPERATIONAL_THROTTLING environment variable have to be an u64 compatible type"),
            ))
            .into_actor(self),
        );

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
        main_supervisor: Option<Addr<SupervisorAgent>>,
        supervisor_agent_addr: HashMap<Id, Addr<SupervisorAgent>>,
    ) -> Self {
        Self(OperationalAgent {
            id_operational,
            scheduling_environment,
            operational_algorithm,
            capacity: None,
            backup_activities: None,
            operational_configuration,
            main_supervisor,
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
            main_supervisor: self.0.main_supervisor,
            supervisor_agent_addr: self.0.supervisor_agent_addr,
        }
    }
}

pub struct InitialMessage {
    work_order_activity: WorkOrderActivity,
    delegate: Arc<AtomicDelegate>,
    tactical_operation: Arc<TacticalOperation>,
    marginal_fitness: MarginalFitness,
    supervisor_id: Id,
}

impl InitialMessage {
    pub fn new(
        work_order_activity: WorkOrderActivity,
        delegate: Arc<AtomicDelegate>,
        tactical_operation: Arc<TacticalOperation>,
        marginal_fitness: MarginalFitness,
        supervisor_id: Id,
    ) -> Self {
        Self {
            work_order_activity,
            delegate,
            tactical_operation,
            marginal_fitness,
            supervisor_id,
        }
    }
}

type StrategicMessage = ();
type TacticalMessage = ();
type SupervisorMessage = InitialMessage;
type OperationalMessage = ();

impl
    Handler<
        StateLinkWrapper<StrategicMessage, TacticalMessage, SupervisorMessage, OperationalMessage>,
    > for OperationalAgent
{
    type Result = Result<(), StateLinkError>;

    #[instrument(skip_all)]
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

        assert!(!self
            .operational_algorithm
            .operational_parameters
            .0
            .iter()
            .any(|(_, op)| op.delegated.load(Ordering::Acquire).is_done()));
        event!(
            Level::INFO,
            self.operational_algorithm.operational_parameters =
                self.operational_algorithm.operational_parameters.0.len()
        );
        match state_link {
            StateLink::Strategic(_) => todo!(),
            StateLink::Tactical(_) => todo!(),
            StateLink::Supervisor(initial_message) => {
                let scheduling_environment = self.scheduling_environment.lock().unwrap();

                let operation: &Operation =
                    scheduling_environment.operation(&initial_message.work_order_activity);

                let (start_datetime, end_datetime) = self.determine_start_and_finish_times(
                    &initial_message.tactical_operation.scheduled,
                );

                assert!(operation.work_remaining() > &Some(Work::from(0.0)));

                let operational_parameter = OperationalParameter::new(
                    operation.work_remaining().clone().unwrap(),
                    operation.operation_analytic.preparation_time.clone(),
                    start_datetime,
                    end_datetime,
                    initial_message.delegate.clone(),
                    initial_message.marginal_fitness,
                    initial_message.supervisor_id,
                );

                let replaced_operational_parameter =
                    self.operational_algorithm.insert_optimized_operation(
                        initial_message.work_order_activity,
                        operational_parameter,
                    );

                match replaced_operational_parameter {
                    Some(operational_parameter) => {
                        event!(Level::INFO, operational_parameter = ?operational_parameter, "An OperationalParameter was inserted into the OperationalAgent that was already present. If the WOA is not Delegate::Drop panic!() the thread.");
                        assert!(operational_parameter
                            .delegated
                            .load(Ordering::Acquire)
                            .is_drop());
                    }
                    None => (),
                }

                self.operational_algorithm
                    .history_of_dropped_operational_parameters
                    .insert(initial_message.work_order_activity);

                info!(id = ?self.id_operational, tactical_operation = ?initial_message.tactical_operation);
                Ok(())
            }
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
                let (assign, assess, unassign): (usize, usize, usize) = self
                    .operational_algorithm
                    .operational_parameters
                    .count_delegate_types();
                let operational_response_status = OperationalStatusResponse::new(
                    self.id_operational.clone(),
                    assign,
                    assess,
                    unassign,
                    self.operational_algorithm
                        .objective_value
                        .load(Ordering::Acquire),
                );
                Ok(OperationalResponseMessage::Status(
                    operational_response_status,
                ))
            }
            OperationalRequestMessage::Scheduling(operational_scheduling_request) => {
                match operational_scheduling_request {
                    OperationalSchedulingRequest::OperationalIds => todo!(),
                    OperationalSchedulingRequest::OperationalState(_) => {
                        let mut json_assignments_events: Vec<JsonAssignmentEvents> = vec![];

                        for (work_order_activity, operational_solution) in
                            &self.operational_algorithm.operational_solutions.0
                        {
                            let mut json_assignments = vec![];
                            for assignment in &operational_solution.assignments {
                                let json_assignment = JsonAssignment::new(
                                    assignment.event_type.clone().into(),
                                    assignment.start,
                                    assignment.finish,
                                );
                                json_assignments.push(json_assignment);
                            }

                            let event_info = EventInfo::new(Some(*work_order_activity));
                            let json_assignment_event =
                                JsonAssignmentEvents::new(event_info, json_assignments);
                            json_assignments_events.push(json_assignment_event);
                        }

                        let operational_scheduling_response =
                            OperationalSchedulingResponse::EventList(json_assignments_events);
                        Ok(OperationalResponseMessage::Scheduling(
                            operational_scheduling_response,
                        ))
                    }
                }
            }

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
            "ID: {}, traits: {}, Objective: {:?}",
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
