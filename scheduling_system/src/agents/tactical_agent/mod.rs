pub mod messages;
pub mod tactical_algorithm;

use actix::prelude::*;
use shared_types::agent_error::AgentError;
use shared_types::scheduling_environment::work_order::{WorkOrderActivity, WorkOrderNumber};
use shared_types::scheduling_environment::worker_environment::resources::Id;
use shared_types::tactical::tactical_response_status::TacticalResponseStatus;
use shared_types::tactical::{TacticalRequestMessage, TacticalResponseMessage};
use shared_types::{Asset, SolutionExportMessage};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, instrument, span, warn, Level};

use crate::agents::tactical_agent::tactical_algorithm::{OperationSolution, TacticalAlgorithm};
use crate::agents::SetAddr;
use shared_types::scheduling_environment::time_environment::period::Period;
use shared_types::scheduling_environment::SchedulingEnvironment;

use super::strategic_agent::StrategicAgent;
use super::supervisor_agent::SupervisorAgent;
use super::traits::{LargeNeighborHoodSearch, TestAlgorithm};
use super::{
    ScheduleIteration, StateLink, StateLinkError, StateLinkWrapper, UpdateWorkOrderMessage,
};

#[allow(dead_code)]
pub struct TacticalAgent {
    asset: Asset,
    id_tactical: i32,
    time_horizon: Vec<Period>,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    tactical_algorithm: TacticalAlgorithm,
    strategic_addr: Addr<StrategicAgent>,
    supervisor_addrs: HashMap<Id, Addr<SupervisorAgent>>,
}

impl TacticalAgent {
    pub fn new(
        asset: Asset,
        id_tactical: i32,
        time_horizon: Vec<Period>,
        strategic_addr: Addr<StrategicAgent>,
        tactical_algorithm: TacticalAlgorithm,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    ) -> Self {
        TacticalAgent {
            asset,
            id_tactical,
            time_horizon,
            scheduling_environment: scheduling_environment.clone(),
            tactical_algorithm,
            strategic_addr,
            supervisor_addrs: HashMap::new(),
        }
    }

    pub fn time_horizon(&self) -> &Vec<Period> {
        &self.time_horizon
    }

    pub fn status(&self) -> Result<TacticalResponseStatus, AgentError> {
        Ok(TacticalResponseStatus::new(
            self.id_tactical,
            *self.tactical_algorithm.get_objective_value(),
            self.time_horizon.clone(),
        ))
    }
}

impl Actor for TacticalAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        warn!(
            "TacticalAgent {} has started, sending Its address to the StrategicAgent",
            self.id_tactical
        );
        self.strategic_addr
            .do_send(SetAddr::Tactical(ctx.address()));
        ctx.notify(ScheduleIteration {});
    }
}

impl Handler<ScheduleIteration> for TacticalAgent {
    type Result = ();

    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut Context<Self>) {
        let mut rng = rand::thread_rng();

        let mut temporary_schedule: TacticalAlgorithm = self.tactical_algorithm.clone();

        temporary_schedule.unschedule_random_work_orders(&mut rng, 50);

        temporary_schedule.schedule();

        temporary_schedule.calculate_objective_value();

        if temporary_schedule.get_objective_value() < self.tactical_algorithm.get_objective_value()
        {
            self.tactical_algorithm = temporary_schedule;

            self.supervisor_addrs.iter().for_each(|(id, addr)| {
                let mut work_orders_to_supervisor: HashMap<WorkOrderActivity, OperationSolution> =
                    HashMap::new();
                self.tactical_algorithm
                    .optimized_work_orders()
                    .iter()
                    .for_each(|(work_order_number, optimized_work_order)| {
                        if id.2.as_ref().unwrap() == &optimized_work_order.main_work_center {
                            debug!(main_work_center = ?optimized_work_order.main_work_center);
                            debug!(id_of_supervisor = ?id.2.as_ref());
                            for (acn, os) in optimized_work_order
                                .operation_solutions
                                .as_ref()
                                .unwrap()
                                .clone()
                            {
                                work_orders_to_supervisor.insert((*work_order_number, acn), os);
                            }
                        }
                    });
                info!(work_orders_to_supervisors = ?work_orders_to_supervisor);
                let state_link = StateLink::Tactical(work_orders_to_supervisor);

                let span = span!(Level::INFO, "tactical_supervisor", state_link = ?state_link);
                let _enter = span.enter();
                let state_link_wrapper = StateLinkWrapper::new(state_link, span.clone());

                addr.do_send(state_link_wrapper);
            });
            info!(tactical_objective_value = %self.tactical_algorithm.get_objective_value());
        };

        ctx.notify(ScheduleIteration {});
    }
}

impl Handler<TacticalRequestMessage> for TacticalAgent {
    type Result = Result<TacticalResponseMessage, AgentError>;

    #[instrument(level = "info", skip_all)]
    fn handle(
        &mut self,
        tactical_request: TacticalRequestMessage,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        match tactical_request {
            TacticalRequestMessage::Status(_tactical_status_message) => {
                let status_message = self.status().unwrap();
                Ok(TacticalResponseMessage::Status(status_message))
            }
            TacticalRequestMessage::Scheduling(_tactical_scheduling_message) => {
                todo!()
            }
            TacticalRequestMessage::Resources(tactical_resources_message) => {
                let resource_response = self
                    .tactical_algorithm
                    .update_resources_state(tactical_resources_message)
                    .unwrap();
                Ok(TacticalResponseMessage::Resources(resource_response))
            }
            TacticalRequestMessage::Days(_tactical_time_message) => {
                todo!()
            }
            TacticalRequestMessage::Test => {
                let algorithm_state = self.tactical_algorithm.determine_algorithm_state();
                Ok(TacticalResponseMessage::Test(algorithm_state))
            }
        }
    }
}

type StrategicMessage = Vec<(WorkOrderNumber, Period)>;
type TacticalMessage = ();
type SupervisorMessage = ();
type OperationalMessage = ();

impl
    Handler<
        StateLinkWrapper<
            Vec<(WorkOrderNumber, Period)>,
            TacticalMessage,
            SupervisorMessage,
            OperationalMessage,
        >,
    > for TacticalAgent
{
    // Strategic(Vec<(WorkOrderNumber, Period)>),
    type Result = Result<(), StateLinkError>;

    fn handle(
        &mut self,
        state_link_wrapper: StateLinkWrapper<
            StrategicMessage,
            TacticalMessage,
            SupervisorMessage,
            OperationalMessage,
        >,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();
        let state_link = state_link_wrapper.state_link;
        let _enter = state_link_wrapper.span.enter();

        match state_link {
            // Strategic(Vec<(WorkOrderNumber, Period)>),
            StateLink::Strategic(strategic_state) => {
                let work_orders = scheduling_environment_guard.work_orders().clone();
                drop(scheduling_environment_guard);
                self.tactical_algorithm
                    .update_state_based_on_strategic(&work_orders, strategic_state);
                Ok(())
            }
            StateLink::Tactical(_) => {
                todo!()
            }
            StateLink::Supervisor(_) => {
                todo!()
            }
            StateLink::Operational(_) => {
                todo!()
            }
        }
    }
}

impl Handler<SetAddr> for TacticalAgent {
    type Result = ();

    fn handle(&mut self, msg: SetAddr, _ctx: &mut Context<Self>) {
        match msg {
            SetAddr::Supervisor(id, addr) => {
                self.supervisor_addrs.insert(id, addr);
            }
            _ => {
                println!("The tactical agent received an Addr<T>, where T is not a valid Actor");
                todo!()
            }
        }
    }
}

impl Handler<UpdateWorkOrderMessage> for TacticalAgent {
    type Result = ();

    fn handle(
        &mut self,
        _update_work_order: UpdateWorkOrderMessage,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        // todo!();
        warn!("Update 'impl Handler<UpdateWorkOrderMessage> for TacticalAgent'");
    }
}

impl Handler<SolutionExportMessage> for TacticalAgent {
    type Result = String;

    fn handle(&mut self, _msg: SolutionExportMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let mut tactical_solution = HashMap::new();
        for (work_order_number, optimized_work_order) in
            self.tactical_algorithm.optimized_work_orders()
        {
            let mut tactical_operation_solution = HashMap::new();
            for (activity, operation) in optimized_work_order.operation_solutions.as_ref().unwrap()
            {
                tactical_operation_solution
                    .insert(activity, operation.scheduled.first().unwrap().0.date());
            }
            tactical_solution.insert(work_order_number, tactical_operation_solution);
        }
        serde_json::to_string(&tactical_solution).unwrap()
    }
}
