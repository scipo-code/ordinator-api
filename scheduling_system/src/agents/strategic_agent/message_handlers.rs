use std::collections::HashMap;

use actix::Handler;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use shared_types::orchestrator::WorkOrdersStatus;
use shared_types::scheduling_environment::work_order::status_codes::MaterialStatus;
use shared_types::scheduling_environment::work_order::WorkOrder;
use shared_types::strategic::strategic_response_periods::StrategicResponsePeriods;
use shared_types::strategic::strategic_response_scheduling::StrategicResponseScheduling;
use shared_types::strategic::strategic_response_status::StrategicResponseStatus;
use shared_types::strategic::StrategicSchedulingEnvironmentCommands;
use shared_types::AgentExports;
use shared_types::SolutionExportMessage;
use shared_types::{
    orchestrator::WorkOrderResponse,
    scheduling_environment::work_order::WorkOrderNumber,
    strategic::{
        strategic_request_status_message::StrategicStatusMessage, StrategicRequestMessage,
        StrategicResponseMessage,
    },
};
use tracing::event;
use tracing::Level;

use crate::agents::strategic_agent::strategic_algorithm::strategic_parameters::StrategicParameterBuilder;
use crate::agents::traits::LargeNeighborHoodSearch;
use crate::agents::SetAddr;
use crate::agents::UpdateWorkOrderMessage;

use super::StrategicAgent;

impl Handler<StrategicRequestMessage> for StrategicAgent {
    type Result = Result<StrategicResponseMessage>;

    fn handle(
        &mut self,
        strategic_request_message: StrategicRequestMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        match strategic_request_message {
            StrategicRequestMessage::Status(strategic_status_message) => {
                match strategic_status_message {
                    StrategicStatusMessage::General => {
                        let strategic_objective_value =
                            self.strategic_algorithm.strategic_solution.objective_value;

                        let strategic_parameters = &self.strategic_algorithm.strategic_parameters;

                        let number_of_strategic_work_orders =
                            strategic_parameters.strategic_work_order_parameters.len();

                        let asset = &self.asset;

                        let number_of_periods = self.strategic_algorithm.periods().len();

                        let strategic_response_status = StrategicResponseStatus::new(
                            asset.clone(),
                            strategic_objective_value,
                            number_of_strategic_work_orders,
                            number_of_periods,
                        );

                        let strategic_response_message =
                            StrategicResponseMessage::Status(strategic_response_status);
                        Ok(strategic_response_message)
                    }
                    StrategicStatusMessage::Period(period) => {
                        if !self
                            .strategic_algorithm
                            .periods()
                            .iter()
                            .map(|period| period.period_string())
                            .collect::<Vec<_>>()
                            .contains(&period)
                        {
                            bail!("Period not found in the the scheduling environment".to_string());
                        }

                        let work_orders_by_period: HashMap<WorkOrderNumber, WorkOrderResponse> =
                            self.strategic_algorithm
                                .strategic_periods()
                                .iter()
                                .filter(|(_, sch_per)| match sch_per {
                                    Some(scheduled_period) => {
                                        scheduled_period.period_string() == period
                                    }
                                    None => false,
                                })
                                .map(|(work_order_number, _)| {
                                    let work_order = self
                                        .scheduling_environment
                                        .lock()
                                        .unwrap()
                                        .work_orders()
                                        .inner
                                        .get(work_order_number)
                                        .unwrap()
                                        .clone();

                                    let work_order_response = WorkOrderResponse::new(
                                        &work_order,
                                        (**self.strategic_algorithm.loaded_shared_solution)
                                            .clone()
                                            .into(),
                                    );
                                    (*work_order_number, work_order_response)
                                })
                                .collect();

                        let work_orders_in_period =
                            WorkOrdersStatus::Multiple(work_orders_by_period);

                        let strategic_response_message =
                            StrategicResponseMessage::WorkOrder(work_orders_in_period);

                        Ok(strategic_response_message)
                    }
                }
            }
            StrategicRequestMessage::Scheduling(scheduling_message) => {
                let scheduling_output: StrategicResponseScheduling = self
                    .strategic_algorithm
                    .update_scheduling_state(scheduling_message)
                    .context("scheduling request message was not handled correct")?;

                self.strategic_algorithm.calculate_objective_value();
                event!(Level::INFO, strategic_objective_value = %self.strategic_algorithm.strategic_solution.objective_value);
                Ok(StrategicResponseMessage::Scheduling(scheduling_output))
            }
            StrategicRequestMessage::Resources(resources_message) => {
                let resources_output = self
                    .strategic_algorithm
                    .update_resources_state(resources_message);

                self.strategic_algorithm.calculate_objective_value();
                event!(Level::INFO, strategic_objective_value = %self.strategic_algorithm.strategic_solution.objective_value);
                Ok(StrategicResponseMessage::Resources(
                    resources_output.unwrap(),
                ))
            }
            StrategicRequestMessage::Periods(periods_message) => {
                let mut scheduling_env_lock = self.scheduling_environment.lock().unwrap();

                let periods = scheduling_env_lock.periods_mut();

                for period_id in periods_message.periods.iter() {
                    if periods.last().unwrap().id() + 1 == *period_id {
                        let new_period =
                            periods.last().unwrap().clone() + chrono::Duration::weeks(2);
                        periods.push(new_period);
                    } else {
                        event!(Level::ERROR, "periods not handled correctly");
                    }
                }
                self.strategic_algorithm.set_periods(periods.to_vec());
                let strategic_response_periods = StrategicResponsePeriods::new(periods.clone());
                Ok(StrategicResponseMessage::Periods(
                    strategic_response_periods,
                ))
            }
            StrategicRequestMessage::SchedulingEnvironment(
                strategic_scheduling_environment_commands,
            ) => match strategic_scheduling_environment_commands {
                StrategicSchedulingEnvironmentCommands::UserStatus(strategic_user_status_codes) => {
                    let scheduling_environment_lock =
                        &mut self.scheduling_environment.lock().unwrap();

                    let user_status_codes = &mut scheduling_environment_lock
                        .work_orders
                        .inner
                        .get_mut(&strategic_user_status_codes.work_order_number)
                        .with_context(|| {
                            format!(
                                "{:?} is not found for {:?}",
                                strategic_user_status_codes.work_order_number, self.asset
                            )
                        })?
                        .work_order_analytic
                        .user_status_codes;

                    if let Some(sece) = strategic_user_status_codes.sch {
                        user_status_codes.sece = sece;
                    }
                    if let Some(sch) = strategic_user_status_codes.awsc {
                        user_status_codes.sch = sch;
                    }
                    if let Some(awsc) = strategic_user_status_codes.sece {
                        user_status_codes.awsc = awsc;
                    }
                    Ok(StrategicResponseMessage::Success)
                }
            },
        }
    }
}

impl Handler<SetAddr> for StrategicAgent {
    type Result = Result<()>;

    fn handle(&mut self, msg: SetAddr, _ctx: &mut actix::Context<Self>) -> Self::Result {
        match msg {
            SetAddr::Tactical(addr) => {
                self.tactical_agent_addr = Some(addr);
                Ok(())
            }
            _ => {
                bail!("Could not set the tactical Addr")
            }
        }
    }
}

impl Handler<UpdateWorkOrderMessage> for StrategicAgent {
    type Result = ();

    fn handle(
        &mut self,
        update_work_order: UpdateWorkOrderMessage,
        _ctx: &mut actix::Context<Self>,
    ) {
        let locked_scheduling_environment = self.scheduling_environment.lock().unwrap();

        let periods = locked_scheduling_environment.periods().clone();

        let work_order: &WorkOrder = locked_scheduling_environment
            .work_orders()
            .inner
            .get(&update_work_order.0)
            .unwrap();

        let optimized_work_order_builder = StrategicParameterBuilder::new();

        let optimized_work_order = optimized_work_order_builder
            .build_from_work_order(work_order, &periods)
            .build();
        assert!(work_order.work_order_analytic.work_order_weight == optimized_work_order.weight);
        if let Some(period) =
            Into::<MaterialStatus>::into(work_order.work_order_analytic.user_status_codes.clone())
                .period_delay(&periods)
        {
            assert!(&optimized_work_order.excluded_periods.contains(&period));
        }

        self.strategic_algorithm
            .strategic_parameters
            .strategic_work_order_parameters
            .insert(update_work_order.0, optimized_work_order);
    }
}

impl Handler<SolutionExportMessage> for StrategicAgent {
    type Result = Option<AgentExports>;

    fn handle(&mut self, _msg: SolutionExportMessage, _ctx: &mut Self::Context) -> Self::Result {
        let mut strategic_solution = HashMap::new();
        for (work_order_number, scheduled_period) in
            self.strategic_algorithm.strategic_periods().iter()
        {
            strategic_solution.insert(*work_order_number, scheduled_period.clone().unwrap());
        }
        Some(AgentExports::Strategic(strategic_solution))
    }
}
