pub mod algorithm;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use shared_types::{
    agent_error::AgentError,
    scheduling_environment::work_order::{operation::ActivityNumber, WorkOrderNumber},
    supervisor::{
        supervisor_response_status::SupervisorResponseStatus, SupervisorInfeasibleCases,
        SupervisorRequestMessage, SupervisorResponseMessage,
    },
    AlgorithmState, Asset, ConstraintState, StatusMessage, StopMessage,
};

use shared_types::scheduling_environment::worker_environment::resources::Id;
use tracing::{error, instrument, warn};

use shared_types::scheduling_environment::SchedulingEnvironment;

use self::algorithm::SupervisorAlgorithm;

use super::{
    operational_agent::{algorithm::OperationalObjective, OperationalAgent},
    tactical_agent::{tactical_algorithm::OperationSolution, TacticalAgent},
    traits::TestAlgorithm,
    ScheduleIteration, SetAddr, StateLink, UpdateWorkOrderMessage,
};

#[allow(dead_code)]
pub struct SupervisorAgent {
    id_supervisor: Id,
    asset: Asset,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    assigned_work_orders: Vec<(WorkOrderNumber, HashMap<ActivityNumber, OperationSolution>)>,
    operational_solutions:
        HashMap<(Id, WorkOrderNumber, ActivityNumber), Option<OperationalObjective>>,
    assigned_to_operational_agents: HashSet<(WorkOrderNumber, ActivityNumber)>,
    pub supervisor_algorithm: SupervisorAlgorithm,
    tactical_agent_addr: Addr<TacticalAgent>,
    operational_agent_addrs: HashMap<Id, Addr<OperationalAgent>>,
}

impl Actor for SupervisorAgent {
    type Context = Context<Self>;

    #[instrument(level = "trace", skip_all)]
    fn started(&mut self, ctx: &mut Self::Context) {
        self.tactical_agent_addr.do_send(SetAddr::Supervisor(
            self.id_supervisor.clone(),
            ctx.address(),
        ));
        ctx.notify(ScheduleIteration {});
    }
}

impl Handler<ScheduleIteration> for SupervisorAgent {
    type Result = ();

    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut Context<Self>) {
        for (work_order_number, operations) in &self.assigned_work_orders {
            // Sync here

            let mut all_messages: Vec<Request<OperationalAgent, OperationSolution>> = vec![];
            for (activity_number, operation_solution) in operations {
                // send a message to each relevant agent
                if !self
                    .assigned_to_operational_agents
                    .contains(&(*work_order_number, *activity_number))
                {
                    for (id, operational_addr) in &self.operational_agent_addrs {
                        if id.1.contains(&operation_solution.resource) {
                            all_messages.push(operational_addr.send(operation_solution.clone()));
                            self.operational_solutions
                                .entry((
                                    id.clone(),
                                    operation_solution.work_order_number,
                                    operation_solution.activity_number,
                                ))
                                .or_insert(None);
                        }
                        // self.operational_agent_addrs;
                    }
                }
            }

            for message in all_messages {
                ctx.wait(message.into_actor(self).map(|_, _, _| ()))
            }
        }

        for (work_order_number, activities) in &self.assigned_work_orders {
            for (activity_number, operation_solution) in activities {
                let mut operational_solution_across_ids: Vec<_> = self
                    .operational_solutions
                    .iter()
                    .filter(|(key, _)| key.1 == *work_order_number && key.2 == *activity_number)
                    .map(|(key, value)| (&key.0, *value))
                    .collect();

                if self
                    .assigned_to_operational_agents
                    .contains(&(*work_order_number, *activity_number))
                {
                    continue;
                }

                if operational_solution_across_ids
                    .iter()
                    .all(|objectives| objectives.1.is_some())
                {
                    operational_solution_across_ids
                        .sort_by(|a, b| a.1.unwrap().partial_cmp(&b.1.unwrap()).unwrap());

                    let operational_solution_across_ids =
                        operational_solution_across_ids.iter().rev();

                    let number_of_operational_solutions = operational_solution_across_ids.len();
                    let (top_operational_agents, remaining_operational_agents): (Vec<_>, Vec<_>) =
                        operational_solution_across_ids
                            .into_iter()
                            .enumerate()
                            .partition(|&(i, _)| i < operation_solution.number as usize);

                    assert_eq!(
                        remaining_operational_agents.len() + top_operational_agents.len(),
                        number_of_operational_solutions
                    );

                    let mut messages_to_operational_agents = vec![];
                    for toa in top_operational_agents {
                        messages_to_operational_agents.push(
                            self.operational_agent_addrs.get(toa.1 .0).unwrap().send(
                                StateLink::Supervisor(Delegate::Keep((
                                    *work_order_number,
                                    *activity_number,
                                ))),
                            ),
                        );
                    }

                    for roa in remaining_operational_agents {
                        messages_to_operational_agents.push(
                            self.operational_agent_addrs.get(roa.1 .0).unwrap().send(
                                StateLink::Supervisor(Delegate::Drop((
                                    *work_order_number,
                                    *activity_number,
                                ))),
                            ),
                        );
                    }

                    self.assigned_to_operational_agents
                        .insert((*work_order_number, *activity_number));

                    for message in messages_to_operational_agents {
                        ctx.wait(message.into_actor(self).map(|_, _, _| ()))
                    }
                }
            }
        }

        ctx.wait(tokio::time::sleep(tokio::time::Duration::from_millis(200)).into_actor(self));
        ctx.notify(ScheduleIteration {});
    }
}

pub enum Delegate {
    Keep((WorkOrderNumber, ActivityNumber)),
    Drop((WorkOrderNumber, ActivityNumber)),
}

impl Message for Delegate {
    type Result = ();
}

impl SupervisorAgent {
    pub fn new(
        id_supervisor: Id,
        asset: Asset,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        tactical_agent_addr: Addr<TacticalAgent>,
    ) -> SupervisorAgent {
        SupervisorAgent {
            id_supervisor,
            asset,
            scheduling_environment,
            assigned_work_orders: Vec::new(),
            operational_solutions: HashMap::new(),
            assigned_to_operational_agents: HashSet::new(),
            supervisor_algorithm: SupervisorAlgorithm::new(),
            tactical_agent_addr,
            operational_agent_addrs: HashMap::new(),
        }
    }
}

impl Handler<StatusMessage> for SupervisorAgent {
    type Result = String;

    #[instrument(level = "trace", skip_all)]
    fn handle(&mut self, _msg: StatusMessage, _ctx: &mut Self::Context) -> Self::Result {
        format!(
            "ID: {}, Work Center: {:?}, Main Work Center: {:?}",
            self.id_supervisor.0, self.id_supervisor.1, self.id_supervisor.2
        )
    }
}

impl Handler<StopMessage> for SupervisorAgent {
    type Result = ();

    fn handle(&mut self, _msg: StopMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}

impl Handler<SetAddr> for SupervisorAgent {
    type Result = ();

    #[instrument(level = "trace", skip_all)]
    fn handle(&mut self, msg: SetAddr, _ctx: &mut Self::Context) {
        if let SetAddr::Operational(id, addr) = msg {
            self.operational_agent_addrs.insert(id, addr);
        }
    }
}

impl Handler<StateLink> for SupervisorAgent {
    type Result = ();

    #[instrument(level = "trace", skip_all)]
    fn handle(&mut self, state_link: StateLink, _ctx: &mut Self::Context) {
        match state_link {
            StateLink::Strategic(_) => {}
            StateLink::Tactical(tactical_supervisor_link) => {
                self.assigned_work_orders = tactical_supervisor_link;
            }
            StateLink::Supervisor(_) => {}
            StateLink::Operational(operational_solution) => {
                self.operational_solutions
                    .insert(operational_solution.0, Some(operational_solution.1));
            }
        }
    }
}

impl Handler<UpdateWorkOrderMessage> for SupervisorAgent {
    type Result = ();

    fn handle(
        &mut self,
        _update_work_order: UpdateWorkOrderMessage,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        // todo!()
        warn!("Updateimpl Handler<UpdateWorkOrderMessage> for SupervisorAgent should be implemented for the supervisor agent");
    }
}

impl Handler<SupervisorRequestMessage> for SupervisorAgent {
    type Result = Result<SupervisorResponseMessage, AgentError>;

    #[instrument(level = "trace", skip_all)]
    fn handle(
        &mut self,
        supervisor_request_message: SupervisorRequestMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        tracing::info!(
            "Received SupervisorRequestMessage: {:?}",
            supervisor_request_message
        );

        match supervisor_request_message {
            SupervisorRequestMessage::Status(supervisor_status_message) => {
                tracing::info!(
                    "Received SupervisorStatusMessage: {:?}",
                    supervisor_status_message
                );
                let supervisor_status = SupervisorResponseStatus::new(
                    self.id_supervisor.clone().2.unwrap(),
                    self.assigned_work_orders.len(),
                    self.supervisor_algorithm.objective_value,
                );

                Ok(SupervisorResponseMessage::Status(supervisor_status))
            }
            SupervisorRequestMessage::Test => {
                let algorithm_state = self.determine_algorithm_state();

                let supervisor_test = SupervisorResponseMessage::Test(algorithm_state);
                Ok(supervisor_test)
            }
        }
    }
}
impl TestAlgorithm for SupervisorAgent {
    type InfeasibleCases = SupervisorInfeasibleCases;

    fn determine_algorithm_state(&self) -> AlgorithmState<Self::InfeasibleCases> {
        let mut supervisor_state = SupervisorInfeasibleCases::default();

        let mut feasible_main_resources: bool = true;
        let work_orders = self
            .scheduling_environment
            .lock()
            .unwrap()
            .work_orders()
            .clone();
        for (work_order_number, _operation_solution) in self.assigned_work_orders.iter() {
            let work_order_main_resource = work_orders
                .inner
                .get(work_order_number)
                .unwrap()
                .main_work_center
                .clone();
            if &work_order_main_resource == self.id_supervisor.2.as_ref().unwrap() {
                continue;
            } else {
                error!(work_order_number = ?work_order_number, work_order_main_resource = ?work_order_main_resource, supervisor_trait = ?self.id_supervisor.2.as_ref().unwrap());
                feasible_main_resources = false;
                break;
            }
        }
        if feasible_main_resources {
            supervisor_state.respect_main_work_center = ConstraintState::Feasible;
        }

        AlgorithmState::Infeasible(supervisor_state)
    }
}
