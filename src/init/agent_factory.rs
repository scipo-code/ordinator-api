use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use actix::prelude::*;

use crate::agents::scheduler_agent::SchedulerAgent;
use crate::models::scheduling_environment::SchedulingEnvironment;
use crate::agents::scheduler_agent::OptimizedWorkOrders;
use crate::agents::scheduler_agent::PriorityQueues;
use crate::agents::scheduler_agent::SchedulerAgentAlgorithm;
use crate::models::scheduling_environment::WorkOrders;
use crate::agents::scheduler_agent::OptimizedWorkOrder;


/// This is a very powerful function. I will explore it further.
//create_agent(type: AgentType, config: Config) -> Box<dyn Agent>

pub fn build_scheduler_agent(scheduling_environment: SchedulingEnvironment) -> Addr<SchedulerAgent> {

    let cloned_work_orders = scheduling_environment.work_orders.clone();
    
    let optimized_work_orders: OptimizedWorkOrders = create_optimized_work_orders(&cloned_work_orders);
    
    let scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
        HashMap::new(),
        HashMap::new(),
        cloned_work_orders,
        PriorityQueues::new(),
        optimized_work_orders,
        scheduling_environment.period.clone(),
    );

    let scheduler_agent = SchedulerAgent::new(
        String::from("Dan F"),
        scheduler_agent_algorithm,  
        None
    );
    
    scheduler_agent.start()
}


fn create_optimized_work_orders(work_orders: &WorkOrders) -> OptimizedWorkOrders {
    
    let mut optimized_work_orders: HashMap<u32, OptimizedWorkOrder> = HashMap::new();

    for (work_order_number, work_order) in &work_orders.inner {
        if work_order.unloading_point.present {
            let period = work_order.unloading_point.period.clone();
            optimized_work_orders.insert(*work_order_number, OptimizedWorkOrder::new(
                period.clone(),
                period,
                HashSet::new(),
            ));
        }
    }
    OptimizedWorkOrders::new(optimized_work_orders)
}