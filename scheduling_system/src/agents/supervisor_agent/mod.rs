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
        worker_environment::resources::{MainResources, Resources},
    },
    supervisor::{
        supervisor_response_scheduling::SupervisorResponseScheduling,
        supervisor_response_status::SupervisorResponseStatus, SupervisorInfeasibleCases,
        SupervisorRequestMessage, SupervisorResponseMessage,
    },
    AlgorithmState, Asset, ConstraintState, StatusMessage, StopMessage,
};

use shared_types::scheduling_environment::worker_environment::resources::Id;
use tracing::{debug, error, event, info, instrument, span, warn, Level};

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

pub enum TransitionTypes {
    Entering(WorkOrderActivity),
    Leaving(WorkOrderActivity),
    Present(WorkOrderActivity),
}

type TransitionSets = HashSet<TransitionTypes>;

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

        // self.delegate_assign_and_drop(ctx);

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

    pub(crate) fn is_fixed(&self) -> bool {
        matches!(self, Self::Fixed)
    }
}

#[derive(Debug)]
pub struct DelegateAndId(pub Delegate, pub Id);

impl Message for DelegateAndId {
    type Result = ();
}

impl SupervisorAgent {
    pub fn new(
        id_supervisor: Id,
        asset: Asset,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        tactical_agent_addr: Addr<TacticalAgent>,
    ) -> SupervisorAgent {
        let supervisor_resource = id_supervisor.2.clone().unwrap();
        SupervisorAgent {
            id_supervisor,
            asset,
            scheduling_environment,
            supervisor_algorithm: SupervisorAlgorithm::new(supervisor_resource),
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
        let delegate = match transition {
            TransitionState::Entering(ref operation_solution) => {
                Delegate::Assess((woa, Some(operation_solution.clone())))
            }
            TransitionState::Leaving => Delegate::Drop(woa),
        };

        warn!(resource_in_change_state = ?resource);
        for (operational_agent, addr) in &self.operational_agent_addrs {
            warn!(resource_in_change_state = ?resource);
            if operational_agent.1.contains(resource) {
                let state_link = StateLink::Supervisor(DelegateAndId(
                    delegate.clone(),
                    self.id_supervisor.clone(),
                ));
                warn!(operational_agent = ?operational_agent);
                warn!(resource_in_change_state = ?resource);

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
                        error!(resource = ?resource);
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

        let total_assigned_to_supervisor = self.count_unique_woa();

        let total_operational_state = self.supervisor_algorithm.operational_state.0.len();

        assert_eq!(total_assigned_to_supervisor, total_operational_state);
        // assert!(self.supervisor_algorithm.are_states_consistent());
        Some(woa)
    }

    pub fn count_unique_woa(&self) -> usize {
        self.supervisor_algorithm
            .operational_state
            .0
            .keys()
            .map(|(_, woa)| woa)
            .len()
    }

    fn delegate_assign_and_drop(&mut self, ctx: &mut Context<SupervisorAgent>) {
        for work_order_activity in &self
            .supervisor_algorithm
            .operational_state
            .0
            .keys()
            .map(|(_, woa)| woa)
            .cloned()
            .collect::<Vec<WorkOrderActivity>>()
        {
            let number = self
                .scheduling_environment
                .lock()
                .unwrap()
                .work_orders()
                .inner
                .get(&work_order_activity.0)
                .unwrap()
                .operations
                .get(&work_order_activity.1)
                .unwrap()
                .number();

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
                        .partition(|&(i, _)| i < number as usize);

                assert_eq!(
                    remaining_operational_agents.len() + top_operational_agents.len(),
                    number_of_operational_solutions
                );

                let mut messages_to_operational_agents = vec![];
                for toa in top_operational_agents {
                    let delegate = Delegate::Assign(*work_order_activity);

                    let state_link = StateLink::Supervisor(DelegateAndId(
                        delegate.clone(),
                        self.id_supervisor.clone(),
                    ));

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

                    let state_link = StateLink::Supervisor(DelegateAndId(
                        delegate.clone(),
                        self.id_supervisor.clone(),
                    ));

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
        let supervisor_set: HashSet<(WorkOrderNumber, ActivityNumber)> = self.get_unique_woa();

        let tactical_set: HashSet<(WorkOrderNumber, ActivityNumber)> = tactical_supervisor_link
            .keys()
            .cloned()
            .collect::<HashSet<_>>();

        let present_woas = supervisor_set
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

        (entering_woas, present_woas, leaving_woas)
    }

    fn get_unique_woa(&self) -> HashSet<(WorkOrderNumber, ActivityNumber)> {
        self.supervisor_algorithm
            .operational_state
            .0
            .keys()
            .map(|(_, woa)| woa)
            .cloned()
            .collect()
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

        // let _enter = span.enter();

        match state_link {
            StateLink::Strategic(_) => Ok(()),
            StateLink::Tactical(tactical_supervisor_link) => {
                info!(self.id = ?self.id_supervisor);
                let (entering_woas, present_woas, leaving_woas) =
                    self.generate_sets_of_work_order_activities(tactical_supervisor_link.clone());

                assert_eq!(
                    entering_woas.len() + present_woas.len(),
                    tactical_supervisor_link.len()
                );
                assert!(entering_woas.is_disjoint(&leaving_woas));
                assert!(leaving_woas.is_disjoint(
                    &tactical_supervisor_link
                        .keys()
                        .cloned()
                        .collect::<HashSet<_>>()
                ));

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
                    info!(unique_woas = ?self.count_unique_woa(),
                        id_supervisor = ?self.id_supervisor,
                        entering_woa = ?entering_woa,
                         entering_resource_into_supervisor_agent = ?resource,
                    );
                    match success {
                        Some(woa) => event!(Level::DEBUG, woa = ?woa, "SUCCESFUL scheduling"),
                        None => panic!(),
                    }
                }

                // assert!(self.supervisor_algorithm.are_states_consistent());

                let supervisor_state_len = self.count_unique_woa();
                // assert_eq!(supervisor_state_len, tactical_supervisor_link.len());
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
                    self.count_unique_woa(),
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
        for ((work_order_number, woa), _) in self.supervisor_algorithm.operational_state.0.iter() {
            let work_order_main_resource = work_orders
                .inner
                .get(&woa.0)
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
