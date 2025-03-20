use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use priority_queue::PriorityQueue;
use shared_types::agents::tactical::TacticalRequestMessage;
use shared_types::agents::tactical::TacticalResponseMessage;
use shared_types::agents::tactical::requests::tactical_resources_message::TacticalResourceRequest;
use shared_types::agents::tactical::responses::tactical_response_resources::TacticalResourceResponse;
use shared_types::scheduling_environment::time_environment::day::Day;
use shared_types::scheduling_environment::work_order::WorkOrderNumber;
use shared_types::scheduling_environment::worker_environment::EmptyFull;

use crate::agents::Actor;
use crate::agents::ActorSpecific;
use crate::agents::MessageHandler;
use crate::agents::StateLink;
use crate::agents::TacticalSolution;
use crate::agents::WhereIsWorkOrder;
use crate::agents::tactical_agent::algorithm::tactical_parameters::TacticalParameters;
use crate::agents::tactical_agent::algorithm::tactical_parameters::create_tactical_parameter;

impl MessageHandler
    for Actor<
        TacticalRequestMessage,
        TacticalResponseMessage,
        TacticalSolution,
        TacticalParameters,
        PriorityQueue<WorkOrderNumber, u64>,
    >
{
    type Req = TacticalRequestMessage;
    type Res = TacticalResponseMessage;

    fn handle_request_message(
        &mut self,
        tactical_request: TacticalRequestMessage,
    ) -> Result<Self::Res>
    {
        match tactical_request {
            TacticalRequestMessage::Status(_tactical_status_message) => {
                // let status_message = self.status().unwrap();
                // Ok(TacticalResponseMessage::Status(status_message))
                todo!()
            }
            TacticalRequestMessage::Scheduling(_tactical_scheduling_message) => {
                todo!()
            }
            TacticalRequestMessage::Resources(tactical_resources_message) => {
                let resource_response = self
                    .update_resources_state(tactical_resources_message)
                    .unwrap();
                Ok(TacticalResponseMessage::Resources(resource_response))
            }
            TacticalRequestMessage::Days(_tactical_time_message) => {
                todo!()
            }
            TacticalRequestMessage::Update => {
                todo!()
                // let locked_scheduling_environment =
                // &self.scheduling_environment.lock().unwrap();
                // let asset = &self.asset;

                // self.algorithm
                //     .create_tactical_parameters(locked_scheduling_environment, asset);
                // Ok(TacticalResponseMessage::Update)
            }
        }
    }

    fn handle_state_link(&mut self, state_link: StateLink) -> Result<()>
    {
        match state_link {
            StateLink::WorkOrders(agent_specific) => match agent_specific {
                ActorSpecific::Strategic(changed_work_orders) => {
                    let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                    let work_orders = &scheduling_environment_guard.work_orders.inner;

                    for work_order_number in changed_work_orders {
                        let work_order =
                            work_orders.get(&work_order_number).with_context(|| {
                                format!(
                                    "{:?} should always be present in {}",
                                    work_order_number,
                                    std::any::type_name::<TacticalParameters>()
                                )
                            })?;

                        // FIX
                        // The solution should also be updated here. Think about how you can make
                        // this generic.
                        // QUESTION
                        // Is this a good way of coding the program? I think that there is common
                        // behavior here that we are going to have to
                        // exploit to make sense of this.
                        let tactical_work_order = create_tactical_parameter(work_order);

                        // It is only the agent that can modify parameters. Not
                        self.algorithm
                            .parameters
                            .tactical_work_orders
                            .insert(work_order_number, tactical_work_order);

                        self.algorithm
                            .solution
                            .tactical_work_orders
                            .0
                            .insert(work_order_number, WhereIsWorkOrder::NotScheduled);
                    }
                    Ok(())
                }
            },
            StateLink::WorkerEnvironment => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();
                let tactical_resources = scheduling_environment_guard
                    .worker_environment
                    .generate_tactical_resources(
                        &self.algorithm.parameters.tactical_days,
                        EmptyFull::Full,
                    );

                self.algorithm
                    .parameters
                    .tactical_capacity
                    .update_resources(tactical_resources);

                Ok(())
            }

            StateLink::TimeEnvironment => {
                todo!()
            }
        }
    }
}

impl
    Actor<
        TacticalRequestMessage,
        TacticalResponseMessage,
        TacticalSolution,
        TacticalParameters,
        PriorityQueue<WorkOrderNumber, u64>,
    >
{
    fn update_resources_state(
        &mut self,
        resource_message: TacticalResourceRequest,
    ) -> Result<TacticalResourceResponse>
    {
        match resource_message {
            TacticalResourceRequest::SetResources(resources) => {
                // The resources should be initialized together with the Agent itself
                let mut count = 0;
                for (resource, days) in resources.resources {
                    for (day, capacity) in days.days {
                        let day: Day = match self
                            .algorithm
                            .parameters
                            .tactical_days
                            .iter()
                            .find(|d| **d == day)
                        {
                            Some(day) => {
                                count += 1;
                                day.clone()
                            }
                            None => {
                                bail!("Day not found in the tactical days".to_string(),);
                            }
                        };

                        *self.algorithm.capacity_mut(&resource, &day) = capacity;
                    }
                }
                Ok(TacticalResourceResponse::UpdatedResources(count))
            }
            TacticalResourceRequest::GetLoadings {
                days_end: _,
                select_resources: _,
            } => {
                let loadings = self.algorithm.solution.tactical_loadings.clone();

                let tactical_response_resources = TacticalResourceResponse::Loading(loadings);
                Ok(tactical_response_resources)
            }
            TacticalResourceRequest::GetCapacities {
                days_end: _,
                select_resources: _,
            } => {
                let capacities = self.algorithm.parameters.tactical_capacity.clone();

                let tactical_response_resources = TacticalResourceResponse::Capacity(capacities);

                Ok(tactical_response_resources)
            }
            TacticalResourceRequest::GetPercentageLoadings {
                days_end: _,
                resources: _,
            } => {
                let capacities = &self.algorithm.parameters.tactical_capacity;
                let loadings = &self.algorithm.solution.tactical_loadings;

                let tactical_response_resources =
                    TacticalResourceResponse::Percentage((capacities.clone(), loadings.clone()));
                Ok(tactical_response_resources)
            }
        }
    }
}
