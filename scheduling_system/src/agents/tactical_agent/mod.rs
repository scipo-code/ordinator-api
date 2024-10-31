pub mod assert_functions;
pub mod messages;
pub mod tactical_algorithm;

use actix::prelude::*;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
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
use super::{ScheduleIteration, StateLink, StateLinkWrapper, UpdateWorkOrderMessage};

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

    pub fn status(&self) -> Result<TacticalResponseStatus> {
        Ok(TacticalResponseStatus::new(
            self.id_tactical,
            self.tactical_algorithm.objective_value(),
            self.time_horizon.clone(),
        ))
    }
}

impl Actor for TacticalAgent {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut actix::Context<Self>) {
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
    type Result = Result<()>;

    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut actix::Context<Self>) -> Self::Result {
        let mut rng = rand::thread_rng();
        self.tactical_algorithm.load_shared_solution();
        let current_tactical_solution = self.tactical_algorithm.tactical_solution.clone();

        event!(Level::INFO, tactical_objective_value = ?current_tactical_solution.objective_value);
        self.tactical_algorithm
            .unschedule_random_work_orders(&mut rng, 50)
            .context("random unschedule failed")
            .expect("Error in the Handler<ScheduleIteration>");

        self.tactical_algorithm.schedule();

        self.tactical_algorithm.calculate_objective_value();

        if self.tactical_algorithm.tactical_solution.objective_value
            < current_tactical_solution.objective_value
        {
            self.tactical_algorithm
                .make_atomic_pointer_swap_for_with_the_better_tactical_solution();

            event!(Level::INFO, tactical_objective_value = ?self.tactical_algorithm.tactical_solution.objective_value);
        } else {
            self.tactical_algorithm.tactical_solution = current_tactical_solution;
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
        Ok(())
    }
}

impl Handler<TacticalRequestMessage> for TacticalAgent {
    type Result = Result<TacticalResponseMessage>;

    #[instrument(level = "info", skip_all)]
    fn handle(
        &mut self,
        tactical_request: TacticalRequestMessage,
        _ctx: &mut actix::Context<Self>,
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
    type Result = Result<()>;

    fn handle(
        &mut self,
        state_link_wrapper: StateLinkWrapper<
            StrategicMessage,
            TacticalMessage,
            SupervisorMessage,
            OperationalMessage,
        >,
        _ctx: &mut actix::Context<Self>,
    ) -> Self::Result {
        let state_link = state_link_wrapper.state_link;
        let _enter = state_link_wrapper.span.enter();

        match state_link {
            StateLink::Strategic(_strategic_state) => Ok(()),
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
    type Result = Result<()>;

    fn handle(&mut self, msg: SetAddr, _ctx: &mut actix::Context<Self>) -> Self::Result {
        match msg {
            SetAddr::Supervisor(id, addr) => {
                self.main_supervisor_addr = Some((id, addr));
                Ok(())
            }
            _ => {
                bail!("The tactical agent received an Addr<T>, where T is not a valid Actor")
            }
        }
    }
}

impl Handler<UpdateWorkOrderMessage> for TacticalAgent {
    type Result = ();

    fn handle(
        &mut self,
        _update_work_order: UpdateWorkOrderMessage,
        _ctx: &mut actix::Context<Self>,
    ) -> Self::Result {
        // todo!();
        event!(
            Level::WARN,
            "Update 'impl Handler<UpdateWorkOrderMessage> for TacticalAgent'"
        );
    }
}
