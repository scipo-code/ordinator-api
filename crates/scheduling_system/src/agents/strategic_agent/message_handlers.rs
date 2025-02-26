use std::any::type_name;
use std::collections::HashMap;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use colored::Colorize;
use priority_queue::PriorityQueue;
use shared_types::agents::strategic::requests::strategic_request_scheduling_message::StrategicRequestScheduling;
use shared_types::agents::strategic::requests::strategic_request_status_message::StrategicStatusMessage;
use shared_types::agents::strategic::responses::strategic_response_periods::StrategicResponsePeriods;
use shared_types::agents::strategic::responses::strategic_response_scheduling::StrategicResponseScheduling;
use shared_types::agents::strategic::responses::strategic_response_status::StrategicResponseStatus;
use shared_types::agents::strategic::StrategicSchedulingEnvironmentCommands;
use shared_types::agents::strategic::{StrategicRequestMessage, StrategicResponseMessage};
use shared_types::orchestrator::StrategicApiSolution;
use shared_types::orchestrator::WorkOrderResponse;
use shared_types::orchestrator::WorkOrdersStatus;
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use tracing::event;
use tracing::Level;

use crate::agents::traits::ActorBasedLargeNeighborhoodSearch;
use crate::agents::Agent;
use crate::agents::AgentSpecific;
use crate::agents::Algorithm;
use crate::agents::MessageHandler;
use crate::agents::StateLink;
use crate::agents::StrategicSolution;

use super::algorithm::strategic_parameters::StrategicParameters;
use super::algorithm::strategic_parameters::WorkOrderParameter;
use super::algorithm::strategic_parameters::WorkOrderParameterBuilder;
use super::algorithm::ScheduleWorkOrder;

type StrategicAlgorithm =
    Algorithm<StrategicSolution, StrategicParameters, PriorityQueue<WorkOrderNumber, u64>>;

impl MessageHandler
    for Agent<StrategicAlgorithm, StrategicRequestMessage, StrategicResponseMessage>
{
    type Req = StrategicRequestMessage;
    type Res = StrategicResponseMessage;

    fn handle_request_message(
        &mut self,
        strategic_request_message: Self::Req,
    ) -> Result<Self::Res> {
        let strategic_response = match strategic_request_message {
            StrategicRequestMessage::Status(strategic_status_message) => {
                match strategic_status_message {
                    StrategicStatusMessage::General => {
                        let strategic_objective_value = &self.algorithm.solution.objective_value;

                        let strategic_parameters = &self.algorithm.parameters;

                        let number_of_strategic_work_orders =
                            strategic_parameters.strategic_work_order_parameters.len();

                        let asset = self.agent_id.asset();

                        let number_of_periods = self.algorithm.parameters.strategic_periods.len();

                        // Yes so you use
                        let strategic_response_status = StrategicResponseStatus::new(
                            asset.clone(),
                            (strategic_objective_value.clone()).into(),
                            number_of_strategic_work_orders,
                            number_of_periods,
                        );

                        let strategic_response_message =
                            StrategicResponseMessage::Status(strategic_response_status);
                        Ok(strategic_response_message)
                    }
                    StrategicStatusMessage::Period(period) => {
                        if !self
                            .algorithm
                            .parameters
                            .strategic_periods
                            .iter()
                            .map(|period| period.period_string())
                            .collect::<Vec<_>>()
                            .contains(&period)
                        {
                            bail!("Period not found in the the scheduling environment".to_string());
                        }

                        let work_orders_by_period: HashMap<WorkOrderNumber, WorkOrderResponse> =
                            self.algorithm
                                .solution
                                .strategic_scheduled_work_orders
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
                                        .work_orders
                                        .inner
                                        .get(work_order_number)
                                        .unwrap()
                                        .clone();

                                    let work_order_response = WorkOrderResponse::new(
                                        &work_order,
                                        (**self.algorithm.loaded_shared_solution).clone().into(),
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
                    StrategicStatusMessage::WorkOrder(work_order_number) => {
                        let strategic_solution_for_specific_work_order = self
                            .algorithm
                            .solution
                            .strategic_scheduled_work_orders
                            .get(&work_order_number)
                            .with_context(|| {
                                format!(
                                    "{:?} not found in {}",
                                    work_order_number,
                                    std::any::type_name::<StrategicAlgorithm>()
                                )
                            })?;

                        let strategic_parameter = self
                            .algorithm
                            .parameters
                            .strategic_work_order_parameters
                            .get(&work_order_number)
                            .with_context(|| {
                                format!(
                                    "{:?} does not have a {} in {}",
                                    work_order_number,
                                    std::any::type_name::<WorkOrderParameter>(),
                                    std::any::type_name::<StrategicAlgorithm>()
                                )
                            })?;

                        let locked_in_period = &strategic_parameter.locked_in_period;
                        let excluded_from_period = &strategic_parameter.excluded_periods;

                        let strategic_api_solution = StrategicApiSolution {
                            solution: strategic_solution_for_specific_work_order.clone(),
                            locked_in_period: locked_in_period.clone(),
                            excluded_from_period: excluded_from_period.clone(),
                        };

                        let work_orders_in_period =
                            WorkOrdersStatus::SingleSolution(strategic_api_solution);

                        let strategic_response_message =
                            StrategicResponseMessage::WorkOrder(work_orders_in_period);

                        Ok(strategic_response_message)
                    }
                }
            }
            StrategicRequestMessage::Scheduling(scheduling_message) => {
                let scheduling_output: StrategicResponseScheduling = self
                    .algorithm
                    .update_scheduling_state(scheduling_message)
                    .with_context(|| {
                        format!(
                            "{} was not Resolved",
                            type_name::<StrategicRequestScheduling>()
                                .split("::")
                                .last()
                                .unwrap()
                                .bright_red()
                        )
                    })?;

                self.algorithm.calculate_objective_value()?;
                event!(Level::INFO, strategic_objective_value = ?self.algorithm.solution.objective_value);
                Ok(StrategicResponseMessage::Scheduling(scheduling_output))
            }
            StrategicRequestMessage::Resources(resources_message) => {
                let resources_output = self.algorithm.update_resources_state(resources_message);

                self.algorithm.calculate_objective_value()?;
                event!(Level::INFO, strategic_objective_value = ?self.algorithm.solution.objective_value);
                Ok(StrategicResponseMessage::Resources(
                    resources_output.unwrap(),
                ))
            }
            StrategicRequestMessage::Periods(periods_message) => {
                let mut scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let periods = &mut scheduling_environment_guard
                    .time_environment
                    .strategic_periods;

                for period_id in periods_message.periods.iter() {
                    if periods.last().unwrap().id() + 1 == *period_id {
                        let new_period =
                            periods.last().unwrap().clone() + chrono::Duration::weeks(2);
                        periods.push(new_period);
                    } else {
                        event!(Level::ERROR, "periods not handled correctly");
                    }
                }
                self.algorithm.parameters.strategic_periods = periods.to_vec();
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

                    for work_order_number in &strategic_user_status_codes.work_order_numbers {
                        let work_order = scheduling_environment_lock
                            .work_orders
                            .inner
                            .get_mut(work_order_number)
                            .with_context(|| {
                                format!(
                                    "{:?} is not found for {:?}",
                                    work_order_number,
                                    self.agent_id.asset()
                                )
                            })?;

                        // This should ideally be encapsulated into the a method on the WorkOrder that accepts a StrategicUserStatusCodes
                        let user_status_codes =
                            &mut work_order.work_order_analytic.user_status_codes;

                        if let Some(sece) = strategic_user_status_codes.sece {
                            user_status_codes.sece = sece;
                        }
                        if let Some(sch) = strategic_user_status_codes.sch {
                            user_status_codes.sch = sch;
                        }
                        if let Some(awsc) = strategic_user_status_codes.awsc {
                            user_status_codes.awsc = awsc;
                        }

                        work_order.work_order_value();

                        let last_period =
                            self.algorithm.parameters.strategic_periods.last().cloned();

                        let unscheduled_period = self
                            .algorithm
                            .solution
                            .strategic_scheduled_work_orders
                            .insert(*work_order_number, last_period.clone())
                            .expect("WorkOrderNumber should always be present")
                            .expect(
                                "All WorkOrders should be scheduled in between ScheduleIteration loops",
                            );

                        let work_load = self
                            .algorithm
                            .parameters
                            .strategic_work_order_parameters
                            .get(work_order_number)
                            .unwrap()
                            .work_load
                            .clone();

                        let unscheduled_resources = self
                            .algorithm
                            .determine_best_permutation(
                                work_load.clone(),
                                &unscheduled_period,
                                ScheduleWorkOrder::Unschedule,
                            )
                            .with_context(|| {
                                format!(
                                    "{:?}\nin period {:?}\ncould not be {:?}",
                                    work_order_number,
                                    unscheduled_period,
                                    ScheduleWorkOrder::Unschedule
                                )
                            })?
                            .expect("It should always be possible to release resources");

                        self.algorithm.update_loadings(
                            unscheduled_resources,
                            shared_types::LoadOperation::Sub,
                        );

                        let scheduled_resources = self
                            .algorithm
                            .determine_best_permutation(
                                work_load,
                                &last_period.unwrap(),
                                ScheduleWorkOrder::Forced,
                            )
                            .with_context(|| {
                                format!(
                                    "{:?}\nin period {:?}\ncould not be {:?}",
                                    work_order_number,
                                    unscheduled_period,
                                    ScheduleWorkOrder::Forced
                                )
                            })?
                            .expect("It should always be possible to release resources");

                        self.algorithm
                            .update_loadings(scheduled_resources, shared_types::LoadOperation::Add);
                    }

                    // Signal Orchestrator that the it should tell all actor to update work orders
                    self.notify_orchestrator
                        .notify_all_agents_of_work_order_change(
                            strategic_user_status_codes.work_order_numbers,
                            &self.agent_id.asset(),
                        )
                        .context("Could not notify Orchestrator")?;

                    Ok(StrategicResponseMessage::Success)
                }
            },
        };
        self.algorithm.calculate_objective_value()?;
        strategic_response
    }

    fn handle_state_link(&mut self, msg: StateLink) -> Result<()> {
        match msg {
            StateLink::WorkOrders(agent_specific) => {
                match agent_specific {
                    AgentSpecific::Strategic(changed_work_orders) => {
                        for work_order_number in changed_work_orders {
                            let scheduling_environment_guard =
                                self.scheduling_environment.lock().unwrap();
                            let work_order = scheduling_environment_guard
                                .work_orders
                                .inner
                                .get(&work_order_number)
                                .with_context(|| {
                                    format!(
                                        "{:?} is not present in SchedulingEnvironment",
                                        work_order_number
                                    )
                                })?;

                            let strategic_parameter = WorkOrderParameterBuilder::new()
                                .build_from_work_order(
                                    work_order,
                                    &self.algorithm.parameters.strategic_periods,
                                )
                                .build();

                            self.algorithm
                                .parameters
                                .insert_strategic_parameter(work_order_number, strategic_parameter);
                        }
                    }
                }

                Ok(())
            }
            StateLink::WorkerEnvironment => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();
                let strategic_resources = scheduling_environment_guard
                    .worker_environment
                    .generate_strategic_resources(&self.algorithm.parameters.strategic_periods);

                self.algorithm
                    .parameters
                    .strategic_capacity
                    .update_resource_capacities(strategic_resources)
                    .expect("Could not update the StrategicResources");

                Ok(())
            }
            StateLink::TimeEnvironment => todo!(),
        }
    }
}
