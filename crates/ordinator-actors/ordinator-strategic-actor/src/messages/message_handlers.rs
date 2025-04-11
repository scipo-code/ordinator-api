use std::any::type_name;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use colored::Colorize;
use ordinator_actor_core::algorithm::LoadOperation;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_orchestrator_actor_traits::ActorSpecific;
use ordinator_orchestrator_actor_traits::MessageHandler;
use ordinator_orchestrator_actor_traits::SharedSolutionTrait;
use ordinator_orchestrator_actor_traits::StateLink;
use tracing::Level;
use tracing::event;

use super::StrategicRequestMessage;
use super::StrategicResponseMessage;
use super::StrategicResponsePeriods;
use super::StrategicResponseStatus;
use super::StrategicSchedulingEnvironmentCommands;
use super::StrategicStatusMessage;
use crate::StrategicActor;
use crate::StrategicOptions;
use crate::algorithm::ScheduleWorkOrder;
use crate::algorithm::StrategicUtils;
use crate::algorithm::strategic_parameters::WorkOrderParameter;
use crate::algorithm::strategic_parameters::WorkOrderParameterBuilder;
use crate::algorithm::strategic_solution::StrategicSolution;
use crate::messages::StrategicRequestScheduling;
use crate::messages::StrategicResponseScheduling;

impl<Ss> MessageHandler for StrategicActor<Ss>
where
    Ss: SharedSolutionTrait<Strategic = StrategicSolution>,
{
    type Req = StrategicRequestMessage;
    type Res = StrategicResponseMessage;

    fn handle_request_message(&mut self, strategic_request_message: Self::Req)
    -> Result<Self::Res>
    {
        let strategic_response = match strategic_request_message {
            StrategicRequestMessage::Status(strategic_status_message) => {
                match strategic_status_message {
                    StrategicStatusMessage::General => {

                        StrategicResponseStatus::from
                        let strategic_objective_value = &self.algorithm.solution.objective_value;

                        let strategic_parameters = &self.algorithm.parameters;

                        let number_of_strategic_work_orders =
                            strategic_parameters.strategic_work_order_parameters.len();

                        let asset = self.agent_id.asset();

                        let number_of_periods = self.algorithm.parameters.strategic_periods.len();

                        // You need to generate a trait for the `objective value`
                        // I would rather want to have a `from` implementation on this type than
                        // make something like this.
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
                    // This could be created by the `Orchestrator` instead
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

                        // let work_orders_by_period: HashMap<WorkOrderNumber, WorkOrderResponse> =
                        //     self.algorithm
                        //         .solution
                        //         .strategic_scheduled_work_orders
                        //         .iter()
                        //         .filter(|(_, sch_per)| match sch_per {
                        //             Some(scheduled_period) => {
                        //                 scheduled_period.period_string() == period
                        //             }
                        //             None => false,
                        //         })
                        //         .map(|(work_order_number, _)| {
                        //             let work_orders =
                        //                 self.scheduling_environment.lock().unwrap().work_orders;

                        //             let work_order_configurations =
                        //                 &work_orders.work_order_configurations;

                        //             let work_order =
                        //
                        // &work_orders.inner.get(work_order_number).unwrap().clone();

                        //             let work_order_response = WorkOrderResponse::new(
                        //                 work_order,
                        //                 (**self.algorithm.loaded_shared_solution).clone().into(),
                        //                 &self
                        //                     .scheduling_environment
                        //                     .lock()
                        //                     .unwrap()
                        //                     .time_environment
                        //                     .strategic_periods
                        //                     .clone(),
                        //                 work_order_configurations,
                        //             );
                        //             (*work_order_number, work_order_response)
                        //         })
                        //         .collect();

                        bail!("The endpoints are being refactored")
                    }
                    // This should be created in a different way. I think that you should rely
                    // on a lot of `From` implementations here. I think that is the best approach
                    // to fixing this issue.
                    StrategicStatusMessage::WorkOrder(work_order_number) => {
                        // TODO [ ]
                        // Make a `From` implementation.
                        // let strategic_solution_for_specific_work_order = self
                        //     .algorithm
                        //     .solution
                        //     .strategic_scheduled_work_orders
                        //     .get(&work_order_number)
                        //     .with_context(|| format!("{:?} not found in", work_order_number,))?;

                        // let strategic_parameter = self
                        //     .algorithm
                        //     .parameters
                        //     .strategic_work_order_parameters
                        //     .get(&work_order_number)
                        //     .with_context(|| {
                        //         format!(
                        //             "{:?} does not have a {}",
                        //             work_order_number,
                        //             std::any::type_name::<WorkOrderParameter>(),
                        //         )
                        //     })?;

                        // let locked_in_period = &strategic_parameter.locked_in_period;
                        // let excluded_from_period = &strategic_parameter.excluded_periods;

                        // let strategic_api_solution = StrategicApiSolution {
                        //     solution: strategic_solution_for_specific_work_order.clone(),
                        //     locked_in_period: locked_in_period.clone(),
                        //     excluded_from_period: excluded_from_period.clone(),
                        // };

                        // let work_orders_in_period =
                        //     WorkOrdersStatus::SingleSolution(strategic_api_solution);

                        // let strategic_response_message =
                        //     StrategicResponseMessage::WorkOrder(work_orders_in_period);

                        // Ok(strategic_response_message)
                        todo!()
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

                let strategic_options =
                    StrategicOptions::from((&self.configurations.load(), &self.agent_id));
                self.algorithm
                    .calculate_objective_value(&strategic_options)?;
                event!(Level::INFO, strategic_objective_value = ?self.algorithm.solution.objective_value);
                Ok(StrategicResponseMessage::Scheduling(scheduling_output))
            }
            StrategicRequestMessage::Resources(resources_message) => {
                let resources_output = self.algorithm.update_resources_state(resources_message);

                let strategic_options =
                    StrategicOptions::from((&self.configurations.load(), &self.agent_id));
                self.algorithm
                    .calculate_objective_value(&strategic_options)?;
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
            // Make from implementations for all of this
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

                        // This should ideally be encapsulated into the a method on the WorkOrder
                        // that accepts a StrategicUserStatusCodes
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

                        self.algorithm
                            .update_loadings(unscheduled_resources, LoadOperation::Sub);

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
                            .update_loadings(scheduled_resources, LoadOperation::Add);
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
        let strategic_options =
            StrategicOptions::from((&self.configurations.load(), &self.agent_id));
        self.algorithm
            .calculate_objective_value(&strategic_options)?;

        strategic_response
    }

    fn handle_state_link(&mut self, msg: StateLink) -> Result<StrategicResponseMessage>
    {
        match msg {
            StateLink::WorkOrders(agent_specific) => {
                match agent_specific {
                    ActorSpecific::Strategic(changed_work_orders) => {
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

                            // This is not made in the best way. I can sense it. What should I do
                            // about it? I think that the best way is to
                            // make something that can schedule everything.
                            let options = StrategicOptions::from((
                                &self.configurations.load(),
                                &self.agent_id,
                            ));
                            let strategic_parameter = WorkOrderParameter::builder()
                                .with_scheduling_environment(
                                    work_order,
                                    &scheduling_environment_guard
                                        .time_environment
                                        .strategic_periods,
                                    &options,
                                )
                                .build();

                            self.algorithm
                                .parameters
                                .strategic_work_order_parameters
                                .insert(work_order_number, strategic_parameter);
                        }
                    }
                }

                Ok(StrategicResponseMessage::Status(()))
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
