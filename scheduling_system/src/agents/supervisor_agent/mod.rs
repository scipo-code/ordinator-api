pub mod algorithm;
pub mod assert_functions;
pub mod delegate;
pub mod operational_state_machine;

use assert_functions::SupervisorAssertions;
use operational_state_machine::assert_functions::OperationalStateMachineAssertions;
use rand::{prelude::SliceRandom, rngs::ThreadRng};
use std::{
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicU64, Arc, Mutex},
    time::Instant,
};

use actix::prelude::*;
use shared_types::{
    agent_error::AgentError,
    orchestrator::OrchestratorMessage,
    scheduling_environment::{
        work_order::{
            operation::{ActivityNumber, Work},
            WorkOrderActivity, WorkOrderNumber,
        },
        worker_environment::resources::Resources,
    },
    supervisor::{
        supervisor_response_scheduling::SupervisorResponseScheduling,
        supervisor_response_status::SupervisorResponseStatus, SupervisorRequestMessage,
        SupervisorResponseMessage,
    },
    Asset, StopMessage,
};

use shared_types::scheduling_environment::worker_environment::resources::Id;
use tracing::{event, info, instrument, warn, Level};

use shared_types::scheduling_environment::SchedulingEnvironment;

use self::algorithm::SupervisorAlgorithm;

use super::{
    operational_agent::{algorithm::OperationalObjective, OperationalAgent},
    tactical_agent::{tactical_algorithm::TacticalOperation, TacticalAgent},
    traits::LargeNeighborHoodSearch,
    ScheduleIteration, SetAddr, StateLink, StateLinkError, StateLinkWrapper,
};

#[allow(dead_code)]
pub struct SupervisorAgent {
    supervisor_id: Id,
    asset: Asset,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub supervisor_algorithm: SupervisorAlgorithm,
    tactical_agent_addr: Addr<TacticalAgent>,
    operational_agent_addrs: HashMap<Id, Addr<OperationalAgent>>,
    number_of_operational_agents: Arc<AtomicU64>,
}

#[derive(Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum TransitionTypes {
    Entering((WorkOrderActivity, Arc<TacticalOperation>)),
    Leaving(WorkOrderActivity),
    Unchanged(WorkOrderActivity),
    Changed((WorkOrderActivity, Arc<TacticalOperation>)),
    Done((WorkOrderActivity, Arc<TacticalOperation>)),
}

impl TransitionTypes {
    pub fn resource(&self) -> &Resources {
        match self {
            TransitionTypes::Entering((_, tac)) => (**tac).get_resource(),
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
        self.test_that_operational_state_machine_woas_are_a_subset_of_tactical_operations();
        ctx.set_mailbox_capacity(1000);
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
        self.test_that_operational_state_machine_woas_are_a_subset_of_tactical_operations();
        self.calculate_objective_value();

        let rng = rand::thread_rng();

        let number_of_removed_work_orders = 10;
        self.unschedule_random_work_orders(number_of_removed_work_orders, rng);

        self.schedule();
        self.test_that_operational_state_machine_woas_are_a_subset_of_tactical_operations();

        //TODO: Asset here that the state is correct inside of the OperationalState
        self.supervisor_algorithm.operational_state.assert_that_operational_state_machine_for_each_work_order_is_either_delegate_assign_and_unassign_or_all_assess();

        event!(
            Level::DEBUG,
            number_of_operational_agents =
                self.supervisor_algorithm.operational_agent_objectives.len()
        );

        event!(
            Level::INFO,
            supervisor_objective = self.supervisor_algorithm.objective_value
        );

        ctx.wait(
            tokio::time::sleep(tokio::time::Duration::from_millis(
                dotenvy::var("SUPERVISOR_THROTTLING")
                    .expect("The SUPERVISOR_THROTTLING environment variable should always be set")
                    .parse::<u64>()
                    .expect("The SUPERVISOR_THROTTLING environment variable have to be an u64 compatible type"),
            ))
            .into_actor(self),
        );
        ctx.notify(ScheduleIteration {});
    }
}

impl SupervisorAgent {
    pub fn new(
        id_supervisor: Id,
        asset: Asset,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        tactical_agent_addr: Addr<TacticalAgent>,
        number_of_operational_agents: Arc<AtomicU64>,
    ) -> SupervisorAgent {
        let supervisor_resource = id_supervisor.2.clone().unwrap();
        SupervisorAgent {
            supervisor_id: id_supervisor,
            asset,
            scheduling_environment,
            supervisor_algorithm: SupervisorAlgorithm::new(supervisor_resource),
            tactical_agent_addr,
            operational_agent_addrs: HashMap::new(),
            number_of_operational_agents,
        }
    }

    fn unschedule_random_work_orders(&mut self, number_of_work_orders: u64, mut rng: ThreadRng) {
        let work_order_numbers = self
            .supervisor_algorithm
            .operational_state
            .get_assigned_and_unassigned_work_orders();

        let sampled_work_order_numbers = work_order_numbers
            .choose_multiple(&mut rng, number_of_work_orders as usize)
            .collect::<Vec<_>>()
            .clone();

        for work_order_number in sampled_work_order_numbers {
            self.unschedule(*work_order_number)
        }
    }

    fn make_transition_sets_from_tactical_state_link(
        &self,
        tactical_supervisor_link: HashMap<
            (WorkOrderNumber, ActivityNumber),
            Arc<TacticalOperation>,
        >,
    ) -> TransitionSets {
        let supervisor_set: HashSet<WorkOrderActivity> = self
            .supervisor_algorithm
            .tactical_operations
            .keys()
            .cloned()
            .collect();

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
                let tactical_operation = tactical_supervisor_link.get(&woa).unwrap();
                TransitionTypes::Done((woa, Arc::clone(tactical_operation)))
            })
            .collect();

        let mut changed_woas = HashSet::new();

        let mut unchanged_woas = HashSet::new();

        supervisor_set
            .intersection(&tactical_set)
            .cloned()
            .for_each(|woa| {
                let tactical_operation = self
                    .supervisor_algorithm
                    .tactical_operations
                    .get(&woa)
                    .unwrap()
                    .clone();
                if tactical_operation == *tactical_supervisor_link.get(&woa).unwrap() {
                    let transition_type = TransitionTypes::Unchanged(woa);
                    unchanged_woas.insert(transition_type);
                } else {
                    let transition_type = TransitionTypes::Changed((woa, tactical_operation));
                    changed_woas.insert(transition_type);
                }
            });

        let leaving_woas = supervisor_set
            .difference(&tactical_set)
            .cloned()
            .map(|woa| TransitionTypes::Leaving(woa))
            .collect::<HashSet<TransitionTypes>>();

        let entering_woas = tactical_set
            .difference(&supervisor_set)
            .cloned()
            .map(|woa| {
                let tactical_operation = tactical_supervisor_link.get(&woa).unwrap().clone();
                TransitionTypes::Entering((woa, tactical_operation))
            })
            .collect::<HashSet<TransitionTypes>>();

        assert!(leaving_woas.is_disjoint(&entering_woas));
        assert!(entering_woas.is_disjoint(&done_woas));
        assert!(leaving_woas.is_disjoint(&done_woas));

        assert!(unchanged_woas.is_disjoint(&done_woas));
        assert!(changed_woas.is_disjoint(&done_woas));

        let mut final_set = entering_woas;

        final_set.extend(unchanged_woas);
        final_set.extend(leaving_woas);
        final_set.extend(done_woas);

        final_set
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
// Why do we send this message? I am not really sure?
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
                let instant = Instant::now();
                let transition_sets = self.make_transition_sets_from_tactical_state_link(
                    tactical_supervisor_link.clone(),
                );

                // assert!(self
                //     .supervisor_algorithm
                //     .operational_state
                //     .are_unassigned_woas_valid());
                for transition_type in &transition_sets {
                    match transition_type {
                        TransitionTypes::Entering((work_order_activity, tactical_operation)) => {
                            let insert_option = self
                                .supervisor_algorithm
                                .tactical_operations
                                .insert(*work_order_activity, tactical_operation.clone());
                            match insert_option {
                                Some(_) => panic!(),
                                None => (),
                            }

                            for operational_agent in &self.operational_agent_addrs {
                                if operational_agent.0 .1.contains(transition_type.resource()) {
                                    self.supervisor_algorithm
                                        .operational_state
                                        .update_operational_state(
                                            transition_type.clone(),
                                            operational_agent,
                                            self.supervisor_id.clone(),
                                        )
                                }
                            }
                        }
                        TransitionTypes::Leaving(work_order_activity) => {
                            let remove_option = self
                                .supervisor_algorithm
                                .tactical_operations
                                .remove(work_order_activity);
                            match remove_option {
                                Some(_) => {
                                    event!(Level::DEBUG, work_order_activity = ?work_order_activity, "TacticalOperation left the SupervisorAgent");
                                }
                                None => {
                                    event!(Level::ERROR, work_order_activity = ?work_order_activity, all_work_order_activities = ?self.supervisor_algorithm.tactical_operations.keys());
                                    panic!();
                                }
                            }
                            for operational_agent in &self.operational_agent_addrs {
                                let leaving_delegate_option = self
                                    .supervisor_algorithm
                                    .operational_state
                                    .get(&(operational_agent.0.clone(), *work_order_activity));

                                match leaving_delegate_option {
                                    Some(_woa) => self
                                        .supervisor_algorithm
                                        .operational_state
                                        .update_operational_state(
                                            transition_type.clone(),
                                            operational_agent,
                                            self.supervisor_id.clone(),
                                        ),
                                    None => {
                                        event!(Level::DEBUG, "If you get this, and suspect an error, check that the woa that is being dropped does not match the resource of operational agent. This could be a very pernicious bug if true, but a significant rewrite of the type system is needed to assert! this")
                                    }
                                }
                            }

                            assert!(!self
                                .supervisor_algorithm
                                .operational_state
                                .is_work_order_activity_present(work_order_activity))
                        }
                        TransitionTypes::Unchanged(_delegate) => {}
                        TransitionTypes::Changed(_delegate) => {
                            todo!();
                        }
                        TransitionTypes::Done((work_order_activity, tactical_operation)) => {
                            let insert_option = self
                                .supervisor_algorithm
                                .tactical_operations
                                .insert(*work_order_activity, tactical_operation.clone());
                            match insert_option {
                                Some(_) => (), //panic!(),
                                None => (),
                            }
                        }
                    }
                }

                // assert!(self
                //     .supervisor_algorithm
                //     .operational_state
                //     .are_unassigned_woas_valid());

                if instant.elapsed().as_secs_f32() > 4.0 {
                    panic!()
                };
                Ok(())
            }
            StateLink::Supervisor(_) => Ok(()),
            StateLink::Operational(_operational_solution) => Ok(()),
        }
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
        event!(Level::WARN, "start_of_supervisor_handler");
        tracing::info!(
            "Received SupervisorRequestMessage: {:?}",
            supervisor_request_message
        );

        match supervisor_request_message {
            SupervisorRequestMessage::Status(supervisor_status_message) => {
                event!(Level::WARN, "start of status message initialization");
                tracing::info!(
                    "Received SupervisorStatusMessage: {:?}",
                    supervisor_status_message
                );
                let supervisor_status = SupervisorResponseStatus::new(
                    self.supervisor_id.clone().2.unwrap().resource,
                    self.supervisor_algorithm
                        .operational_state
                        .count_unique_woa(),
                    self.supervisor_algorithm.objective_value(),
                );
                event!(Level::WARN, "after creation of the supervisor_status");

                Ok(SupervisorResponseMessage::Status(supervisor_status))
            }

            SupervisorRequestMessage::Scheduling(_scheduling_message) => Ok(
                SupervisorResponseMessage::Scheduling(SupervisorResponseScheduling {}),
            ),
        }
    }
}

impl Handler<OrchestratorMessage<(Id, OperationalObjective)>> for SupervisorAgent {
    type Result = ();

    fn handle(
        &mut self,
        msg: OrchestratorMessage<(Id, OperationalObjective)>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.supervisor_algorithm
            .operational_agent_objectives
            .insert(
                msg.message_from_orchestrator.0,
                msg.message_from_orchestrator.1,
            );
    }
}
