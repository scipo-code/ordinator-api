pub mod scheduler_message;
pub mod scheduler_algorithm;
pub mod display;

use std::collections::HashMap;
use actix::prelude::*; 
use actix::Message;
use priority_queue::PriorityQueue;
use std::hash::Hash;
use tracing::{info, event};
use tokio::time::{sleep, Duration};

use crate::agents::scheduler_agent::scheduler_message::{SetAgentAddrMessage, SchedulerMessages, InputMessage};
use crate::models::scheduling_environment::WorkOrders;
use crate::models::order_period::OrderPeriod;
use crate::models::period::Period;
use crate::api::websocket_agent::WebSocketAgent;
use crate::agents::scheduler_agent::scheduler_algorithm::QueueType;

pub struct SchedulerAgent {
    platform: String,
    scheduler_agent_algorithm: SchedulerAgentAlgorithm,
    ws_agent_addr: Option<Addr<WebSocketAgent>>,
}

pub struct SchedulerAgentAlgorithm {
    manual_resources_capacity : HashMap<(String, Period), f64>,
    manual_resources_loading: HashMap<(String, Period), f64>,
    backlog: WorkOrders,
    priority_queues: PriorityQueues<u32, u32>,
    scheduled_work_orders: HashMap<u32, OrderPeriod>,
    periods: Vec<Period>,
}


impl SchedulerAgent {
    pub fn set_ws_agent_addr(&mut self, ws_agent_addr: Addr<WebSocketAgent>) {
        self.ws_agent_addr = Some(ws_agent_addr);
    }

    // TODO: Here the other Agents Addr messages will also be handled.
}

impl Actor for SchedulerAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        self.populate_priority_queues();
        for _ in 0..self.scheduler_agent_algorithm.priority_queues.normal.len() {
            let work_order = self.scheduler_agent_algorithm.priority_queues.normal.pop();
            info!("SchedulerAgent is populated: {}", work_order.unwrap().0 );
        }
        ctx.notify(ScheduleIteration {})
    }

    fn stopped(&mut self, ctx: &mut Context<Self>) {
        println!("SchedulerAgent is stopped");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct ScheduleIteration {}


/// I think that the priotity queue should be a struct that is a member of the scheduler agent.
impl Handler<ScheduleIteration> for SchedulerAgent {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: ScheduleIteration, ctx: &mut Self::Context) -> Self::Result {
        event!(tracing::Level::INFO , "A round of scheduling has been triggered");
        self.schedule_work_orders_by_type(QueueType::Normal);
        // TODO self.ws_agent_addr.do_send(SchedulerMessages::WorkPlanner);

        let actor_addr = ctx.address().clone();

        let fut = async move {
            // Sleep for one second
            sleep(Duration::from_secs(1)).await;
            actor_addr.do_send(ScheduleIteration {});
        };
        Box::pin(actix::fut::wrap_future::<_, Self>(fut))
    }
}


impl Handler<SchedulerMessages> for SchedulerAgent {
    type Result = ();
    fn handle(&mut self, msg: SchedulerMessages, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SchedulerMessages::Input(msg) => {
                println!("SchedulerAgentReceived a FrontEnd message");
                let input_message: InputMessage = msg.into();
               
                self.update_scheduler_state(input_message);

                // TODO - modify state of scheduler agent
            }
            SchedulerMessages::WorkPlanner(msg) => {
               println!("SchedulerAgentReceived a WorkPlannerMessage message");
            },
            SchedulerMessages::ExecuteIteration => {
                // TODO - execute one optimization iteration of the scheduler agent
                self.execute_iteration(ctx);
            }
        }	
    }
}

impl Handler<SetAgentAddrMessage<WebSocketAgent>> for SchedulerAgent {
    type Result = ();

    fn handle(&mut self, msg: SetAgentAddrMessage<WebSocketAgent>, ctx: &mut Self::Context) -> Self::Result {
        self.set_ws_agent_addr(msg.addr);
    }
}

impl SchedulerAgent {
    pub fn execute_iteration(&mut self, ctx: &mut <SchedulerAgent as Actor>::Context) {

        println!("I am running a single iteration");  
        ctx.notify(SchedulerMessages::ExecuteIteration)
    }
}

impl SchedulerAgent {
    pub fn new(
        platform: String, 
        scheduler_agent_algorithm: SchedulerAgentAlgorithm,
        ws_agent_addr: Option<Addr<WebSocketAgent>>) 
            -> Self {
  
        Self {
            platform,
            scheduler_agent_algorithm,
            ws_agent_addr,
        }
    }
}

impl SchedulerAgentAlgorithm {
    pub fn new(
        manual_resources_capacity: HashMap<(String, Period), f64>, 
        manual_resources_loading: HashMap<(String, Period), f64>, 
        backlog: WorkOrders, 
        priority_queues: PriorityQueues<u32, u32>,
        scheduled_work_orders: HashMap<u32, OrderPeriod>, 
        periods: Vec<Period>,
    ) -> Self {
        SchedulerAgentAlgorithm {
            manual_resources_capacity,
            manual_resources_loading,
            backlog,
            priority_queues,
            scheduled_work_orders,
            periods            
        }
    }
}


impl SchedulerAgent {
    pub fn update_scheduler_state(&mut self, input_message: InputMessage) {
        self.scheduler_agent_algorithm.manual_resources_capacity = input_message.get_manual_resources();
    }


}



impl SchedulerAgent {

    fn populate_priority_queues(&mut self) -> () {
        for (key, work_order) in self.scheduler_agent_algorithm.backlog.inner.iter() {
            if work_order.unloading_point.present {
                self.scheduler_agent_algorithm.priority_queues.unloading.push(*key, work_order.order_weight);
            } else if work_order.revision.shutdown || work_order.vendor {
                self.scheduler_agent_algorithm.priority_queues.shutdown_vendor.push(*key, work_order.order_weight);
            } else {
                self.scheduler_agent_algorithm.priority_queues.normal.push(*key, work_order.order_weight);
            }
        }
    }


}


pub struct PriorityQueues<T, P> 
    where T: Hash + Eq,
          P: Ord
{ 
    unloading: PriorityQueue<T, P>,
    shutdown_vendor: PriorityQueue<T, P>,
    normal: PriorityQueue<T, P>,
}

impl PriorityQueues<u32, u32> {
    pub fn new() -> Self{
        Self {
            unloading: PriorityQueue::<u32, u32>::new(),
            shutdown_vendor: PriorityQueue::<u32, u32>::new(),
            normal: PriorityQueue::<u32, u32>::new(),
        }
    }
}
