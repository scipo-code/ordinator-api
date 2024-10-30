pub mod algorithm;
pub mod assert_functions;
pub mod delegate;
pub mod operational_state_machine;

use anyhow::{bail, Context, Result};
use assert_functions::SupervisorAssertions;
use delegate::Delegate;
use operational_state_machine::assert_functions::OperationalStateMachineAssertions;
use rand::{prelude::SliceRandom, rngs::ThreadRng};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
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
use tracing::{event, instrument, Level};

use shared_types::scheduling_environment::SchedulingEnvironment;

use self::algorithm::SupervisorAlgorithm;

use super::{
    operational_agent::{algorithm::OperationalObjective, OperationalAgent},
    tactical_agent::{tactical_algorithm::TacticalOperation, TacticalAgent},
    traits::LargeNeighborHoodSearch,
    ArcSwapSharedSolution, ScheduleIteration, SetAddr, StateLink, StateLinkWrapper,
};

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
    type Context = actix::Context<Self>;

    #[instrument(level = "trace", skip_all)]
    fn started(&mut self, ctx: &mut Self::Context) {
        self.assert_that_operational_state_machine_woas_are_a_subset_of_tactical_operations();
        ctx.set_mailbox_capacity(1000);
        self.tactical_agent_addr.do_send(SetAddr::Supervisor(
            self.supervisor_id.clone(),
            ctx.address(),
        ));
        ctx.notify(ScheduleIteration {});
    }
}

impl Handler<ScheduleIteration> for SupervisorAgent {
    type Result = Result<()>;

    #[instrument(skip_all)]
    fn handle(&mut self, _msg: ScheduleIteration, ctx: &mut actix::Context<Self>) -> Self::Result {
        self.assert_that_operational_state_machine_woas_are_a_subset_of_tactical_operations();
        self.supervisor_algorithm.load_shared_solution();

        let rng = rand::thread_rng();
        self.supervisor_algorithm.calculate_objective_value();

        let current_state = self.capture_current_state();

        let old_objective_value = current_state.objective_value.clone();

        event!(
            Level::WARN,
            current_state = ?current_state.state_of_each_agent
        );

        let number_of_removed_work_orders = 10;
        self.unschedule_random_work_orders(number_of_removed_work_orders, rng);

        self.supervisor_algorithm.schedule();
        self.assert_that_operational_state_machine_woas_are_a_subset_of_tactical_operations();

        let new_objective_value = self.supervisor_algorithm.calculate_objective_value();

        assert_eq!(
            new_objective_value,
            self.supervisor_algorithm.calculate_objective_value()
        );
        event!(
            Level::WARN,
            new_state = ?self.capture_current_state().state_of_each_agent
        );

        self.supervisor_algorithm.operational_state.assert_that_operational_state_machine_for_each_work_order_is_either_delegate_assign_and_unassign_or_all_assess();

        // self.supervisor_algorithm.operational_state.assert_that_operational_state_machine_is_different_from_saved_operational_state_machine(&current_state).unwrap();

        if self.supervisor_algorithm.objective_value < current_state.objective_value {
            self.release_current_state(current_state.clone());
            self.supervisor_algorithm.calculate_objective_value();
        }

        assert!(self.supervisor_algorithm.objective_value >= old_objective_value);

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
        Ok(())
    }
}

#[derive(Clone)]
pub struct CapturedSupervisorState {
    objective_value: f64,
    state_of_each_agent: HashMap<(Id, WorkOrderActivity), Delegate>,
}

impl SupervisorAgent {
    fn capture_current_state(&self) -> CapturedSupervisorState {
        let mut state_of_each_agent = HashMap::new();
        self.supervisor_algorithm
            .operational_state
            .get_iter()
            .for_each(|(id_woa, del_fit)| {
                state_of_each_agent.insert(id_woa.clone(), del_fit.0.load(Ordering::SeqCst));
            });

        CapturedSupervisorState {
            objective_value: self.supervisor_algorithm.objective_value,
            state_of_each_agent,
        }
    }

    fn release_current_state(&mut self, captured_supervisor_state: CapturedSupervisorState) {
        self.supervisor_algorithm
            .operational_state
            .set_operational_state(captured_supervisor_state);
    }
}

impl SupervisorAgent {
    pub fn new(
        id_supervisor: Id,
        asset: Asset,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        tactical_agent_addr: Addr<TacticalAgent>,
        arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
        number_of_operational_agents: Arc<AtomicU64>,
    ) -> SupervisorAgent {
        let supervisor_resource = id_supervisor.2.clone().unwrap();
        SupervisorAgent {
            supervisor_id: id_supervisor,
            asset,
            scheduling_environment,
            supervisor_algorithm: SupervisorAlgorithm::new(
                supervisor_resource,
                arc_swap_shared_solution,
            ),
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
            self.supervisor_algorithm
                .unschedule(*work_order_number)
                .unwrap_or_else(|err| {
                    event!(Level::ERROR, error = ?err, work_order_number = ?work_order_number);
                    eprintln!(
                        "Could not unschedule work_order_number: {:?}",
                        work_order_number
                    );
                    panic!();
                })
        }
        // self.supervisor_algorithm.operational_state.assert_that_operational_state_machine_is_different_from_saved_operational_state_machine(&old_state).unwrap();
    }

    fn make_transition_sets_from_tactical_state_link(
        &self,
        tactical_supervisor_link: HashMap<
            (WorkOrderNumber, ActivityNumber),
            Arc<TacticalOperation>,
        >,
    ) -> TransitionSets {
        let tactical_supervisor_link = self
            .supervisor_algorithm
            .loaded_shared_solution
            .tactical
            .tactical_days;

        let tactical_supervisor_link: HashMap<WorkOrderActivity, TacticalOperation> =
            tactical_supervisor_link
                .iter()
                // Here we only extract the map from the option
                .filter_map(|(won, opt_map)| opt_map.map(|map| (won, map)))
                // Now we want to extract the data from the inners HashMap,
                .flat_map(|(won, map)| map.into_iter().map(move |(acn, to)| ((*won, acn), to)))
                .collect();

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

                if tactical_operation == tactical_supervisor_link.get(&woa).unwrap() {
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
    fn handle_transition_sets(&mut self, transition_sets: HashSet<TransitionTypes>) -> Result<()> {
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
                        Some(previous_entry) => bail!(
                            "A TransitionTypes::Done should not already be present: {:?}",
                            previous_entry
                        ), //panic!(),
                        None => (),
                    }
                }
            }
        }
        Ok(())
    }
}

impl Handler<StopMessage> for SupervisorAgent {
    type Result = ();

    fn handle(&mut self, _msg: StopMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}

impl Handler<SetAddr> for SupervisorAgent {
    type Result = Result<()>;

    #[instrument(level = "trace", skip_all)]
    fn handle(&mut self, set_addr: SetAddr, _ctx: &mut Self::Context) -> Self::Result {
        if let SetAddr::Operational(id, addr) = set_addr {
            self.operational_agent_addrs.insert(id, addr);
            Ok(())
        } else {
            bail!("We have not created the logic for fixing this yet")
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
    type Result = Result<()>;

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
                let transition_sets = self.make_transition_sets_from_tactical_state_link(
                    tactical_supervisor_link.clone(),
                );

                self.handle_transition_sets(transition_sets)
                    .context("TranstionSets were not handled correctly")?;

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
                    self.supervisor_algorithm.objective_value,
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
