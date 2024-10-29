pub mod assert_functions;
pub mod messages;
pub mod tactical_algorithm;

use actix::prelude::*;
use shared_types::agent_error::AgentError;
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use shared_types::scheduling_environment::worker_environment::resources::Id;
use shared_types::tactical::tactical_response_status::TacticalResponseStatus;
use shared_types::tactical::{TacticalRequestMessage, TacticalResponseMessage};
use shared_types::Asset;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{event, instrument, Level};

use crate::agents::tactical_agent::tactical_algorithm::TacticalAlgorithm;
use crate::agents::SetAddr;
use shared_types::scheduling_environment::time_environment::period::Period;
use shared_types::scheduling_environment::SchedulingEnvironment;

use super::strategic_agent::StrategicAgent;
use super::supervisor_agent::SupervisorAgent;
use super::traits::LargeNeighborHoodSearch;
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
    main_supervisor_addr: Option<(Id, Addr<SupervisorAgent>)>,
    _other_supervisor: Option<HashMap<Id, Addr<SupervisorAgent>>>,
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
            main_supervisor_addr: None,
            _other_supervisor: None,
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
        event!(
            Level::DEBUG,
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

        // TODO:
        self.tactical_algorithm.load_and_clone_shared_solution();
        let current_objective_value = self.tactical_algorithm.objective_value;

        self.tactical_algorithm
            .unschedule_random_work_orders(&mut rng, 50);

        self.tactical_algorithm.schedule();

        self.tactical_algorithm.calculate_objective_value();

        if self.tactical_algorithm.objective_value < current_objective_value {
            self.tactical_algorithm
                .make_atomic_pointer_swap_for_with_the_better_tactical_solution();

            event!(Level::INFO, tactical_objective_value = ?self.tactical_algorithm.objective_value);
        };

        ctx.wait(
            tokio::time::sleep(tokio::time::Duration::from_millis(
                dotenvy::var("TACTICAL_THROTTLING")
                    .expect("The TACTICAL_THROTTLING environment variable should always be set")
                    .parse::<u64>()
                    .expect("The TACTICAL_THROTTLING environment variable have to be an u64 compatible type"),
            ))
            .into_actor(self),
        );
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
            StateLink::Strategic(strategic_state) => {
                let work_orders = scheduling_environment_guard.work_orders().clone();
                drop(scheduling_environment_guard);
                self.tactical_algorithm
                    .update_state_based_on_strategic(&work_orders);
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
                self.main_supervisor_addr = Some((id, addr));
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
        event!(
            Level::WARN,
            "Update 'impl Handler<UpdateWorkOrderMessage> for TacticalAgent'"
        );
    }
}
