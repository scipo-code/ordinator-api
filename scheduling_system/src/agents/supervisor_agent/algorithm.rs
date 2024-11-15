use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, MutexGuard},
};

use anyhow::{Context, Result};
use arc_swap::Guard;
use shared_types::{
    scheduling_environment::{
        time_environment::period::Period,
        work_order::{
            operation::{operation_info::NumberOfPeople, ActivityNumber},
            WorkOrderActivity, WorkOrderNumber,
        },
        worker_environment::resources::Resources,
        SchedulingEnvironment,
    },
    supervisor::{
        supervisor_response_resources::SupervisorResponseResources,
        supervisor_response_scheduling::SupervisorResponseScheduling,
        supervisor_response_time::SupervisorResponseTime, SupervisorObjectiveValue,
    },
};

#[allow(unused_imports)]
use tracing::{event, Level};

use crate::agents::{
    traits::LargeNeighborHoodSearch, ArcSwapSharedSolution, GetMarginalFitness, SharedSolution,
    SupervisorSolution,
};

use super::delegate::Delegate;

#[derive(Eq, PartialEq, PartialOrd, Ord, Debug, Clone)]
pub struct MarginalFitness(pub u64);

impl MarginalFitness {
    pub const MAX: Self = Self(u64::MAX);
}

impl Default for MarginalFitness {
    fn default() -> Self {
        Self(u64::MAX)
    }
}
// pub struct MarginalFitness(pub Arc<AtomicUsize>);

// impl MarginalFitness {
//     pub fn inner(&self) -> u64 {
//         self.0
//     }

//     pub fn store(&self, value: usize) {
//         self.0.store(value, std::sync::atomic::Ordering::SeqCst)
//     }

//     pub fn compare(&self, other: &Self) -> Ordering {
//         let self_value = self.inner();
//         let other_value = other.inner();

//         if self_value == other_value {
//             return Ordering::Equal;
//         } else if self_value > other_value {
//             return Ordering::Greater;
//         } else {
//             return Ordering::Less;
//         }
//     }
// }

// impl Default for MarginalFitness {
//     fn default() -> Self {
//         MarginalFitness(Arc::new(AtomicUsize::new(usize::MAX)))
//     }
// }

pub struct SupervisorSchedulingRequest;
pub struct SupervisorResourceRequest;
pub struct SupervisorTimeRequest;

pub struct SupervisorAlgorithm {
    pub objective_value: SupervisorObjectiveValue,
    pub resources: Vec<Resources>,
    pub supervisor_parameters: SupervisorParameters,
    pub supervisor_solution: SupervisorSolution,
    arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
    pub loaded_shared_solution: Guard<Arc<SharedSolution>>,
}

pub struct SupervisorParameters {
    pub supervisor_work_orders:
        HashMap<WorkOrderNumber, HashMap<ActivityNumber, SupervisorParameter>>,
    pub supervisor_periods: Vec<Period>,
}

impl SupervisorParameters {
    pub fn new(supervisor_periods: Vec<Period>) -> Self {
        Self {
            supervisor_work_orders: HashMap::new(),
            supervisor_periods,
        }
    }

    pub(crate) fn supervisor_parameter(
        &self,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<&SupervisorParameter> {
        Ok(self.supervisor_work_orders
            .get(&work_order_activity.0)
            .context(format!("WorkOrderNumber: {:?} was not part of the SupervisorParameters", work_order_activity.0))?
            .get(&work_order_activity.1)
            .context(format!("WorkOrderNumber: {:?} with ActivityNumber: {:?} was not part of the SupervisorParameters", work_order_activity.0, work_order_activity.1))?)
    }

    pub(crate) fn create(
        &mut self,
        scheduling_environment_lock: &MutexGuard<SchedulingEnvironment>,
        work_order_activity: &WorkOrderActivity,
    ) {
        let operation = scheduling_environment_lock.operation(work_order_activity);

        let supervisor_parameter =
            SupervisorParameter::new(operation.resource.clone(), operation.operation_info.number);
        self.supervisor_work_orders
            .entry(work_order_activity.0)
            .or_insert_with(HashMap::new)
            .insert(work_order_activity.1, supervisor_parameter);
    }
}

pub struct SupervisorParameter {
    pub resource: Resources,
    pub number: NumberOfPeople,
}

impl SupervisorParameter {
    pub fn new(resource: Resources, number: NumberOfPeople) -> Self {
        Self { resource, number }
    }
}

impl SupervisorAlgorithm {
    pub fn new(
        resources: Vec<Resources>,
        arc_swap_shared_solution: Arc<ArcSwapSharedSolution>,
        supervisor_periods: &[Period],
    ) -> Self {
        let loaded_shared_solution = arc_swap_shared_solution.0.load();
        Self {
            objective_value: SupervisorObjectiveValue::default(),
            resources,
            supervisor_parameters: SupervisorParameters::new(supervisor_periods.to_vec()),
            supervisor_solution: SupervisorSolution::default(),
            arc_swap_shared_solution,
            loaded_shared_solution,
        }
    }

    pub fn load_shared_solution(&mut self) {
        self.loaded_shared_solution = self.arc_swap_shared_solution.0.load();
    }

    pub fn make_atomic_pointer_swap(&self) {
        // Performance enhancements:
        // * COW:
        //      #[derive(Clone)]
        //      struct SharedSolution<'a> {
        //          tactical: Cow<'a, TacticalSolution>,
        //          // other fields...
        //      }
        //
        // * Reuse the old SharedSolution, cloning only the fields that are needed.
        //     let shared_solution = Arc::new(SharedSolution {
        //             tactical: self.tactical_solution.clone(),
        //             // Copy over other fields without cloning
        //             ..(**old).clone()
        //         });
        self.arc_swap_shared_solution.0.rcu(|old| {
            let mut shared_solution = (**old).clone();
            shared_solution.supervisor = self.supervisor_solution.clone();
            Arc::new(shared_solution)
        });
    }
}

impl LargeNeighborHoodSearch for SupervisorAlgorithm {
    type BetterSolution = SupervisorObjectiveValue;
    type SchedulingRequest = SupervisorSchedulingRequest;
    type SchedulingResponse = SupervisorResponseScheduling;
    type ResourceRequest = SupervisorResourceRequest;
    type ResourceResponse = SupervisorResponseResources;
    type TimeRequest = SupervisorTimeRequest;
    type TimeResponse = SupervisorResponseTime;

    type SchedulingUnit = WorkOrderNumber;

    fn calculate_objective_value(&mut self) -> Self::BetterSolution {
        let assigned_woas = &self.supervisor_solution.number_of_assigned_work_orders();

        let all_woas: HashSet<_> = self.supervisor_solution.get_work_order_activities();

        assert!(is_assigned_part_of_all(assigned_woas, &all_woas));

        let mut intermediate = assigned_woas.len() as f64 / all_woas.len() as f64;
        if intermediate.is_nan() {
            intermediate = 0.0;
        };

        let objective_value = (intermediate * 1000.0) as u64;

        self.objective_value = objective_value;
        objective_value
    }

    fn schedule(&mut self) -> Result<()> {
        'next_work_order_activity: for work_order_activity in
            &self.supervisor_solution.get_work_order_activities()
        {
            let number = self
                .supervisor_parameters
                .supervisor_work_orders
                .get(&work_order_activity.0)
                .and_then(|activities| activities.get(&work_order_activity.1))
                .expect("The SupervisorParameter should always be available")
                .number;

            let mut operational_status_by_work_order_activity = self
                .supervisor_solution
                .operational_status_by_work_order_activity(&work_order_activity);

            let operational = &self.loaded_shared_solution.operational;

            operational_status_by_work_order_activity.sort_by_cached_key(|(agent_id, _)| {
                match operational
                    .marginal_fitness(agent_id, work_order_activity) {
                        Ok(marginal_fitness) => marginal_fitness,
                        Err(e) => {
                            event!(Level::WARN, operational_agent_marginal_fitness_error = ?e, "Could be that the OperationalAgent did not have time to initialize. A bug could be hiding here");
                            MarginalFitness::MAX
                        },
                    }
            });

            let number_of_assigned = operational_status_by_work_order_activity
                .iter()
                .filter(|(_, delegate)| *delegate == Delegate::Assign)
                .count() as u64;

            let mut remaining_to_assign = number - number_of_assigned;

            event!(Level::WARN, remaining_to_assign = ?remaining_to_assign);
            for (agent_id, delegate_status) in &mut operational_status_by_work_order_activity {
                if *delegate_status != Delegate::Assess {
                    continue;
                }

                let marginal_fitness = self
                    .loaded_shared_solution
                    .operational
                    .marginal_fitness(&agent_id, work_order_activity)
                    .unwrap_or_default();

                if marginal_fitness == MarginalFitness::MAX {
                    continue 'next_work_order_activity;
                }

                if remaining_to_assign >= 1 {
                    remaining_to_assign -= 1;
                    delegate_status.state_change_to_assign();
                } else {
                    if *delegate_status == Delegate::Assign {
                        continue;
                    }

                    delegate_status.state_change_to_unassign()
                }
            }
        }
        Ok(())
    }

    fn unschedule(&mut self, work_order_number: Self::SchedulingUnit) -> Result<()> {
        self.supervisor_solution
            .turn_work_order_into_delegate_assess(work_order_number);
        Ok(())
    }

    fn update_scheduling_state(
        &mut self,
        _message: Self::SchedulingRequest,
    ) -> Result<Self::SchedulingResponse> {
        todo!()
    }

    fn update_time_state(&mut self, _message: Self::TimeRequest) -> Result<Self::TimeResponse> {
        todo!()
    }

    fn update_resources_state(
        &mut self,
        _message: Self::ResourceRequest,
    ) -> Result<Self::ResourceResponse> {
        todo!()
    }
}

fn is_assigned_part_of_all(
    assigned_woas: &HashSet<(WorkOrderNumber, ActivityNumber)>,
    all_woas: &HashSet<(WorkOrderNumber, ActivityNumber)>,
) -> bool {
    assigned_woas
        .iter()
        .map(|(wo, ac)| all_woas.contains(&(*wo, *ac)))
        .all(|present_woa| present_woa)
}
