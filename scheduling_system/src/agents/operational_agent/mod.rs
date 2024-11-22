pub mod algorithm;
pub mod assert_functions;
pub mod message_handlers;

use algorithm::{
    operational_parameter::OperationalParameter,
    operational_solution::{Assignment, OperationalAssignment},
};
use anyhow::{Context, Result};
use assert_functions::OperationalAssertions;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use shared_types::operational::OperationalConfiguration;
use shared_types::scheduling_environment::work_order::operation::ActivityNumber;
use shared_types::scheduling_environment::work_order::{
    operation::Work, WorkOrderActivity, WorkOrderNumber,
};
use shared_types::scheduling_environment::worker_environment::resources::Id;

use shared_types::scheduling_environment::{
    work_order::operation::Operation, SchedulingEnvironment,
};

use tracing::{event, Level};

use crate::agents::{supervisor_agent::algorithm::delegate::Delegate, OperationalSolution};

use self::algorithm::OperationalAlgorithm;

use super::supervisor_agent::SupervisorAgent;
use super::traits::LargeNeighborHoodSearch;
use super::ScheduleIteration;
use super::SetAddr;

pub struct OperationalAgent {
    operational_id: Id,
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
    pub fn create_operational_parameter(
        &mut self,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<()> {
        let scheduling_environment = self.scheduling_environment.lock().unwrap();

        let operation: &Operation = scheduling_environment.operation(work_order_activity);

        assert!(
            operation.work_remaining() > &Some(Work::from(0.0))
                || self
                    .operational_algorithm
                    .loaded_shared_solution
                    .supervisor
                    .operational_state_machine
                    .get(&(self.operational_id.clone(), *work_order_activity))
                    .unwrap()
                    .is_done()
        );

        // TODO: move this around
        let operational_parameter = OperationalParameter::new(
            operation.work_remaining().clone().unwrap(),
            operation.operation_analytic.preparation_time.clone(),
        );

        self.operational_algorithm
            .insert_operational_parameter(*work_order_activity, operational_parameter);

        self.operational_algorithm
            .history_of_dropped_operational_parameters
            .insert(*work_order_activity);

        event!(Level::INFO, id = ?self.operational_id, tactical_operation = ?self.operational_algorithm.loaded_shared_solution.tactical.tactical_days.get(&work_order_activity.0).unwrap());
        Ok(())
    }
}

impl Actor for OperationalAgent {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.supervisor_agent_addr.iter().for_each(|(_, addr)| {
            addr.do_send(SetAddr::Operational(
                self.operational_id.clone(),
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

        let unavailability_start_event = OperationalAssignment::new(vec![start_event]);

        let unavailability_end_event = OperationalAssignment::new(vec![end_event]);

        self.operational_algorithm
            .operational_solution
            .work_order_activities
            .push((
                (WorkOrderNumber(0), ActivityNumber(0)),
                unavailability_start_event,
            ));

        self.operational_algorithm
            .operational_solution
            .work_order_activities
            .push((
                (WorkOrderNumber(0), ActivityNumber(0)),
                unavailability_end_event,
            ));

        ctx.notify(ScheduleIteration {})
    }
}

impl Handler<ScheduleIteration> for OperationalAgent {
    type Result = Result<()>;

    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut Self::Context) -> Self::Result {
        let mut rng = rand::thread_rng();

        self.operational_algorithm.load_shared_solution();

        event!(Level::DEBUG,
            operational_view_in_supervisor_solution = ?self.operational_algorithm.loaded_shared_solution.supervisor
        );

        event!(
            Level::DEBUG,
            number_of_operational_delegates = ?self
                .operational_algorithm
                .loaded_shared_solution
                .supervisor
                .state_of_agent(&self.operational_id)
        );

        let loaded_supervisor_solution =
            &self.operational_algorithm.loaded_shared_solution.supervisor;

        for (work_order_activity, delegate) in
            loaded_supervisor_solution.state_of_agent(&self.operational_id)
        {
            if delegate == Delegate::Done {
                continue;
            }
            self.create_operational_parameter(&work_order_activity)
                .expect("Could not create OperationalParameter");
        }

        self.operational_algorithm
            .remove_delegate_drop(&self.operational_id);

        event!(
            Level::DEBUG,
            operational_solutions = self
                .operational_algorithm
                .operational_solution
                .work_order_activities
                .len(),
            operational_parameters = self
                .operational_algorithm
                .operational_parameters
                .work_order_parameters
                .len()
        );

        let temporary_operational_solution: OperationalSolution =
            self.operational_algorithm.operational_solution.clone();

        self.operational_algorithm
            .unschedule_random_work_order_activies(&mut rng, 15);

        self.operational_algorithm
            .schedule()
            .expect("Operational.schedule() method failed");

        let is_better_schedule = self.operational_algorithm.calculate_objective_value();

        if is_better_schedule {
            self.operational_algorithm
                .make_atomic_pointer_swap(&self.operational_id);
        } else {
            self.operational_algorithm.operational_solution = temporary_operational_solution;

            event!(Level::INFO, operational_objective_value = ?self.operational_algorithm.operational_solution.objective_value);
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
        self.assert_no_operation_overlap()
            .with_context(|| {
                format!(
                    "OperationalAgent: {} is having overlaps in his state",
                    self.operational_id
                )
            })
            .expect("");
        ctx.notify(ScheduleIteration {});
        Ok(())
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
            operational_id: id_operational,
            scheduling_environment,
            operational_algorithm,
            capacity: None,
            backup_activities: None,
            operational_configuration,
            main_supervisor,
            supervisor_agent_addr,
        })
    }

    pub fn build(self) -> OperationalAgent {
        OperationalAgent {
            operational_id: self.0.operational_id,
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
