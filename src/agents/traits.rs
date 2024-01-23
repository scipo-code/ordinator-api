use crate::models::SchedulingEnvironment;
use std::sync::{Arc, Mutex};
/// What should an algorithm be able to do? This is the trait that all scheduling algorithms should
/// implement. It is a trait so that we can have multiple algorithms in the same system.
pub trait Algorithm {
    fn get_scheduling_environment(&self) -> Arc<Mutex<SchedulingEnvironment>>;

    fn update_state(&mut self, message: impl actix::Message);

    fn schedule(&mut self);

    fn unschedule(&mut self);
}
