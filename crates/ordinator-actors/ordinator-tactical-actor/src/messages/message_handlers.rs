use anyhow::Context;
use anyhow::Result;
use ordinator_orchestrator_actor_traits::ActorSpecific;
use ordinator_orchestrator_actor_traits::CommandHandler;
use ordinator_orchestrator_actor_traits::StateLink;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder;

use super::TacticalRequestMessage;
use super::TacticalResponseMessage;
use crate::TacticalActor;
use crate::algorithm::tactical_parameters::TacticalParameters;
use crate::algorithm::tactical_parameters::create_tactical_parameter;
use crate::algorithm::tactical_resources::TacticalResources;
use crate::algorithm::tactical_solution::TacticalSolution;

// TODO [ ]
// Make a TacticalAgent here! I believe that this is the best appraoch. The only
// way that you will find out is by creating the system in the new way you are
// so much out of the water here that getting it to compile and run is the only
// way to consolidate your knowledge.
impl<Ss> CommandHandler for TacticalActor<Ss>
where
    Ss: SystemSolutions<Tactical = TacticalSolution>,
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
            TacticalRequestMessage::Resource(_tactical_resources_message) => {
                // let resource_response = self
                //     .update_resources_state(tactical_resources_message)
                //     .unwrap();
                Ok(TacticalResponseMessage::FreeStringResponse(
                    "Implement the Update code here.".to_string(),
                ))
            }
            TacticalRequestMessage::Time(_tactical_time_message) => {
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

    fn handle_state_link(&mut self, state_link: StateLink) -> Result<Self::Res>
    {
        match state_link {
            StateLink::WorkOrders(agent_specific) => match agent_specific {
                ActorSpecific::Strategic(changed_work_orders) => {
                    let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                    let work_orders = &scheduling_environment_guard.work_orders.inner.clone();
                    let work_order_configurations = &scheduling_environment_guard
                        .worker_environment
                        .actor_specification
                        .get(self.actor_id.asset())
                        .unwrap()
                        .work_order_configurations
                        .clone();

                    drop(scheduling_environment_guard);
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
                        // exploit to make sense of this. You are not creating this in the best
                        // possible way at the moment I think. There is a
                        // better approach for dealing with this.
                        //
                        // You should wrap this up in the `Interface`

                        let tactical_parameter =
                            create_tactical_parameter(work_order, work_order_configurations)?;

                        // It is only the algorithm that can modify parameters. Not the the Actor
                        // directly you should fix this issue soon. What
                        // about the code. You should make the interface
                        // here for interacting with the algorithm.
                        self.algorithm
                            .parameters
                            .tactical_work_orders
                            .insert(work_order_number, tactical_parameter);

                        self.algorithm
                            .solution
                            .tactical_work_orders
                            .0
                            .insert(work_order_number, WhereIsWorkOrder::NotScheduled);
                    }
                    Ok(TacticalResponseMessage::FreeStringResponse(
                        "Updated StateLink::WorkOrders".to_string(),
                    ))
                }
            },
            StateLink::WorkerEnvironment => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                // The issue here is that `from` does not consume the value. But instead work
                // with the reference.
                let tactical_resources =
                    TacticalResources::from((&scheduling_environment_guard, &self.actor_id));
                drop(scheduling_environment_guard);

                self.algorithm
                    .parameters
                    .tactical_capacity
                    .update_resources(tactical_resources);

                // TODO [ ]
                // Turn this into a JSON
                Ok(TacticalResponseMessage::FreeStringResponse(
                    "Updated StateLink::WorkerEnvironment".to_string(),
                ))
            }

            StateLink::TimeEnvironment => {
                todo!()
            }
        }
    }
}

// impl<Ss> TacticalActor<TacticalRequestMessage, TacticalResponseMessage,
// TacticalAlgorithm<Ss>> {
//     fn update_resources_state(
//         &mut self,
//         resource_message: TacticalResourceRequest,
//     ) -> Result<TacticalResourceResponse>
//     {
//         match resource_message {
// TacticalResourceRequest::SetResources(resources) => {
//     // The resources should be initialized together with the Agent itself
//     let mut count = 0;
//     for (resource, days) in resources.resources {
//         for (day, capacity) in days.days {
//             let day: Day = match self
//                 .algorithm
//                 .parameters
//                 .tactical_days
//                 .iter()
//                 .find(|d| **d == day)
//             {
//                 Some(day) => {
//                     count += 1;
//                     day.clone()
//                 }
//                 None => {
//                     bail!("Day not found in the tactical days".to_string(),);
//                 }
//             };

//             *self.algorithm.capacity_mut(&resource, &day) = capacity;
//         }
//     }
//     Ok(TacticalResourceResponse::UpdatedResources(count))
// TacticalResourceRequest::GetLoadings {
//     days_end: _,
//     select_resources: _,
// } => {
//     let loadings = self.algorithm.solution.tactical_loadings.clone();

//     let tactical_response_resources =
// TacticalResourceResponse::Loading(loadings);
//     Ok(tactical_response_resources)
// }
// TacticalResourceRequest::GetCapacities {
//     days_end: _,
//     select_resources: _,
// } => {
//     let capacities = self.algorithm.parameters.tactical_capacity.clone();

//     let tactical_response_resources =
// TacticalResourceResponse::Capacity(capacities);

//     Ok(tactical_response_resources)
// }
// TacticalResourceRequest::GetPercentageLoadings {
//     days_end: _,
//     resources: _,
// } => {
//     let capacities = &self.algorithm.parameters.tactical_capacity;
//     let loadings = &self.algorithm.solution.tactical_loadings;

//     let tactical_response_resources =
//         TacticalResourceResponse::Percentage((capacities.clone(),
// loadings.clone()));     Ok(tactical_response_resources)
// }
//             _ => todo!(),
//         }
//     }
// }
