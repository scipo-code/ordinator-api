use actix::prelude::*;

pub struct WorkPlannerAgent {
    id: i32,
    orders: Vec<u32>,
}

impl Actor for WorkPlannerAgent {
    type Context = Context<Self>;
}