pub mod algorithm;
pub mod display;
pub mod message_handlers;

use crate::agents::strategic_agent::algorithm::StrategicAlgorithm;
use crate::agents::traits::LargeNeighborhoodSearch;
use anyhow::Context;
use anyhow::Result;
use shared_types::scheduling_environment::SchedulingEnvironment;

use actix::prelude::*;
use shared_types::Asset;
use tracing::event;
use tracing::Level;

use std::sync::Arc;
use std::sync::Mutex;
use tracing::instrument;
use tracing::warn;

use super::orchestrator::NotifyOrchestrator;
use super::ScheduleIteration;
use crate::agents::tactical_agent::TacticalAgent;

pub struct StrategicAgent {
    asset: Asset,
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    pub strategic_algorithm: StrategicAlgorithm,
    pub tactical_agent_addr: Option<Addr<TacticalAgent>>,
    pub notify_orchestrator: NotifyOrchestrator,
}

impl Actor for StrategicAgent {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut actix::Context<Self>) {
        self.strategic_algorithm.populate_priority_queues();
        event!(
            Level::INFO,
            "StrategicAgent has started for asset: {}",
            self.asset
        );
        // Should this even be called here? I do not think that it should! ... Hmmm
        // It is basically creating the initial solution
        self.strategic_algorithm
            .schedule()
            .with_context(|| "Initial iteration of StrategicAlgorithm")
            .expect("StrategicAlgorithm.schedule() method failed");

        ctx.notify(ScheduleIteration::default())
    }

    fn stopped(&mut self, _ctx: &mut actix::Context<Self>) {
        println!("SchedulerAgent is stopped");
    }
}

impl StrategicAgent {
    pub fn new(
        asset: Asset,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
        strategic_algorithm: StrategicAlgorithm,
        tactical_agent_addr: Option<Addr<TacticalAgent>>,
        notify_orchestrator: NotifyOrchestrator,
    ) -> Self {
        Self {
            asset,
            scheduling_environment,
            strategic_algorithm,
            tactical_agent_addr,
            notify_orchestrator,
        }
    }
}

impl Handler<ScheduleIteration> for StrategicAgent {
    type Result = Result<()>;

    #[instrument(level = "trace", skip_all)]
    fn handle(
        &mut self,
        schedule_iteration: ScheduleIteration,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        self.strategic_algorithm.load_shared_solution();

        let rng: &mut rand::rngs::ThreadRng = &mut rand::thread_rng();

        self.strategic_algorithm
            .schedule_forced_work_orders()
            .expect("It should always be possible to force schedule work orders");

        self.strategic_algorithm.calculate_objective_value();

        let old_strategic_solution = self.strategic_algorithm.strategic_solution.clone();

        self.strategic_algorithm
            .unschedule_random_work_orders(50, rng)
            .expect("Unscheduling random work order should always be possible");

        // assert_eq!(self.strategic_algorithm.priority_queues.normal.len(), 1000);
        // assert!(self
        //     .strategic_algorithm
        //     .priority_queues
        //     .normal
        //     .iter()
        //     .all(|ele| {
        //         self.strategic_algorithm
        //             .strategic_solution
        //             .strategic_periods
        //             .get(&ele.0)
        //             .unwrap()
        //             .is_none()
        //     }));
        self.strategic_algorithm
            .schedule()
            .with_context(|| format!("{:?} of StrategicAlgorithm", schedule_iteration))
            .expect("StrategicAlgorithm::schedule method failed");

        // self.strategic_algorithm.swap_scheduled_work_orders(rng);
        // self.assert_aggregated_load().unwrap();

        self.strategic_algorithm.calculate_objective_value();

        if self
            .strategic_algorithm
            .strategic_solution
            .objective_value
            .objective_value
            < old_strategic_solution.objective_value.objective_value
        {
            self.strategic_algorithm.make_atomic_pointer_swap();

            event!(Level::INFO,
                strategic_objective_value = self.strategic_algorithm.strategic_solution.objective_value.objective_value,
                strategic_urgency = self.strategic_algorithm.strategic_solution.objective_value.urgency.1,
                strategic_resource_penalty = self.strategic_algorithm.strategic_solution.objective_value.resource_penalty.1,
                strategic_clustering_value = self.strategic_algorithm.strategic_solution.objective_value.clustering_value.1,
                scheduled_work_orders = ?self.strategic_algorithm.strategic_solution.strategic_periods.iter().filter(|ele| ele.1.is_some()).count(),
                total_work_orders = ?self.strategic_algorithm.strategic_solution.strategic_periods.len(),
                // percentage_utilization_by_period = ?self.strategic_algorithm.calculate_utilization(),
            );
        } else {
            self.strategic_algorithm.strategic_solution = old_strategic_solution;
        }

        ctx.wait(
            tokio::time::sleep(tokio::time::Duration::from_millis(
                dotenvy::var("STRATEGIC_THROTTLING")
                    .expect("The STRATEGIC_THROTTLING environment variable should always be set")
                    .parse::<u64>()
                    .expect("The STRATEGIC_THROTTLING environment variable have to be an u64 compatible type"),
            ))
            .into_actor(self),
        );

        ctx.notify(ScheduleIteration {
            loop_iteration: schedule_iteration.loop_iteration + 1,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use algorithm::strategic_parameters::StrategicClustering;
    use algorithm::ForcedWorkOrder;
    use operation::OperationBuilder;
    use shared_types::scheduling_environment::work_order::operation::ActivityNumber;
    use shared_types::scheduling_environment::work_order::operation::Work;
    use shared_types::strategic::strategic_request_scheduling_message::ScheduleChange;
    use shared_types::strategic::strategic_request_scheduling_message::StrategicSchedulingRequest;
    use shared_types::strategic::StrategicResources;
    use tests::algorithm::strategic_parameters::StrategicParameter;
    use tests::algorithm::strategic_parameters::StrategicParameters;
    use unloading_point::UnloadingPoint;

    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::str::FromStr;

    use crate::agents::ArcSwapSharedSolution;

    use super::{algorithm::PriorityQueues, *};
    use shared_types::scheduling_environment::worker_environment::resources::Resources;

    use shared_types::scheduling_environment::work_order::operation::Operation;
    use shared_types::scheduling_environment::work_order::*;
    use shared_types::scheduling_environment::WorkOrders;

    use shared_types::scheduling_environment::time_environment::period::Period;

    #[test]
    fn test_extract_state_to_scheduler_overview() {
        let mut operations: HashMap<u32, Operation> = HashMap::new();

        let unloading_point = UnloadingPoint::default();
        let operation_1 = OperationBuilder::new(
            ActivityNumber(10),
            unloading_point.clone(),
            Resources::MtnMech,
            Some(Work::from(1.0)),
        )
        .build();

        let operation_2 = OperationBuilder::new(
            ActivityNumber(20),
            unloading_point.clone(),
            Resources::MtnMech,
            Some(Work::from(1.0)),
        )
        .build();

        let operation_3 = OperationBuilder::new(
            ActivityNumber(30),
            unloading_point.clone(),
            Resources::MtnMech,
            Some(Work::from(1.0)),
        )
        .build();

        operations.insert(10, operation_1);
        operations.insert(20, operation_2);
        operations.insert(30, operation_3);

        let work_order_1 = WorkOrder::work_order_test();

        let mut work_orders = WorkOrders::default();

        work_orders.insert(work_order_1);
    }

    #[test]
    fn test_update_scheduler_state() {
        let work_order_number = WorkOrderNumber(2200002020);
        let vec_work_order_number = vec![work_order_number];
        let period_string: String = "2023-W47-48".to_string();

        let schedule_work_order = ScheduleChange::new(vec_work_order_number, period_string);

        let strategic_scheduling_internal =
            StrategicSchedulingRequest::Schedule(schedule_work_order);

        let periods: Vec<Period> = vec![Period::from_str("2023-W47-48").unwrap()];

        let optimized_work_orders = StrategicParameters::new(
            HashMap::new(),
            StrategicResources::default(),
            StrategicClustering::default(),
        );

        let mut scheduler_agent_algorithm = StrategicAlgorithm::new(
            PriorityQueues::new(),
            optimized_work_orders,
            ArcSwapSharedSolution::default().into(),
            HashSet::new(),
            periods.clone(),
        );

        let optimized_work_order = StrategicParameter::new(
            Some(periods[0].clone()),
            HashSet::new(),
            periods.first().unwrap().clone(),
            1000,
            HashMap::new(),
        );

        scheduler_agent_algorithm.set_strategic_parameter(work_order_number, optimized_work_order);

        scheduler_agent_algorithm
            .update_scheduling_state(strategic_scheduling_internal)
            .unwrap();

        assert_eq!(
            scheduler_agent_algorithm
                .strategic_parameters
                .strategic_work_order_parameters
                .get(&work_order_number)
                .as_ref()
                .unwrap()
                .locked_in_period
                .as_ref()
                .unwrap()
                .period_string(),
            "2023-W47-48"
        );
    }

    #[test]
    fn test_calculate_objective_value() {
        let work_order_number = WorkOrderNumber(2100023841);
        let mut strategic_parameters = StrategicParameters::new(
            HashMap::new(),
            StrategicResources::default(),
            StrategicClustering::default(),
        );

        let strategic_parameter = StrategicParameter::new(
            Some(Period::from_str("2023-W49-50").unwrap()),
            HashSet::new(),
            Period::from_str("2023-W47-48").unwrap(),
            1000,
            HashMap::new(),
        );

        strategic_parameters
            .insert_strategic_parameter(WorkOrderNumber(2100023841), strategic_parameter);

        let mut strategic_algorithm = StrategicAlgorithm::new(
            PriorityQueues::new(),
            strategic_parameters,
            ArcSwapSharedSolution::default().into(),
            HashSet::new(),
            vec![],
        );

        strategic_algorithm
            .strategic_solution
            .strategic_periods
            .insert(work_order_number, None);

        strategic_algorithm
            .schedule_forced_work_order(&ForcedWorkOrder::Locked(work_order_number))
            .unwrap();

        strategic_algorithm.calculate_objective_value();

        assert_eq!(
            strategic_algorithm
                .strategic_solution
                .objective_value
                .objective_value,
            2000
        );
    }
}
