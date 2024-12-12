pub mod message_handlers;
pub mod algorithm;

use actix::prelude::*;
use anyhow::{Context, Result};
use algorithm::assert_functions::TacticalAssertions;
use shared_types::scheduling_environment::worker_environment::resources::Id;
use shared_types::tactical::tactical_response_status::TacticalResponseStatus;
use shared_types::Asset;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{event, Level};

use crate::agents::tactical_agent::algorithm::TacticalAlgorithm;
use crate::agents::SetAddr;
use shared_types::scheduling_environment::SchedulingEnvironment;

use super::orchestrator::NotifyOrchestrator;
use super::strategic_agent::StrategicAgent;
use super::supervisor_agent::SupervisorAgent;
use super::traits::LargeNeighborHoodSearch;
use super::ScheduleIteration;

pub struct TacticalAgent {
    asset: Asset,
    id_tactical: i32,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    tactical_algorithm: TacticalAlgorithm,
    strategic_addr: Addr<StrategicAgent>,
    main_supervisor_addr: Option<(String, Addr<SupervisorAgent>)>,
    _other_supervisor: Option<HashMap<Id, Addr<SupervisorAgent>>>,
    pub notify_orchestrator: NotifyOrchestrator,
}

impl TacticalAgent {
    pub fn new(
        asset: Asset,
        id_tactical: i32,
        strategic_addr: Addr<StrategicAgent>,
        tactical_algorithm: TacticalAlgorithm,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        notify_orchestrator: NotifyOrchestrator,
    ) -> Self {
        TacticalAgent {
            asset,
            id_tactical,
            scheduling_environment: scheduling_environment.clone(),
            tactical_algorithm,
            strategic_addr,
            main_supervisor_addr: None,
            _other_supervisor: None,
            notify_orchestrator,
        }
    }

    pub fn status(&self) -> Result<TacticalResponseStatus> {
        Ok(TacticalResponseStatus::new(
            self.id_tactical,
            self.tactical_algorithm.objective_value(),
            self.tactical_algorithm.tactical_days.clone(),
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

        self.tactical_algorithm
            .unschedule_random_work_orders(&mut rng, 50)
            .context("random unschedule failed")
            .expect("Error in the Handler<ScheduleIteration>");

        self.tactical_algorithm.schedule().expect("TacticalAlgorithm.schedule method failed");

        let total_excess_hours = self.tactical_algorithm.asset_that_capacity_is_not_exceeded().ok();
        
        if self.tactical_algorithm.calculate_objective_value()
            < current_tactical_solution.objective_value
        {
            self.tactical_algorithm
                .make_atomic_pointer_swap();
            event!(Level::INFO,
                 new_tactical_objective_value = ?self.tactical_algorithm.tactical_solution.objective_value,
                 tactical_objective_value = ?current_tactical_solution.objective_value,
                 difference_in_objective_value = self.tactical_algorithm.tactical_solution.objective_value.0 as i64 - current_tactical_solution.objective_value.0 as i64, 
                 total_excess_hours = ?total_excess_hours,
                 scheduled_work_orders = self
                .tactical_algorithm
                .tactical_solution
                .tactical_scheduled_work_orders.scheduled_work_orders())
        } else {
            event!(Level::INFO,
                 new_tactical_objective_value = ?self.tactical_algorithm.tactical_solution.objective_value,
                 tactical_objective_value = ?current_tactical_solution.objective_value,
                 difference_in_objective_value = self.tactical_algorithm.tactical_solution.objective_value.0 as i64 - current_tactical_solution.objective_value.0 as i64, 
                 total_excess_hours = ?total_excess_hours,
                 scheduled_work_orders = self
                .tactical_algorithm
                .tactical_solution
                .tactical_scheduled_work_orders.scheduled_work_orders(),
                );

            self.tactical_algorithm.tactical_solution = current_tactical_solution;
        };

            event!(Level::INFO,
                 new_tactical_objective_value = ?self.tactical_algorithm.tactical_solution.objective_value,
                 total_excess_hours = ?total_excess_hours,
                 scheduled_work_orders = self
                .tactical_algorithm
                .tactical_solution
                .tactical_scheduled_work_orders.scheduled_work_orders());
 
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
        self.tactical_algorithm
            .asset_that_loading_matches_scheduled()
            .unwrap();
        Ok(())
    }
}


