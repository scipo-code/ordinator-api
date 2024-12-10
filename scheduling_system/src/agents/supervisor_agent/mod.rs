pub mod algorithm;
pub mod assert_functions;
pub mod message_handlers;

use algorithm::delegate::Delegate;
use anyhow::{Context, Result};
use assert_functions::SupervisorAssertions;
use rand::{prelude::SliceRandom, rngs::ThreadRng};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicU64, Arc, Mutex},
};

use actix::prelude::*;
use shared_types::Asset;

use shared_types::scheduling_environment::worker_environment::resources::Id;
use tracing::{event, instrument, Level};

use shared_types::scheduling_environment::SchedulingEnvironment;

use crate::agents::SupervisorSolution;

use self::algorithm::SupervisorAlgorithm;

use super::{
    operational_agent::OperationalAgent, orchestrator::NotifyOrchestrator,
    tactical_agent::TacticalAgent, traits::LargeNeighborHoodSearch, ArcSwapSharedSolution,
    ScheduleIteration, SetAddr,
};

pub struct SupervisorAgent {
    supervisor_id: String,
    asset: Asset,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub supervisor_algorithm: SupervisorAlgorithm,
    tactical_agent_addr: Addr<TacticalAgent>,
    operational_agent_addrs: HashMap<Id, Addr<OperationalAgent>>,
    number_of_operational_agents: Arc<AtomicU64>,
    pub notify_orchestrator: NotifyOrchestrator,
}

impl Actor for SupervisorAgent {
    type Context = actix::Context<Self>;

    #[instrument(level = "trace", skip_all)]
    fn started(&mut self, ctx: &mut Self::Context) {
        self.assert_operational_state_machine_woas_is_subset_of_tactical_shared_solution()
            .unwrap();
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
        self.supervisor_algorithm.load_shared_solution();
        self.update_supervisor_solution_and_parameters()
            .expect("Could not load the data from the load SharedSolution");

        self.assert_operational_state_machine_woas_is_subset_of_tactical_shared_solution()
            .expect("OperationalStates should correspond with TacticalOperations");

        event!(
            Level::DEBUG,
            number_of_operational_states = self.supervisor_algorithm.supervisor_solution.len()
        );

        event!(
            Level::DEBUG,
            number_of_operational_agents = ?self.number_of_operational_agents
        );

        let rng = rand::thread_rng();
        self.supervisor_algorithm.calculate_objective_value();

        let old_supervisor_solution = self.supervisor_algorithm.supervisor_solution.clone();

        let number_of_removed_work_orders = 10;
        self.unschedule_random_work_orders(number_of_removed_work_orders, rng)
            .unwrap_or_else(|err| {
                panic!(
                    "Error: {}, Could not destroy {}",
                    err,
                    std::any::type_name::<SupervisorSolution>()
                )
            });

        self.supervisor_algorithm
            .schedule()
            .expect("SupervisorAlgorithm.schedule method failed");
        // self.assert_that_operational_state_machine_woas_are_a_subset_of_tactical_operations();

        let new_objective_value = self.supervisor_algorithm.calculate_objective_value();

        assert_eq!(
            new_objective_value,
            self.supervisor_algorithm.calculate_objective_value()
        );

        // self.supervisor_algorithm.operational_state_machine.assert_that_operational_state_machine_for_each_work_order_is_either_delegate_assign_and_unassign_or_all_assess();
        // self.supervisor_algorithm.operational_state.assert_that_operational_state_machine_is_different_from_saved_operational_state_machine(&current_state).unwrap();

        if self.supervisor_algorithm.objective_value >= old_supervisor_solution.objective_value {
            self.supervisor_algorithm.make_atomic_pointer_swap();
        } else if self.supervisor_algorithm.objective_value
            < old_supervisor_solution.objective_value
        {
            assert!(
                self.supervisor_algorithm.objective_value
                    >= old_supervisor_solution.objective_value
            );
            self.supervisor_algorithm.supervisor_solution = old_supervisor_solution;
            self.supervisor_algorithm.calculate_objective_value();
        }

        event!(
            Level::INFO,
            supervisor_objective_value = self.supervisor_algorithm.objective_value
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

impl SupervisorAgent {
    pub fn new(
        supervisor_id: Id,
        asset: Asset,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        tactical_agent_addr: Addr<TacticalAgent>,
        arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
        number_of_operational_agents: Arc<AtomicU64>,
        notify_orchestrator: NotifyOrchestrator,
    ) -> Result<SupervisorAgent> {
        let Id(id, resources, toml_supervisor) = supervisor_id;

        let number_of_supervisor_periods = toml_supervisor
            .context("Error with the supervisor configuration file")?
            .number_of_supervisor_periods;

        let supervisor_periods = &scheduling_environment
            .lock()
            .expect("SchedulingEnvironment lock poisoned")
            .time_environment
            .strategic_periods()[0..=number_of_supervisor_periods as usize]
            .to_vec();

        Ok(SupervisorAgent {
            supervisor_id: id,
            asset,
            scheduling_environment,
            supervisor_algorithm: SupervisorAlgorithm::new(
                resources,
                arc_swap_shared_solution,
                supervisor_periods,
            ),
            tactical_agent_addr,
            operational_agent_addrs: HashMap::new(),
            number_of_operational_agents,
            notify_orchestrator,
        })
    }

    fn unschedule_random_work_orders(
        &mut self,
        number_of_work_orders: u64,
        mut rng: ThreadRng,
    ) -> Result<()> {
        let work_order_numbers = self
            .supervisor_algorithm
            .supervisor_solution
            .get_assigned_and_unassigned_work_orders();

        let sampled_work_order_numbers = work_order_numbers
            .choose_multiple(&mut rng, number_of_work_orders as usize)
            .collect::<Vec<_>>()
            .clone();

        for work_order_number in sampled_work_order_numbers {
            self.supervisor_algorithm
                .unschedule(*work_order_number)
                .with_context(|| {
                    format!(
                        "Could not unschedule work_order_number: {:?}",
                        work_order_number
                    )
                })?;
        }
        Ok(())
        // self.supervisor_algorithm.operational_state.assert_that_operational_state_machine_is_different_from_saved_operational_state_machine(&old_state).unwrap();
    }

    fn update_supervisor_solution_and_parameters(&mut self) -> Result<()> {
        let entering_work_orders_from_strategic = self
            .supervisor_algorithm
            .loaded_shared_solution
            .strategic
            .supervisor_work_orders_from_strategic(
                &self
                    .supervisor_algorithm
                    .supervisor_parameters
                    .supervisor_periods,
            );

        self.supervisor_algorithm
            .supervisor_solution
            .remove_leaving_work_order_activities(&entering_work_orders_from_strategic);

        event!(Level::WARN, number_coming_from_tactical = ?entering_work_orders_from_strategic);

        let locked_scheduling_environment = self
            .scheduling_environment
            .lock()
            .expect("Could not acquire SchedulingEnvironment lock");

        let work_order_activities: Vec<_> = locked_scheduling_environment
            .work_orders
            .inner
            .iter()
            .filter(|(won, _)| entering_work_orders_from_strategic.contains(won))
            .flat_map(|(won, wo)| wo.operations.keys().map(move |acn| (*won, *acn)))
            .collect();

        for work_order_activity in work_order_activities {
            self.supervisor_algorithm
                .supervisor_parameters
                .create_and_insert_supervisor_parameter(
                    &locked_scheduling_environment,
                    &work_order_activity,
                );

            for operational_agent in &self.operational_agent_addrs {
                if operational_agent.0 .1.contains(
                    &self
                        .supervisor_algorithm
                        .supervisor_parameters
                        .supervisor_parameter(&work_order_activity)
                        .context("The SupervisorParameter was not found")?
                        .resource,
                ) {
                    let operation = locked_scheduling_environment.operation(&work_order_activity);
                    let delegate = Delegate::build(operation);
                    self.supervisor_algorithm
                        .supervisor_solution
                        .insert_supervisor_solution(
                            operational_agent,
                            delegate,
                            work_order_activity,
                        )
                        .context("Supervisor could not insert operational solution correctly")?;
                }
            }
        }
        Ok(())
    }
}
