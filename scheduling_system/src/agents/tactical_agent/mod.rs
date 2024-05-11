pub mod messages;
pub mod tactical_algorithm;

use actix::prelude::*;
use shared_messages::agent_error::AgentError;
use shared_messages::models::worker_environment::resources::Id;
use shared_messages::tactical::TacticalRequestMessage;
use shared_messages::{Asset, SolutionExportMessage};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{info, instrument, warn};

use crate::agents::strategic_agent::ScheduleIteration;
use crate::agents::tactical_agent::tactical_algorithm::TacticalAlgorithm;
use crate::agents::SetAddr;
use shared_messages::models::time_environment::period::Period;
use shared_messages::models::SchedulingEnvironment;

use super::strategic_agent::StrategicAgent;
use super::supervisor_agent::SupervisorAgent;
use super::traits::{AlgorithmState, LargeNeighborHoodSearch, TestAlgorithm};
use super::{StateLink, UpdateWorkOrderMessage};

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
                let mut work_orders_to_supervisor = vec![];
                self.tactical_algorithm
                    .optimized_work_orders()
                    .iter()
                    .for_each(|(work_order_number, optimized_work_order)| {
                        if id.2.as_ref().unwrap() == &optimized_work_order.main_work_center {
                            work_orders_to_supervisor.push((
                                *work_order_number,
                                optimized_work_order
                                    .operation_solutions
                                    .as_ref()
                                    .unwrap()
                                    .clone(),
                            ))
                        }
                    });

                addr.do_send(StateLink::Tactical(work_orders_to_supervisor));
            });
            info!(tactical_objective_value = %self.tactical_algorithm.get_objective_value());
        };

        ctx.notify(ScheduleIteration {});
    }
}

impl Handler<TacticalRequestMessage> for TacticalAgent {
    type Result = Result<String, AgentError>;

    #[instrument(level = "info", skip_all)]
    fn handle(
        &mut self,
        tactical_request: TacticalRequestMessage,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        match tactical_request {
            TacticalRequestMessage::Status(_tactical_status_message) => {
                self.tactical_algorithm.status()
            }
            TacticalRequestMessage::Scheduling(_tactical_scheduling_message) => {
                todo!()
            }
            TacticalRequestMessage::Resources(tactical_resources_message) => self
                .tactical_algorithm
                .update_resources_state(tactical_resources_message),
            TacticalRequestMessage::Days(_tactical_time_message) => {
                todo!()
            }
            TacticalRequestMessage::Test => {
                let algorithm_state = self.tactical_algorithm.determine_algorithm_state();

                match algorithm_state {
                    AlgorithmState::Feasible => Ok(
                        "Tactical Schedule is Feasible (Additional tests may be needed)"
                            .to_string(),
                    ),
                    AlgorithmState::Infeasible(infeasible_cases) => Ok(format!(
                        "Tactical Schedule is Infesible: \n 
                           aggregated_load: {}\n
                           all_scheduled: {}\n
                           earliest_start_day: {}\n
                           respect_period_id: {}\n",
                        infeasible_cases.aggregated_load,
                        infeasible_cases.all_scheduled,
                        infeasible_cases.earliest_start_day,
                        infeasible_cases.respect_period_id,
                    )
                    .to_string()),
                }
            }
        }
    }
}

impl Handler<StateLink> for TacticalAgent {
    type Result = ();

    fn handle(&mut self, msg: StateLink, _ctx: &mut Context<Self>) {
        let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();
        match msg {
            StateLink::Strategic(strategic_state) => {
                let work_orders = scheduling_environment_guard.work_orders().clone();
                drop(scheduling_environment_guard);
                self.tactical_algorithm
                    .update_state_based_on_strategic(&work_orders, strategic_state);
            }
            StateLink::Tactical(_) => {
                todo!()
            }
            StateLink::Supervisor => {
                todo!()
            }
            StateLink::Operational => {
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
        update_work_order: UpdateWorkOrderMessage,
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
