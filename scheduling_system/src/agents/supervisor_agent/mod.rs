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
use tracing::{error, info, instrument, warn};

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
    supervisor_id: Id,
    asset: Asset,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub supervisor_algorithm: SupervisorAlgorithm,
    tactical_agent_addr: Addr<TacticalAgent>,
    operational_agent_addrs: HashMap<Id, Addr<OperationalAgent>>,
}

#[derive(Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum TransitionTypes {
    Entering(Delegate),
    Leaving(Delegate),
    Unchanged(Delegate),
    Changed(Delegate),
}

impl TransitionTypes {
    pub fn resource(&self) -> &Resources {
        match self {
            TransitionTypes::Entering(delegate) => delegate.get_resource(),
            TransitionTypes::Leaving(delegate) => delegate.get_resource(),
            TransitionTypes::Unchanged(delegate) => delegate.get_resource(),
            TransitionTypes::Changed(delegate) => delegate.get_resource(),
        }
    }
}

type TransitionSets = HashSet<TransitionTypes>;

impl Actor for SupervisorAgent {
    type Context = Context<Self>;

    #[instrument(level = "trace", skip_all)]
    fn started(&mut self, ctx: &mut Self::Context) {
        self.tactical_agent_addr.do_send(SetAddr::Supervisor(
            self.supervisor_id.clone(),
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

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug, Clone)]
pub enum Delegate {
    Assess((WorkOrderActivity, OperationSolution)),
    Assign((WorkOrderActivity, OperationSolution)),
    Drop(WorkOrderActivity),
    Fixed,
}

impl Delegate {
    pub fn operation_solution(&self) -> &OperationSolution {
        match self {
            Delegate::Assess((_, os)) => os,
            Delegate::Assign((_, os)) => os,
            Delegate::Drop(_) => panic!(),
            Delegate::Fixed => panic!(),
        }
    }

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

    fn get_woa(&self) -> (WorkOrderNumber, ActivityNumber) {
        match self {
            Delegate::Assign((woa, _)) => *woa,
            Delegate::Assess((woa, _)) => *woa,
            Delegate::Drop(woa) => *woa,
            Delegate::Fixed => panic!(),
        }
    }

    fn get_resource(&self) -> &Resources {
        match self {
            Delegate::Assess((_, os)) => &os.resource,
            Delegate::Assign((_, os)) => &os.resource,
            Delegate::Drop(_) => panic!(),
            Delegate::Fixed => panic!(),
        }
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
            supervisor_id: id_supervisor,
            asset,
            scheduling_environment,
            supervisor_algorithm: SupervisorAlgorithm::new(supervisor_resource),
            tactical_agent_addr,
            operational_agent_addrs: HashMap::new(),
        }
    }
    fn update_operational_state(
        &mut self,
        transition_type: TransitionTypes,
        resource: &Resources,
        operation_solution: OperationSolution,
    ) -> Option<TransitionTypes> {
        for operational_agent in &self.operational_agent_addrs {
            if operational_agent.0 .1.contains(resource) {
                self.supervisor_algorithm.operational_state.handle_woa(
                    transition_type.clone(),
                    operational_agent,
                    self.supervisor_id.clone(),
                )
            }
        }

        let total_assigned_to_supervisor = self
            .supervisor_algorithm
            .operational_state
            .count_unique_woa();

        let total_operational_state = self.supervisor_algorithm.operational_state.len();

        assert_eq!(total_assigned_to_supervisor, total_operational_state);
        // assert!(self.supervisor_algorithm.are_states_consistent());
        Some(transition_type)
    }

    /// This whole function should be moved. It is important, but it is also circumventing the API defined on
    /// OperationalState and that is not allowed. The question is whether we should delete this or not!
    ///
    /// We should delete it! There are multiple errors in here and I think that the best approach is to
    /// pass everything into the program instead of locking the scheduling environment. That is really
    /// bad practice.J

    fn make_transition_sets_from_tactical_state_link(
        &self,
        tactical_supervisor_link: HashMap<(WorkOrderNumber, ActivityNumber), OperationSolution>,
    ) -> TransitionSets {
        let supervisor_set: HashSet<(WorkOrderNumber, ActivityNumber)> =
            self.supervisor_algorithm.operational_state.get_unique_woa();

        let tactical_set: HashSet<(WorkOrderNumber, ActivityNumber)> = tactical_supervisor_link
            .keys()
            .cloned()
            .collect::<HashSet<_>>();

        let mut changed_woas = HashSet::new();

        let mut unchanged_woas = HashSet::new();

        supervisor_set
            .intersection(&tactical_set)
            .cloned()
            .map(|woa| {
                // We should go into the operational_state and find all matches on the
                // woa! Good this is the first step!
                let delegates = self
                    .supervisor_algorithm
                    .operational_state
                    .get_iter()
                    .filter(|(key, value)| key.1 == woa)
                    .collect::<Vec<_>>();

                delegates.iter().for_each(|delegate| {
                    if delegate.1 .0.operation_solution()
                        == tactical_supervisor_link.get(&woa).unwrap()
                    {
                        let transition_type = TransitionTypes::Unchanged(delegate.1 .0.clone());
                        unchanged_woas.insert(transition_type);
                    } else {
                        let transition_type = TransitionTypes::Changed(delegate.1 .0.clone());
                        changed_woas.insert(transition_type);
                    }
                })

                // TransitionTypes::Unchanged(woa))
            });

        let leaving_woas = supervisor_set
            .difference(&tactical_set)
            .cloned()
            .map(|woa| {
                let delegate_drop = Delegate::Drop(woa);
                TransitionTypes::Leaving(delegate_drop)
            })
            .collect::<HashSet<TransitionTypes>>();

        let entering_woas = tactical_set
            .difference(&supervisor_set)
            .cloned()
            .map(|woa| {
                let operation_solution = tactical_supervisor_link.get(&woa).unwrap();
                let delegate_entering = Delegate::Assess((woa, operation_solution.clone()));
                TransitionTypes::Entering(delegate_entering)
            })
            .collect::<HashSet<TransitionTypes>>();

        let entering_present = entering_woas
            .union(&unchanged_woas)
            .cloned()
            .collect::<HashSet<TransitionTypes>>();
        entering_present.union(&leaving_woas).cloned().collect()
    }
}

impl Handler<StatusMessage> for SupervisorAgent {
    type Result = String;

    #[instrument(level = "trace", skip_all)]
    fn handle(&mut self, _msg: StatusMessage, _ctx: &mut Self::Context) -> Self::Result {
        format!(
            "ID: {}, Work Center: {:?}, Main Work Center: {:?}",
            self.supervisor_id.0, self.supervisor_id.1, self.supervisor_id.2
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
type OperationalMessage = ((Id, WorkOrderActivity), OperationalObjective);

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
                info!(self.id = ?self.supervisor_id);
                let transition_sets = self.make_transition_sets_from_tactical_state_link(
                    tactical_supervisor_link.clone(),
                );

                for transition_type in &transition_sets {
                    for operational_agent in &self.operational_agent_addrs {
                        if operational_agent.0 .1.contains(transition_type.resource()) {
                            self.supervisor_algorithm.operational_state.handle_woa(
                                transition_type.clone(),
                                operational_agent,
                                self.supervisor_id.clone(),
                            )
                        }
                    }
                }
                Ok(())
            }
            StateLink::Supervisor(_) => Ok(()),
            StateLink::Operational(operational_solution) => Ok(()),
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
                    self.supervisor_id.clone().2.unwrap(),
                    self.supervisor_algorithm
                        .operational_state
                        .count_unique_woa(),
                    self.supervisor_algorithm.objective_value(),
                );

                Ok(SupervisorResponseMessage::Status(supervisor_status))
            }

            SupervisorRequestMessage::Scheduling(scheduling_message) => Ok(
                SupervisorResponseMessage::Scheduling(SupervisorResponseScheduling {}),
            ),
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

        for ((work_order_number, woa), _) in self.supervisor_algorithm.operational_state.get_iter()
        {
            let work_order_main_resource = work_orders
                .inner
                .get(&woa.0)
                .unwrap()
                .main_work_center
                .clone();

            if &work_order_main_resource == self.supervisor_id.2.as_ref().unwrap() {
                continue;
            } else {
                error!(work_order_number = ?work_order_number, work_order_main_resource = ?work_order_main_resource, supervisor_trait = ?self.supervisor_id.2.as_ref().unwrap());
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
