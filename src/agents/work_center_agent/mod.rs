use actix::prelude::*;

use crate::models::period::Period;

pub struct WorkCenterAgent {
    work_trait: String,
    capacities: Vec<f32>,
    loading: Vec<f32>,
    excesses: Vec<f32>,
    periods: Vec<Period>
}

impl Actor for WorkCenterAgent {
    type Context = Context<Self>;
}