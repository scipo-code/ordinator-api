pub mod algorithm;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use shared_types::{
    agent_error::AgentError,
    scheduling_environment::work_order::{
        operation::ActivityNumber, WorkOrderActivity, WorkOrderNumber,
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

#[derive(Default)]
pub struct SupervisorAlgorithm {
    objective_value: f64,
    assigned_work_orders: HashMap<WorkOrderActivity, OperationSolution>,
    operational_state: OperationalState,
}

/// This type will contain all the relevant information handles to the operational agents
/// Delegation. This means that the code should... I think that it is simple the code should
/// simply be created in such a way that we only need to change the OperaitonalState and then
/// the correct messages will be sent out.
#[derive(Default)]
pub struct OperationalState(
    HashMap<(Id, WorkOrderActivity), (Delegate, Option<OperationalObjective>)>,
);

/// This is a fundamental type.
impl OperationalState {
    fn insert_delegate(&mut self, key: (Id, WorkOrderActivity), delegate: Delegate) {
        let previous_delegate = self.0.insert(key.clone(), (delegate, None));

        match previous_delegate {
            Some(delegate_objective) => {
                assert!(delegate_objective.0.is_drop())
            }
            None => {
                event!(
                    Level::INFO,
                    operational_agent = key.0 .0,
                    "new Delegate::Assess",
                );
            }
        }
    }

    fn number_of_assigned_work_orders(&self) -> HashSet<WorkOrderActivity> {
        self.0
            .iter()
            .filter(|(key, val)| val.0.is_assign())
            .map(|(key, val)| key.1)
            .collect()
    }

    fn is_assigned(&self, work_order_activity: WorkOrderActivity) -> bool {
        self.0
            .iter()
            .any(|(key, val)| work_order_activity == key.1 && val.0.is_assign())
    }

    fn determine_operational_objectives(
        &self,
        work_order_activity: WorkOrderActivity,
    ) -> Vec<(Id, Option<OperationalObjective>)> {
        self.0
            .iter()
            .filter(|(key, val)| key.1 == work_order_activity)
            .map(|(key, val)| (key.0.clone(), val.1))
            .collect()
    }
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
            if !self
                .supervisor_algorithm
                // He we want to find the work order that is a Delegate::Assign.
                .operational_state
                .is_assigned(*work_order_activity)
            {
                for (id, operational_addr) in &self.operational_agent_addrs {
                    if id.1.contains(&operation_solution.resource) {
                        if operation_solution.work_remaining == 0.0 {
                            continue;
                        }
                        let delegate = Delegate::Assess(operation_solution.clone());
                        let state_link = StateLink::Supervisor(delegate.clone());
                        let span = span!(Level::INFO, "delegate_span", state_link = ?state_link);
                        let _enter = span.enter();

                        let state_link_wrapper = StateLinkWrapper::new(state_link, span.clone());

                        all_messages.push(operational_addr.send(state_link_wrapper));

                        let key = (
                            id.clone(),
                            operation_solution.work_order_number,
                            operation_solution.activity_number,
                        );

                        self.supervisor_algorithm
                            .operational_state
                            .insert_delegate(key.clone(), delegate);
                    }
                    // self.operational_agent_addrs;
                }
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
    Assign((WorkOrderNumber, ActivityNumber)),
    Drop((WorkOrderNumber, ActivityNumber)),
    Assess(OperationSolution),
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

    fn delegate_assign_and_drop(&mut self, ctx: &mut Context<SupervisorAgent>) {
        for (work_order_activity, operation_solution) in
            &self.supervisor_algorithm.assigned_work_orders
        {
            let mut operational_solution_across_ids: Vec<_> = self
                .supervisor_algorithm
                .operational_state
                .determine_operational_objectives(work_order_activity);

            if operational_solution_across_ids
                .iter()
                .all(|objectives| objectives.1.is_some())
            {
                operational_solution_across_ids
                    .sort_by(|a, b| a.1.unwrap().partial_cmp(&b.1.unwrap()).unwrap());

                let operational_solution_across_ids = operational_solution_across_ids.iter().rev();

                let number_of_operational_solutions = operational_solution_across_ids.len();
                dbg!(operation_solution.number);
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
                        (
                            toa.1 .0.clone(),
                            work_order_activity.0,
                            work_order_activity.1,
                        ),
                        delegate,
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
                        .get(&(
                            roa.1 .0.clone(),
                            work_order_activity.0,
                            work_order_activity.1,
                        ))
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

                    self.supervisor_algorithm
                        .operational_state
                        .insert_delegate((roa.1 .0.clone(), work_order_activity), delegate);

                    messages_to_operational_agents.push(
                        self.operational_agent_addrs
                            .get(&roa.1 .0)
                            .unwrap()
                            .send(message),
                    );
                }

                self.supervisor_algorithm
                    .assigned_to_operational_agents
                    .insert(work_order_activity);

                for message in messages_to_operational_agents {
                    ctx.wait(message.into_actor(self).map(|_, _, _| ()))
                }
            }
        }
    }

    fn generate_set_for_work_orders(
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

type StrategicMessage = ();
type TacticalMessage = HashMap<(WorkOrderNumber, ActivityNumber), OperationSolution>;
type SupervisorMessage = ();
type OperationalMessage = ((Id, WorkOrderNumber, ActivityNumber), OperationalObjective);

impl
    Handler<
        StateLinkWrapper<StrategicMessage, TacticalMessage, SupervisorMessage, OperationalMessage>,
    > for SupervisorAgent
{
    type Result = Result<(), StateLinkError>;

    #[instrument(level = "info", skip_all, fields(state_link_handler_supervisor = ?state_link_wrapper.state_link))]
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
                // Does the nested structure even make sense here? I think that a better way would be
                // to flatten the structure, but I am not sure of the implications.
                let mut entering_message_count = 0;

                warn!("The SupervisorAgent received a message from the TacticalAgent. If this happens more than once there is a good change the that leaving work orders are different and that Delegate::Drop should be called again.");

                let (entering_woas, leaving_woas) =
                    self.generate_set_for_work_orders(tactical_supervisor_link.clone());

                for leaving_woa in &leaving_woas {
                    warn!("leaving work order");
                    if self
                        .supervisor_algorithm
                        .assigned_work_orders
                        .get(&(leaving_woa.0, leaving_woa.1))
                        .unwrap()
                        .work_remaining
                        == 0.0
                    {
                        continue;
                    }
                    for (operational_agent, addr) in &self.operational_agent_addrs {
                        let resource = &self
                            .supervisor_algorithm
                            .assigned_work_orders
                            .get(leaving_woa)
                            .unwrap()
                            .resource;
                        if operational_agent.1.contains(resource) {
                            warn!("leaving work order");

                            let state_link = StateLink::Supervisor(Delegate::Drop((
                                leaving_woa.0,
                                leaving_woa.1,
                            )));
                            let leaving_message = StateLinkWrapper::new(state_link, span.clone());

                            addr.do_send(leaving_message);
                        }
                    }

                    self.supervisor_algorithm
                        .assigned_work_orders
                        .remove(&(leaving_woa.0, leaving_woa.1));

                    self.supervisor_algorithm
                        .assigned_to_operational_agents
                        .remove(&(leaving_woa.0, leaving_woa.1));
                }
                for entering_woa in &entering_woas {
                    for (operational_agent, addr) in &self.operational_agent_addrs {
                        let resource =
                            &tactical_supervisor_link.get(entering_woa).unwrap().resource;
                        if operational_agent.1.contains(resource) {
                            let operation_solution = tactical_supervisor_link
                                .get(&(entering_woa.0, entering_woa.1))
                                .unwrap();

                            let key = (operational_agent.clone(), entering_woa.0, entering_woa.1);

                            if operation_solution.work_remaining == 0.0 {
                                entering_message_count += 1;
                                continue;
                            }
                            let delegate = Delegate::Assess(operation_solution.clone());
                            self.supervisor_algorithm
                                .operational_state
                                .insert(key.clone(), delegate.clone());
                            let state_link = StateLink::Supervisor(delegate);
                            let assess_message = StateLinkWrapper::new(state_link, span.clone());

                            entering_message_count += 1;
                            addr.do_send(assess_message);
                        }
                    }
                    // What should be done here? I think that the best approach will be to reuse the
                    // OperationalSolution.
                    self.supervisor_algorithm.assigned_work_orders.insert(
                        (entering_woa.0, entering_woa.1),
                        tactical_supervisor_link
                            .get(&(entering_woa.0, entering_woa.1))
                            .unwrap()
                            .clone(),
                    );
                }
                let expected_entering_messages: u32 = entering_woas
                    .iter()
                    .map(|woa| {
                        let mut counter = 0;
                        self.operational_agent_addrs.keys().for_each(|id| {
                            if id
                                .1
                                .contains(&tactical_supervisor_link.get(woa).unwrap().resource)
                            {
                                counter += 1;
                            }
                        });
                        counter
                    })
                    .sum();

                assert_eq!(entering_message_count, expected_entering_messages);
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

            SupervisorRequestMessage::Scheduling(scheduling_message) => {
                self.supervisor_algorithm
                    .assigned_to_operational_agents
                    .insert((
                        scheduling_message.work_order_number,
                        scheduling_message.activity_number,
                    ));

                *self
                    .supervisor_algorithm
                    .operational_state
                    .get_mut(&(
                        scheduling_message.id_operational,
                        scheduling_message.work_order_number,
                        scheduling_message.activity_number,
                    ))
                    .unwrap() = Delegate::Assign((
                    scheduling_message.work_order_number,
                    scheduling_message.activity_number,
                ));

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
