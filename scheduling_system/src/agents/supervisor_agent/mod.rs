pub mod algorithm;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use shared_messages::{
    agent_error::AgentError,
    models::work_order::{operation::ActivityNumber, WorkOrderNumber},
    supervisor::SupervisorRequestMessage,
    AlgorithmState, Asset, ConstraintState, StatusMessage, StopMessage,
};

use shared_messages::models::worker_environment::resources::Id;
use tracing::{error, instrument, warn};

use shared_messages::models::SchedulingEnvironment;

use super::{
    operational_agent::OperationalAgent,
    tactical_agent::{tactical_algorithm::OperationSolution, TacticalAgent},
    traits::TestAlgorithm,
    SetAddr, StateLink, UpdateWorkOrderMessage,
};

pub struct SupervisorAgent {
    id_supervisor: Id,
    asset: Asset,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    assigned_work_orders: Vec<(WorkOrderNumber, HashMap<ActivityNumber, OperationSolution>)>,
    tactical_agent_addr: Addr<TacticalAgent>,
    operational_agent_addrs: HashMap<Id, Addr<OperationalAgent>>,
}

impl Actor for SupervisorAgent {
    type Context = Context<Self>;

    #[instrument(level = "trace", skip_all)]
    fn started(&mut self, ctx: &mut Self::Context) {
        self.tactical_agent_addr.do_send(SetAddr::Supervisor(
            self.id_supervisor.clone(),
            ctx.address(),
        ));
    }
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
            assigned_work_orders: Vec::new(),
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

impl Handler<StateLink> for SupervisorAgent {
    type Result = ();

    #[instrument(level = "trace", skip_all)]
    fn handle(&mut self, state_link: StateLink, _ctx: &mut Self::Context) {
        match state_link {
            StateLink::Strategic(_) => {}
            StateLink::Tactical(tactical_supervisor_link) => {
                self.assigned_work_orders = tactical_supervisor_link;
            }
            StateLink::Supervisor => {}
            StateLink::Operational => {}
        }
    }
}

impl Handler<UpdateWorkOrderMessage> for SupervisorAgent {
    type Result = ();

    fn handle(
        &mut self,
        update_work_order: UpdateWorkOrderMessage,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        // todo!()
        warn!("Updateimpl Handler<UpdateWorkOrderMessage> for SupervisorAgent should be implemented for the supervisor agent");
    }
}

impl Handler<SupervisorRequestMessage> for SupervisorAgent {
    type Result = Result<String, AgentError>;

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
                Ok(format!(
                    "Received SupervisorStatusMessage: {:?}",
                    self.assigned_work_orders
                ))
            }
            SupervisorRequestMessage::Test => {
                let mut algorithm_state = self.determine_algorithm_state();

                let supervisor_test_output = format!(
                    "respect_main_resource_trait: {}",
                    algorithm_state
                        .infeasible_cases_mut()
                        .unwrap()
                        .respect_main_work_center
                );
                Ok(supervisor_test_output)
            }
        }
    }
}

pub struct SupervisorInfeasibleCases {
    respect_main_work_center: ConstraintState<String>,
}

impl Default for SupervisorInfeasibleCases {
    fn default() -> Self {
        Self {
            respect_main_work_center: ConstraintState::Infeasible("Infeasible".to_string()),
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
        for (work_order_number, _operation_solution) in self.assigned_work_orders.iter() {
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
