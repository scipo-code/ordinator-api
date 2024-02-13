use actix::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::Mutex;

use crate::agents::scheduler_agent::scheduler_algorithm::OptimizedWorkOrder;
use crate::agents::scheduler_agent::scheduler_algorithm::OptimizedWorkOrders;
use crate::agents::scheduler_agent::scheduler_algorithm::PriorityQueues;
use crate::agents::scheduler_agent::scheduler_algorithm::SchedulerAgentAlgorithm;
use crate::agents::scheduler_agent::StrategicAgent;
use crate::models::time_environment::period::Period;
use crate::models::SchedulingEnvironment;
use crate::models::WorkOrders;
use shared_messages::resources::Resources;

// We should not clone in the work orders here. They should reference the same thing to work
// properly.
pub fn build_scheduler_agent(
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
) -> Addr<StrategicAgent> {
    let cloned_work_orders = scheduling_environment.lock().unwrap().clone_work_orders();

    let optimized_work_orders: OptimizedWorkOrders =
        create_optimized_work_orders(&cloned_work_orders);

    // The periods should really not be where it is here. It should be in the SchedulingEnvironment.
    // What is the problem? The problem is that we would either need to clone the periods or
    // increment the reference count. We should increment the reference count, but this leads to the
    // problem of having to points of access to the scheduling environment for the SchedulerAgent.
    // This is not good, especially as the SchedulerAgentAlgorithm will be using a much more
    // efficient data structure in the future. This means that we should actually use the scheduling
    // environments periods and for now we can simply clone them and then later on we can
    // change the data structure to something more efficient. Yes the key insight here is that we
    // do not need the algorithm to have direct access to the SchedulingEnvironment only that the
    // SchedulingAgent will be able to update the SchedulerAgentAlgorithm when the
    // SchedulingEnvironment changes. This is a much better design.

    // let resource_capacity =

    fn initialize_manual_resources(
        scheduling_environment: &SchedulingEnvironment,
        start_value: f64,
    ) -> HashMap<Resources, HashMap<Period, f64>> {
        let mut resource_capacity: HashMap<Resources, HashMap<Period, f64>> = HashMap::new();
        for resource in scheduling_environment
            .get_worker_environment()
            .get_work_centers()
            .iter()
        {
            let mut periods = HashMap::new();
            for period in scheduling_environment.get_periods().iter() {
                periods.insert(period.clone(), start_value);
            }
            resource_capacity.insert(resource.clone(), periods);
        }
        resource_capacity
    }

    let locked_scheduling_environment = scheduling_environment.lock().unwrap();

    let scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
        0.0,
        initialize_manual_resources(&locked_scheduling_environment, 168.0),
        initialize_manual_resources(&locked_scheduling_environment, 0.0),
        PriorityQueues::new(),
        optimized_work_orders,
        locked_scheduling_environment.clone_periods(),
        true,
    );

    drop(locked_scheduling_environment);

    // dbg!(scheduler_agent_algorithm
    //     .get_manual_resources_capacities()
    //     .get(k));
    let scheduler_agent = StrategicAgent::new(
        String::from("Dan F"),
        scheduling_environment,
        scheduler_agent_algorithm,
        None,
        None,
    );
    scheduler_agent.start()
}

/// This function should be used by the scheduling environment. It should not be used by the
/// algorithm itself.
fn create_optimized_work_orders(work_orders: &WorkOrders) -> OptimizedWorkOrders {
    let mut optimized_work_orders: HashMap<u32, OptimizedWorkOrder> = HashMap::new();

    for (work_order_number, work_order) in &work_orders.inner {
        if work_order.get_unloading_point().present {
            let period = work_order.get_unloading_point().period.clone();
            optimized_work_orders.insert(
                *work_order_number,
                OptimizedWorkOrder::new(
                    period.clone(),
                    period,
                    HashSet::new(),
                    None,
                    work_order.get_order_weight(),
                    work_order.get_work_load().clone(),
                ),
            );
        }
    }
    OptimizedWorkOrders::new(optimized_work_orders)
}
