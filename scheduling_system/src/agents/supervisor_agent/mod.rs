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
    orchestrator::OrchestratorMessage,
    scheduling_environment::work_order::{operation::Work, WorkOrderActivity},
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
    tactical_agent::TacticalAgent,
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
    Entering(WorkOrderActivity),
    Leaving(WorkOrderActivity),
    Unchanged(WorkOrderActivity),
    Changed(WorkOrderActivity),
    Done(WorkOrderActivity),
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
        event!(Level::WARN, "FIND STOP POINT");
        self.update_operational_state_machine()
            .expect("Could not load the data from the load SharedSolution");

        event!(Level::WARN, "FIND STOP POINT");
        event!(
            Level::WARN,
            number_of_operational_states =
                self.supervisor_algorithm.operational_state_machine.len()
        );

        let rng = rand::thread_rng();
        self.supervisor_algorithm.calculate_objective_value();

        let current_state = self.capture_current_state();

        let old_objective_value = current_state.objective_value.clone();

        event!(
            Level::WARN,
            current_state = ?current_state.state_of_each_agent
        );

        let number_of_removed_work_orders = 10;
        event!(Level::WARN, "FIND STOP POINT");
        self.unschedule_random_work_orders(number_of_removed_work_orders, rng);

        self.supervisor_algorithm.schedule();
        event!(Level::WARN, "FIND STOP POINT");
        self.assert_that_operational_state_machine_woas_are_a_subset_of_tactical_operations();

        event!(Level::WARN, "FIND STOP POINT");
        let new_objective_value = self.supervisor_algorithm.calculate_objective_value();

        event!(Level::WARN, "FIND STOP POINT");
        assert_eq!(
            new_objective_value,
            self.supervisor_algorithm.calculate_objective_value()
        );
        event!(
            Level::WARN,
            new_state = ?self.capture_current_state().state_of_each_agent
        );

        event!(Level::WARN, "FIND STOP POINT");
        self.supervisor_algorithm.operational_state_machine.assert_that_operational_state_machine_for_each_work_order_is_either_delegate_assign_and_unassign_or_all_assess();

        // self.supervisor_algorithm.operational_state.assert_that_operational_state_machine_is_different_from_saved_operational_state_machine(&current_state).unwrap();

        event!(Level::WARN, "FIND STOP POINT");
        if self.supervisor_algorithm.objective_value < current_state.objective_value {
            event!(Level::WARN, "FIND STOP POINT");
            self.release_current_state(current_state.clone());
            event!(Level::WARN, "FIND STOP POINT");
            self.supervisor_algorithm.calculate_objective_value();
        }

        event!(Level::WARN, "FIND STOP POINT");
        assert!(self.supervisor_algorithm.objective_value >= old_objective_value);

        event!(Level::WARN, "FIND STOP POINT");
        event!(
            Level::DEBUG,
            number_of_operational_agents =
                self.supervisor_algorithm.operational_agent_objectives.len()
        );

        event!(Level::WARN, "FIND STOP POINT");
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
            .operational_state_machine
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
            .operational_state_machine
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
            .operational_state_machine
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

    fn update_operational_state_machine(&mut self) -> Result<()> {
        let loaded_tactical_solution = &self.supervisor_algorithm.loaded_shared_solution.tactical;

        let supervisor_work_order_activities = loaded_tactical_solution.supervisor_activities(
            &self
                .supervisor_algorithm
                .supervisor_parameters
                .supervisor_periods,
        );

        let transition_sets = self.update_supervisor_solution();

        Ok(())
    }

    fn update_supervisor_solution(&self) -> Result<()> {
        event!(Level::WARN, "FIND STOP POINT");
        let work_order_coming_from_tactical = self
            .supervisor_algorithm
            .loaded_shared_solution
            .tactical
            .supervisor_activities(
                &self
                    .supervisor_algorithm
                    .supervisor_parameters
                    .supervisor_periods,
            );

        for work_order_activity in work_order_coming_from_tactical {
            self.supervisor_algorithm
                .supervisor_parameters
                .create(&locked_scheduling_environment, &work_order_activity);

            for operational_agent in &self.operational_agent_addrs {
                if operational_agent.0 .1.contains(
                    &self
                        .supervisor_algorithm
                        .supervisor_parameters
                        .strategic_parameter(&work_order_activity)
                        .context("The SupervisorParameter was not found")?
                        .resource,
                ) {
                    self.supervisor_algorithm
                        .operational_state_machine
                        .update_operational_state(
                            transition_type.clone(),
                            operational_agent,
                            self.supervisor_id.clone(),
                        )
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
type TacticalMessage = ();
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
            StateLink::Tactical(_) => Ok(()),
            StateLink::Supervisor(_) => Ok(()),
            StateLink::Operational(_operational_solution) => Ok(()),
        }
    }
}

impl Handler<SupervisorRequestMessage> for SupervisorAgent {
    type Result = Result<SupervisorResponseMessage>;

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
                        .operational_state_machine
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
