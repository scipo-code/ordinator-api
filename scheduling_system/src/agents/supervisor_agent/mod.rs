pub mod algorithm;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use shared_types::{
    agent_error::AgentError,
    scheduling_environment::{
        work_order::{operation::ActivityNumber, WorkOrderActivity, WorkOrderNumber},
        worker_environment::resources::Resources,
    },
    supervisor::{
        supervisor_response_scheduling::SupervisorResponseScheduling,
        supervisor_response_status::SupervisorResponseStatus, SupervisorInfeasibleCases,
        SupervisorRequestMessage, SupervisorResponseMessage,
    },
    AlgorithmState, Asset, ConstraintState, StatusMessage, StopMessage,
};

use shared_types::scheduling_environment::worker_environment::resources::Id;
use tracing::{debug, error, event, instrument, span, warn, Level};

use shared_types::scheduling_environment::SchedulingEnvironment;

use self::algorithm::SupervisorAlgorithm;

use super::{
    operational_agent::{algorithm::OperationalObjective, OperationalAgent},
    tactical_agent::{tactical_algorithm::OperationSolution, TacticalAgent},
    traits::{LargeNeighborHoodSearch, TestAlgorithm},
    ScheduleIteration, SetAddr, StateLink, StateLinkError, StateLinkWrapper,
    UpdateWorkOrderMessage,
};

#[allow(dead_code)]
pub struct SupervisorAgent {
    id_supervisor: Id,
    asset: Asset,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub supervisor_algorithm: SupervisorAlgorithm,
    tactical_agent_addr: Addr<TacticalAgent>,
    operational_agent_addrs: HashMap<Id, Addr<OperationalAgent>>,
}

type TransitionSets = (
    HashSet<(WorkOrderNumber, ActivityNumber)>,
    HashSet<(WorkOrderNumber, ActivityNumber)>,
);

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

    #[instrument(skip_all)]
    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut Context<Self>) {
        self.calculate_objective_value();
        for (work_order_activity, operation_solution) in
            &self.supervisor_algorithm.assigned_work_orders
        {
            let mut all_messages: Vec<
                Request<OperationalAgent, StateLinkWrapper<_, _, Delegate, _>>,
            > = vec![];
            // send a message to each relevant agent
            if self
                .supervisor_algorithm
                // He we want to find the work order that is a Delegate::Assign.
                .is_assigned(*work_order_activity)
            {
                continue;
            };

            for (id, operational_addr) in &self.operational_agent_addrs {
                if id.1.contains(&operation_solution.resource) {
                    event!(Level::INFO, work_order_activity = ?work_order_activity);
                    event!(Level::INFO, operational_state = ?self.supervisor_algorithm.operational_state);

                    if self
                        .supervisor_algorithm
                        .operational_state
                        .0
                        .get(&(id.clone(), *work_order_activity))
                        .unwrap() // Counts as an assert! Every OperationalAgent should be in some state at every point in time.
                        .0
                        .is_assess()
                    {
                        continue;
                    }
                    if operation_solution.work_remaining == 0.0 {
                        continue;
                    }
                    let delegate =
                        Delegate::Assess((*work_order_activity, Some(operation_solution.clone())));
                    let state_link = StateLink::Supervisor(delegate.clone());
                    let span = span!(Level::INFO, "delegate_span", state_link = ?state_link);
                    let _enter = span.enter();

                    let state_link_wrapper = StateLinkWrapper::new(state_link, span.clone());

                    all_messages.push(operational_addr.send(state_link_wrapper));

                    let key = (id.clone(), operation_solution.work_order_activity);

                    self.supervisor_algorithm.operational_state.insert_delegate(
                        key.clone(),
                        delegate,
                        None,
                    );
                }
                // self.operational_agent_addrs;
            }

            for message in all_messages {
                ctx.wait(message.into_actor(self).map(|_, _, _| ()))
            }
        }

        self.delegate_assign_and_drop(ctx);

        ctx.wait(tokio::time::sleep(tokio::time::Duration::from_millis(200)).into_actor(self));
        ctx.notify(ScheduleIteration {});
    }
}

#[derive(Debug, Clone)]
pub enum Delegate {
    Assign(WorkOrderActivity),
    Drop(WorkOrderActivity),
    Assess((WorkOrderActivity, Option<OperationSolution>)),
    Fixed,
}

impl Delegate {
    pub fn is_assess(&self) -> bool {
        matches!(self, Self::Assess(_))
    }

    fn is_assign(&self) -> bool {
        matches!(self, Self::Assign(_))
    }

    fn is_drop(&self) -> bool {
        matches!(self, Self::Drop(_))
    }
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
    fn change_state(
        &mut self,
        woa: WorkOrderActivity,
        resource: &Resources,
        transition: TransitionState,
    ) -> Option<WorkOrderActivity> {
        let total_assigned_to_supervisor = self
            .supervisor_algorithm
            .assigned_work_orders
            .iter()
            .fold(0, |acc, e| {
                acc + self
                    .operational_agent_addrs
                    .iter()
                    .filter(|(id, _)| id.1.contains(&e.1.resource))
                    .count()
            });

        let total_operational_state = self.supervisor_algorithm.operational_state.0.len();

        assert_eq!(total_assigned_to_supervisor, total_operational_state);
        assert!(self.supervisor_algorithm.are_states_consistent());
        let delegate = match transition {
            TransitionState::Entering(ref operation_solution) => {
                Delegate::Assess((woa, Some(operation_solution.clone())))
            }
            TransitionState::Leaving => Delegate::Drop(woa),
        };

        assert!(self.supervisor_algorithm.are_states_consistent());
        for (operational_agent, addr) in &self.operational_agent_addrs {
            if operational_agent.1.contains(resource) {
                let state_link = StateLink::Supervisor(delegate.clone());

                let number_woas_per_agent = self
                    .supervisor_algorithm
                    .operational_state
                    .0
                    .iter()
                    .filter(|id| id.0 .0 == *operational_agent);

                let span =
                    span!(Level::DEBUG, "fejl-her", number_woas_per_agent = ?number_woas_per_agent);
                let state_link_wrapper = StateLinkWrapper::new(state_link, span.clone());
                let id_woa = (operational_agent.clone(), woa);
                match transition {
                    TransitionState::Entering(_) => {
                        self.supervisor_algorithm.operational_state.insert_delegate(
                            id_woa,
                            delegate.clone(),
                            None,
                        );
                    }
                    TransitionState::Leaving => {
                        self.supervisor_algorithm
                            .operational_state
                            .remove_delegate(&id_woa);
                    }
                }
                addr.do_send(state_link_wrapper);
            }
        }

        dbg!(&transition);
        match transition {
            TransitionState::Entering(operation_solution) => {
                self.supervisor_algorithm
                    .assigned_work_orders
                    .insert(woa, operation_solution);
            }
            TransitionState::Leaving => {
                self.supervisor_algorithm.assigned_work_orders.remove(&woa);
            }
        }
        let total_assigned_to_supervisor = self
            .supervisor_algorithm
            .assigned_work_orders
            .iter()
            .fold(0, |acc, e| {
                acc + self
                    .operational_agent_addrs
                    .iter()
                    .filter(|(id, _)| id.1.contains(&e.1.resource))
                    .count()
            });

        let total_operational_state = self.supervisor_algorithm.operational_state.0.len();

        assert_eq!(total_assigned_to_supervisor, total_operational_state);
        // assert!(self.supervisor_algorithm.are_states_consistent());
        Some(woa)
    }

    fn delegate_assign_and_drop(&mut self, ctx: &mut Context<SupervisorAgent>) {
        for (work_order_activity, operation_solution) in
            &self.supervisor_algorithm.assigned_work_orders
        {
            let mut operational_solution_across_ids: Vec<_> = self
                .supervisor_algorithm
                .operational_state
                .determine_operational_objectives(*work_order_activity);

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
                    let delegate = Delegate::Assign(*work_order_activity);

                    let state_link = StateLink::Supervisor(delegate.clone());

                    event!(Level::DEBUG, state_link = ?state_link, "Delegate::Assign");

                    let span = span!(Level::INFO, "Delegate_to_operational_agent", state_link = ?state_link);
                    let _enter = span.enter();

                    let state_link_wrapper = StateLinkWrapper::new(state_link, span.clone());

                    self.supervisor_algorithm.operational_state.insert_delegate(
                        (toa.1 .0.clone(), *work_order_activity),
                        delegate,
                        toa.1 .1,
                    );

                    messages_to_operational_agents.push(
                        self.operational_agent_addrs
                            .get(&toa.1 .0)
                            .unwrap()
                            .send(state_link_wrapper),
                    );
                }

                for roa in remaining_operational_agents {
                    if !self
                        .supervisor_algorithm
                        .operational_state
                        .0
                        .get(&(roa.1 .0.clone(), *work_order_activity))
                        .unwrap()
                        .0
                        .is_assess()
                    {
                        continue;
                    }
                    let delegate = Delegate::Drop(*work_order_activity);

                    let state_link = StateLink::Supervisor(delegate.clone());

                    let span = span!(Level::INFO, "Delegate::Drop", state_link = ?state_link);
                    let _enter = span.enter();
                    let message = StateLinkWrapper::new(state_link, span.clone());

                    debug!(message = ?message, "message before the Delegate::Drop by lossing WOA");

                    self.supervisor_algorithm.operational_state.insert_delegate(
                        (roa.1 .0.clone(), *work_order_activity),
                        delegate,
                        roa.1 .1,
                    );

                    messages_to_operational_agents.push(
                        self.operational_agent_addrs
                            .get(&roa.1 .0)
                            .unwrap()
                            .send(message),
                    );
                }

                for message in messages_to_operational_agents {
                    ctx.wait(message.into_actor(self).map(|_, _, _| ()))
                }
            }
        }
    }

    fn generate_sets_of_work_order_activities(
        &self,
        tactical_supervisor_link: HashMap<(WorkOrderNumber, ActivityNumber), OperationSolution>,
    ) -> TransitionSets {
        let supervisor_set: HashSet<(WorkOrderNumber, ActivityNumber)> = self
            .supervisor_algorithm
            .assigned_work_orders
            .keys()
            .cloned()
            .collect();

        let tactical_set: HashSet<(WorkOrderNumber, ActivityNumber)> = tactical_supervisor_link
            .keys()
            .cloned()
            .collect::<HashSet<_>>();

        let _present_woas = supervisor_set
            .intersection(&tactical_set)
            .cloned()
            .collect::<HashSet<_>>();

        let leaving_woas = supervisor_set
            .difference(&tactical_set)
            .cloned()
            .collect::<HashSet<_>>();

        let entering_woas = tactical_set
            .difference(&supervisor_set)
            .cloned()
            .collect::<HashSet<_>>();

        (entering_woas, leaving_woas)
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

#[derive(Debug)]
enum TransitionState {
    Entering(OperationSolution),
    Leaving,
}

type StrategicMessage = ();
type TacticalMessage = HashMap<(WorkOrderNumber, ActivityNumber), OperationSolution>;
type SupervisorMessage = ();
type OperationalMessage = ((Id, WorkOrderActivity), (Delegate, OperationalObjective));

impl
    Handler<
        StateLinkWrapper<StrategicMessage, TacticalMessage, SupervisorMessage, OperationalMessage>,
    > for SupervisorAgent
{
    type Result = Result<(), StateLinkError>;

    #[instrument(level = "info", skip_all)]
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

        match state_link {
            StateLink::Strategic(_) => Ok(()),
            StateLink::Tactical(tactical_supervisor_link) => {
                let (entering_woas, leaving_woas) =
                    self.generate_sets_of_work_order_activities(tactical_supervisor_link.clone());

                assert!(self.supervisor_algorithm.are_states_consistent());
                assert!(entering_woas.is_disjoint(&leaving_woas));
                assert!(leaving_woas.is_disjoint(
                    &tactical_supervisor_link
                        .keys()
                        .cloned()
                        .collect::<HashSet<_>>()
                ));

                for leaving_woa in leaving_woas {
                    let resource = &self
                        .supervisor_algorithm
                        .assigned_work_orders
                        .get(&leaving_woa)
                        .unwrap()
                        .resource
                        .clone();
                    let success =
                        self.change_state(leaving_woa, resource, TransitionState::Leaving);
                    match success {
                        Some(woa) => event!(Level::DEBUG, woa = ?woa, "SUCCESFUL scheduling"),
                        None => panic!(),
                    }
                }

                for entering_woa in entering_woas {
                    let resource = &tactical_supervisor_link
                        .get(&entering_woa)
                        .unwrap()
                        .resource;
                    let operation_solution = tactical_supervisor_link
                        .get(&entering_woa)
                        .cloned()
                        .unwrap();
                    let success = self.change_state(
                        entering_woa,
                        resource,
                        TransitionState::Entering(operation_solution),
                    );
                    match success {
                        Some(woa) => event!(Level::DEBUG, woa = ?woa, "SUCCESFUL scheduling"),
                        None => panic!(),
                    }
                }

                // assert!(self.supervisor_algorithm.are_states_consistent());

                Ok(())
            }
            StateLink::Supervisor(_) => Ok(()),
            StateLink::Operational(operational_solution) => {
                self.supervisor_algorithm.operational_state.insert_delegate(
                    operational_solution.0,
                    operational_solution.1 .0,
                    Some(operational_solution.1 .1),
                );
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

            SupervisorRequestMessage::Scheduling(scheduling_message) => {
                self.supervisor_algorithm.operational_state.insert_delegate(
                    (
                        scheduling_message.id_operational,
                        scheduling_message.work_order_activity,
                    ),
                    Delegate::Assign(scheduling_message.work_order_activity),
                    None,
                );

                // I think that I will get a lot of issues if I do not handle all state management through the
                // self.operational_state. I think that I should simply go with that data structure and wrap it
                // in a new type and then implement types on that.

                // What does manual scheduling mean here? I think that it means that we should find all delegates
                // in the state set them to Drop expected for the ones given by the
                Ok(SupervisorResponseMessage::Scheduling(
                    SupervisorResponseScheduling {},
                ))
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
        for ((work_order_number, _operation_solution), _) in
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
