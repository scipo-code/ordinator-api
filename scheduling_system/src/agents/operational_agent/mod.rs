pub mod algorithm;
use std::{
    collections::{HashMap, HashSet},
    ops::RangeBounds,
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use chrono::{DateTime, Duration, NaiveDateTime, NaiveTime, TimeZone, Utc};
use shared_messages::{
    agent_error::AgentError,
    models::{
        time_environment::day::Day,
        work_order::{operation::ActivityNumber, WorkOrderNumber},
        worker_environment::resources::Id,
    },
    operational::{
        operational_response_status::OperationalStatusResponse, OperationalRequestMessage,
        OperationalResponseMessage,
    },
    StatusMessage, StopMessage,
};

use shared_messages::models::{work_order::operation::Operation, SchedulingEnvironment};
use tracing::{info, warn};

use crate::agents::{
    operational_agent::algorithm::{OperationalParameter, OperationalSolution},
    tactical_agent::tactical_algorithm::OperationParameters,
};

use self::algorithm::OperationalAlgorithm;

use super::{
    supervisor_agent::SupervisorAgent, tactical_agent::tactical_algorithm::OperationSolution,
    SetAddr, UpdateWorkOrderMessage,
};

pub struct OperationalAgent {
    id_operational: Id,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    operational_algorithm: OperationalAlgorithm,
    capacity: Option<f32>,
    availability: Option<Vec<Availability>>,
    assigned: HashSet<(Assigned, WorkOrderNumber, ActivityNumber)>,
    backup_activities: Option<HashMap<u32, Operation>>,
    shift: (NaiveTime, NaiveTime),
    supervisor_agent_addr: Addr<SupervisorAgent>,
}

pub struct Availability {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

pub struct AssignedWork {
    work: f64,
    assigned: bool,
    operation_solution: OperationSolution,
}

type Assigned = bool;
impl OperationalAgent {
    fn determine_start_and_finish_times(
        &self,
        days: &Vec<(Day, f64)>,
    ) -> (DateTime<Utc>, DateTime<Utc>) {
        if days.len() == 1 {
            let start_of_time_window = Utc.from_utc_datetime(&NaiveDateTime::new(
                days.first().unwrap().0.date().date_naive(),
                self.shift.0,
            ));
            let end_of_time_window = Utc.from_utc_datetime(&NaiveDateTime::new(
                days.last().unwrap().0.date().date_naive(),
                self.shift.1,
            ));
            (start_of_time_window, end_of_time_window)
        } else {
            let start_day = days[0].0.date().date_naive();
            let end_day = days.last().unwrap().0.date().date_naive();
            let start_datetime = NaiveDateTime::new(
                start_day,
                self.shift.1 - Duration::seconds(3600 * days[0].1.round() as i64),
            );
            let end_datetime = NaiveDateTime::new(
                end_day,
                self.shift.0 + Duration::seconds(3600 * days.last().unwrap().1.round() as i64),
            );

            (
                Utc.from_utc_datetime(&start_datetime),
                Utc.from_utc_datetime(&end_datetime),
            )
        }
    }
}

impl Actor for OperationalAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.supervisor_agent_addr.do_send(SetAddr::Operational(
            self.id_operational.clone(),
            ctx.address(),
        ));
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

        let operational_parameter = OperationalParameter::new(
            operation.work_remaining(),
            operation.operation_analytic.preparation_time,
            start_datetime,
            end_datetime,
        );

        self.assigned.insert((
            false,
            operation_solution.work_order_number,
            operation_solution.activity_number,
        ));

        let operational_solution = OperationalSolution::new(false, Vec::new());

        self.operational_algorithm.insert_optimized_operation(
            operation_solution.work_order_number,
            operation_solution.activity_number,
            operational_parameter,
            operational_solution,
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
    availability: Option<Vec<Availability>>,
    assigned: HashSet<(Assigned, WorkOrderNumber, ActivityNumber)>,
    backup_activities: Option<HashMap<u32, Operation>>,
    shift: (NaiveTime, NaiveTime),
    supervisor_agent_addr: Addr<SupervisorAgent>,
}

impl OperationalAgentBuilder {
    pub fn new(
        id_operational: Id,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        shift: (NaiveTime, NaiveTime),
        supervisor_agent_addr: Addr<SupervisorAgent>,
    ) -> Self {
        OperationalAgentBuilder {
            id_operational,
            scheduling_environment,
            operational_algorithm: OperationalAlgorithm::new(),
            capacity: None,
            availability: None,
            assigned: HashSet::new(),
            backup_activities: None,
            shift,
            supervisor_agent_addr,
        }
    }

    #[allow(dead_code)]
    pub fn with_capacity(mut self, capacity: f32) -> Self {
        self.capacity = Some(capacity);
        self
    }

    #[allow(dead_code)]
    pub fn with_availability(mut self, availability: Vec<Availability>) -> Self {
        self.availability = Some(availability);
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
            availability: self.availability,
            assigned: self.assigned,
            backup_activities: self.backup_activities,
            shift: self.shift,
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
                .2
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
