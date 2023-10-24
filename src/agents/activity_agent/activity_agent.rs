use actix::prelude::*;
use chrono::{DateTime, Utc};

use crate::models::period::Period;

pub struct ActivityAgent {
    order: u32,
    activity: u32,
    sch_start: DateTime<Utc>,
    sch_date: DateTime<Utc>,
    period: Period,
    assigned: Vec<u32>,
}


impl Actor for ActivityAgent {
    type Context = Context<Self>;
}