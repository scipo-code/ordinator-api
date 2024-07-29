pub mod algorithm;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use shared_types::{
    agent_error::AgentError,
    scheduling_environment::{
        work_order::{operation::ActivityNumber, WorkOrderNumber},
        worker_environment::resources::Resources,
    },
    supervisor::{
        supervisor_response_status::SupervisorResponseStatus, SupervisorInfeasibleCases,
        SupervisorRequestMessage, SupervisorResponseMessage,
    },
    AlgorithmState, Asset, ConstraintState, StatusMessage, StopMessage,
};

use shared_types::scheduling_environment::worker_environment::resources::Id;
use tracing::{error, instrument, warn};

use shared_types::scheduling_environment::SchedulingEnvironment;

use super::{
    operational_agent::{algorithm::OperationalObjective, OperationalAgent},
    tactical_agent::{tactical_algorithm::OperationSolution, TacticalAgent},
    traits::{LargeNeighborHoodSearch, TestAlgorithm},
    EnteringState, ScheduleIteration, SetAddr, StateLink, StateLinkError, UpdateWorkOrderMessage,
};

pub struct SupervisorAgent {
    id_supervisor: Id,
    asset: Asset,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub supervisor_algorithm: SupervisorAlgorithm,
    tactical_agent_addr: Addr<TacticalAgent>,
    operational_agent_addrs: HashMap<Id, Addr<OperationalAgent>>,
}

#[derive(Default)]
struct SupervisorAlgorithm {
    objective_value: f64,
    assigned_work_orders: Vec<(WorkOrderNumber, ActivityNumber, OperationSolution)>,
    assigned_to_operational_agents: HashSet<(WorkOrderNumber, ActivityNumber)>,
    operational_solutions:
        HashMap<(Id, WorkOrderNumber, ActivityNumber), Option<OperationalObjective>>,
}

// The type needed here will require significant effort to derive. I think that the best approach
// It should have a list of all assigned WOAs this is a given. I think that the

// Does the SupervisorAgent even need to have the OperationSolution?

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
        self.calculate_objective_value();
        for (work_order_number, activity_number, operation_solution) in
            &self.supervisor_algorithm.assigned_work_orders
        {
            let mut all_messages: Vec<Request<OperationalAgent, StateLink<_, _, Delegate, _>>> =
                vec![];
            // send a message to each relevant agent
            if !self
                .supervisor_algorithm
                .assigned_to_operational_agents
                .contains(&(*work_order_number, *activity_number))
            {
                for (id, operational_addr) in &self.operational_agent_addrs {
                    if id.1.contains(&operation_solution.resource) {
                        all_messages.push(operational_addr.send(StateLink::Supervisor(
                            Delegate::Assess(operation_solution.clone()),
                        )));
                        self.supervisor_algorithm
                            .operational_solutions
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

            for message in all_messages {
                ctx.wait(message.into_actor(self).map(|_, _, _| ()))
            }
        }

        for (work_order_number, activity_number, operation_solution) in
            &self.supervisor_algorithm.assigned_work_orders
        {
            let mut operational_solution_across_ids: Vec<_> = self
                .supervisor_algorithm
                .operational_solutions
                .iter()
                .filter(|(key, _)| key.1 == *work_order_number && key.2 == *activity_number)
                .map(|(key, value)| (&key.0, *value))
                .collect();

            if self
                .supervisor_algorithm
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

                let operational_solution_across_ids = operational_solution_across_ids.iter().rev();

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
                            StateLink::Supervisor(Delegate::Assign((
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

                self.supervisor_algorithm
                    .assigned_to_operational_agents
                    .insert((*work_order_number, *activity_number));

                for message in messages_to_operational_agents {
                    ctx.wait(message.into_actor(self).map(|_, _, _| ()))
                }
            }
        }

        ctx.wait(tokio::time::sleep(tokio::time::Duration::from_millis(200)).into_actor(self));
        ctx.notify(ScheduleIteration {});
    }
}

pub enum Delegate {
    Assign((WorkOrderNumber, ActivityNumber)),
    Drop((WorkOrderNumber, ActivityNumber)),
    Assess(OperationSolution),
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
            supervisor_algorithm: SupervisorAlgorithm::default(),
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

type StrategicMessage = ();
type TacticalMessage = Vec<(WorkOrderNumber, ActivityNumber, OperationSolution)>;
type SupervisorMessage = ();
type OperationalMessage = ((Id, WorkOrderNumber, ActivityNumber), OperationalObjective);

impl Handler<StateLink<StrategicMessage, TacticalMessage, SupervisorMessage, OperationalMessage>>
    for SupervisorAgent
{
    type Result = Result<(), StateLinkError>;

    #[instrument(level = "trace", skip_all)]
    fn handle(
        &mut self,
        state_link: StateLink<
            StrategicMessage,
            TacticalMessage,
            SupervisorMessage,
            OperationalMessage,
        >,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        match state_link {
            StateLink::Strategic(_) => Ok(()),
            StateLink::Tactical(tactical_supervisor_link) => {
                // Does the nested structure even make sense here? I think that a better way would be
                // to flatten the structure, but I am not sure of the implications.

                let supervisor_set: HashSet<(WorkOrderNumber, ActivityNumber, Resources)> = self
                    .supervisor_algorithm
                    .assigned_work_orders
                    .iter()
                    .map(|(won, acn, os)| (*won, *acn, os.resource.clone()))
                    .collect();

                let tactical_set: HashSet<(WorkOrderNumber, ActivityNumber, Resources)> =
                    tactical_supervisor_link
                        .iter()
                        .map(|(won, acn, os)| (*won, *acn, os.resource.clone()))
                        .collect::<HashSet<_>>();

                let present_woas = supervisor_set
                    .intersection(&tactical_set)
                    .collect::<HashSet<_>>();

                let leaving_woas = supervisor_set
                    .difference(&tactical_set)
                    .collect::<HashSet<_>>();

                let entering_woas = tactical_set
                    .difference(&supervisor_set)
                    .collect::<HashSet<_>>();

                for leaving_woa in &leaving_woas {
                    for (operational_agent, addr) in &self.operational_agent_addrs {
                        if operational_agent.1.contains(&leaving_woa.2) {
                            let leaving_message = StateLink::Supervisor(Delegate::Drop((
                                leaving_woa.0,
                                leaving_woa.1,
                            )));
                            addr.do_send(leaving_message);
                        }
                    }

                    self.supervisor_algorithm
                        .assigned_to_operational_agents
                        .remove(&(leaving_woa.0, leaving_woa.1));
                    // Put entering_woas here
                }
                self.supervisor_algorithm.assigned_work_orders = tactical_supervisor_link;
                Ok(())
            }
            StateLink::Supervisor(_) => Ok(()),
            StateLink::Operational(operational_solution) => {
                self.supervisor_algorithm
                    .operational_solutions
                    .insert(operational_solution.0, Some(operational_solution.1));
                Ok(())
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
                    self.supervisor_algorithm.assigned_work_orders.len(),
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
        for (work_order_number, _operation_solution, _) in
            self.supervisor_algorithm.assigned_work_orders.iter()
        {
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
