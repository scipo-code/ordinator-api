use std::collections::HashMap;
use actix::prelude::*; 

use crate::{agents::work_center_agent::WorkCenterAgent, messages::scheduler_message::ManualResource};
use crate::models::work_order::WorkOrder;
use crate::models::order_period::OrderPeriod;
use crate::models::period::Period;

use std::collections::BinaryHeap;

use std::hash::Hash;

use crate::messages::scheduler_message::SchedulerMessages;
use crate::models::scheduling_environment::SchedulingEnvironment;
use crate::data_processing::sources::excel::load_data_file;

use priority_queue::PriorityQueue;
use tokio::time::{sleep, Duration};

pub struct SchedulerAgent {
    platform: String,
    manual_resources : HashMap<(String, Period), f64>,
    // workcenter_agents: HashMap<String, WorkCenterAgent>,
    backlog: Vec<WorkOrder>,
    scheduled_work_orders: HashMap<i32, OrderPeriod>,
    periods: Vec<Period>,
}

impl SchedulerAgent {
    pub fn schedule(&mut self, ctx: &mut Context<Self>) {

        let mut priority_queues = PriorityQueues::<u32, u32> {
            unloading: PriorityQueue::new(),
            shutdown_vendor: PriorityQueue::new(),
            normal: PriorityQueue::new(),
        };

        populate_priority_queues(&self.backlog, &mut priority_queues);

        loop {
            println!("Hello I am scheduling");
            sleep(Duration::from_secs(1)).await;
        }
    }
}

struct PriorityQueues<T, P> 
    where T: Hash + Eq,
          P: Ord
{ 
    unloading: PriorityQueue<T, P>,
    shutdown_vendor: PriorityQueue<T, P>,
    normal: PriorityQueue<T, P>,
}

impl Actor for SchedulerAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        println!("SchedulerAgent is alive");
        self.schedule(ctx);
    }

    fn stopped(&mut self, ctx: &mut Context<Self>) {
        println!("SchedulerAgent is stopped");
    }
}


fn populate_priority_queues(backlog: &Vec<WorkOrder>, priority_queues: &mut PriorityQueues<u32, u32>) {
    for work_order in backlog {
        if work_order.unloading_point.present {
            priority_queues.unloading.push(work_order.order_number, work_order.order_weight);
        } else if work_order.revision.shutdown || work_order.vendor {
            priority_queues.shutdown_vendor.push(work_order.order_number, work_order.order_weight);
        } else {
            priority_queues.normal.push(work_order.order_number, work_order.order_weight);
        }
    }
}

impl SchedulerAgent {
    pub fn new(
        platform: String, 
        manual_resources: HashMap<(String, Period), f64>, 
        backlog: Vec<WorkOrder>, 
        scheduled_work_orders: HashMap<i32, OrderPeriod>, 
        periods: Vec<Period> ) 
            -> Self {
  
        Self {
            platform,
            manual_resources,
            backlog,
            scheduled_work_orders,
            periods,
        }
    }
}