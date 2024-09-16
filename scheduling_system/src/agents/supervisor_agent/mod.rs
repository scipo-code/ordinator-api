pub mod algorithm;
pub mod delegate;

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use shared_types::{
    agent_error::AgentError,
    scheduling_environment::{
        work_order::{
            operation::{ActivityNumber, Work},
            WorkOrderActivity, WorkOrderNumber,
        },
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
use tracing::{error, event, info, instrument, warn, Level};

use shared_types::scheduling_environment::SchedulingEnvironment;

use self::algorithm::SupervisorAlgorithm;

use super::{
    operational_agent::{algorithm::OperationalObjective, OperationalAgent},
    tactical_agent::{tactical_algorithm::TacticalOperation, TacticalAgent},
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
    Entering((WorkOrderActivity, Arc<TacticalOperation>)),
    Leaving(WorkOrderActivity),
    Unchanged(WorkOrderActivity),
    Changed((WorkOrderActivity, Arc<TacticalOperation>)),
    Done(WorkOrderActivity),
}

impl TransitionTypes {
    pub fn resource(&self) -> &Resources {
        match self {
            TransitionTypes::Entering((_,tac)) => (**tac).get_resource(),
            TransitionTypes::Leaving(_) => panic!(),
            TransitionTypes::Unchanged(_) => panic!(),
            TransitionTypes::Changed((_, tac)) => (**tac).get_resource(),
            TransitionTypes::Done(_) => panic!(),
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

        //self.delegate_assign_and_drop(ctx);

        ctx.wait(tokio::time::sleep(tokio::time::Duration::from_millis(200)).into_actor(self));
        ctx.notify(ScheduleIteration {});
    }
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

    /// This whole function should be moved. It is important, but it is also circumventing the API defined on
    /// OperationalState and that is not allowed. The question is whether we should delete this or not!
    ///
    /// We should delete it! There are multiple errors in here and I think that the best approach is to
    /// pass everything into the program instead of locking the scheduling environment. That is really
    /// bad practice.J
    fn make_transition_sets_from_tactical_state_link(
        &self,
        tactical_supervisor_link: HashMap<
            (WorkOrderNumber, ActivityNumber),
            Arc<TacticalOperation>,
        >,
    ) -> TransitionSets {
        let supervisor_set: HashSet<WorkOrderActivity> =
            self.supervisor_algorithm.operational_state.get_unique_woa();

        let tactical_set: HashSet<WorkOrderActivity> = tactical_supervisor_link
            .keys()
            .cloned()
            .collect::<HashSet<_>>();

        let (done_set, tactical_set): (HashSet<WorkOrderActivity>, HashSet<WorkOrderActivity>) =
            tactical_set.into_iter().partition(|woa| {
                tactical_supervisor_link.get(woa).unwrap().work_remaining == Work::from(0.0)
            });

        let done_woas: HashSet<TransitionTypes> = done_set
            .into_iter()
            .map(|woa| {
                TransitionTypes::Done(woa)
            })
            .collect();

        let mut changed_woas = HashSet::new();

        let mut unchanged_woas = HashSet::new();

        supervisor_set
            .intersection(&tactical_set)
            .cloned()
            .for_each(|woa| {
                // We should go into the operational_state and find all matches on the
                // woa! Good this is the first step!
                let operational_state_machine_inner = self
                    .supervisor_algorithm
                    .operational_state
                    .get_iter()
                    .filter(|(key, _)| key.1 == woa)
                    .collect::<Vec<_>>();

                operational_state_machine_inner.iter().for_each(|(_, (delegate, _, _))| {
                    let tactical_operation = delegate.read().unwrap().tactical_operation();
                    if tactical_operation
                        == *tactical_supervisor_link.get(&woa).unwrap()
                    {
                        let transition_type = TransitionTypes::Unchanged(woa);
                        unchanged_woas.insert(transition_type);
                    } else {
                        let transition_type = TransitionTypes::Changed((woa, tactical_operation));
                        changed_woas.insert(transition_type);
                    }
                })
                // TransitionTypes::Unchanged(woa))
            });

        let leaving_woas = supervisor_set
            .difference(&tactical_set)
            .cloned()
            .map(|woa| {
                TransitionTypes::Leaving(woa)
            })
            .collect::<HashSet<TransitionTypes>>();

        let entering_woas = tactical_set
            .difference(&supervisor_set)
            .cloned()
            .map(|woa| {
                let tactical_operation = tactical_supervisor_link.get(&woa).unwrap().clone();
                TransitionTypes::Entering((woa, tactical_operation))
            })
            .collect::<HashSet<TransitionTypes>>();

        let mut final_set = entering_woas;

        final_set.extend(unchanged_woas);
        final_set.extend(leaving_woas);
        final_set.extend(done_woas);

        final_set
    }
}

impl Handler<StatusMessage> for SupervisorAgent {
    type Result = String;

    #[instrument(level = "trace", skip_all)]
    fn handle(&mut self, _msg: StatusMessage, _ctx: &mut Self::Context) -> Self::Result {
        format!(
            "ID*: {}, Work Center: {:?}, Main Work Center: {:?}",
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

type StrategicMessage = ();
type TacticalMessage = HashMap<(WorkOrderNumber, ActivityNumber), Arc<TacticalOperation>>;
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

        let _enter = span.enter();

        match state_link {
            StateLink::Strategic(_) => Ok(()),
            StateLink::Tactical(tactical_supervisor_link) => {
                info!(self.id = ?self.supervisor_id);
                let transition_sets = self.make_transition_sets_from_tactical_state_link(
                    tactical_supervisor_link.clone(),
                );

                assert!(self
                    .supervisor_algorithm
                    .operational_state
                    .are_unassigned_woas_valid());
                for transition_type in &transition_sets {
                    for operational_agent in &self.operational_agent_addrs {
                        match transition_type {
                            TransitionTypes::Entering((_work_order_activity, tactical_operation)) => {
                                if operational_agent.0 .1.contains(transition_type.resource()) {
                                    self.supervisor_algorithm
                                        .operational_state
                                        .update_operaitonal_state(
                                            transition_type.clone(),
                                            operational_agent,
                                            self.supervisor_id.clone(),
                                        )
                                }
                            }
                            TransitionTypes::Leaving(work_order_number) => {
                                let leaving_delegate_option = self
                                    .supervisor_algorithm
                                    .operational_state
                                    .get(&(operational_agent.0.clone(), *work_order_number));

                                match leaving_delegate_option {
                                    Some(_woa) => self
                                        .supervisor_algorithm
                                        .operational_state
                                        .update_operaitonal_state(
                                            transition_type.clone(),
                                            operational_agent,
                                            self.supervisor_id.clone(),
                                        ),
                                    None => {
                                        event!(Level::DEBUG, "If you get this, and suspect an error, check that the woa that is being dropped does not match the resource of operational agent. This could be a very pernicious bug if true, but a significant rewrite of the type system is needed to assert! this")
                                    }
                                }
                            }
                            TransitionTypes::Unchanged(delegate) => {}
                            TransitionTypes::Changed(delegate) => {
                                todo!();
                            }
                            TransitionTypes::Done(delegate) => {
                                // What should the logic be here? I think that the most important think
                                // will be to make something... For an operational agent we want to set
                                // the Delegate::Done. The thing is that we should already know this 
                                // when inside of the agent, we should have the Arc<TacticalOperation> 
                                // laying around.
                                self.supervisor_algorithm
                                    .operational_state
                                    .update_operaitonal_state(
                                        transition_type.clone(),
                                        operational_agent,
                                        self.supervisor_id.clone(),
                                    )
                            }
                        }
                    }
                }
                assert!(self
                    .supervisor_algorithm
                    .operational_state
                    .are_unassigned_woas_valid());
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
