use shared_messages::{
    agent_error::AgentError,
    strategic::{
        strategic_periods_message::StrategicTimeMessage,
        strategic_resources_message::StrategicResourceMessage,
        strategic_scheduling_message::StrategicSchedulingMessage,
    },
};

/// What should an algorithm be able to do? This is the trait that all scheduling algorithms should
/// implement. It is a trait so that we can have multiple algorithms in the same system.
pub trait LargeNeighborHoodSearch {
    type SchedulingMessage;
    type ResourceMessage;
    type TimeMessage;

    type Error;

    fn objective_value(&self) -> f64;

    fn schedule(&mut self);

    fn unschedule(&mut self, message: u32);

    fn update_scheduling_state(
        &mut self,
        message: Self::SchedulingMessage,
    ) -> Result<String, Self::Error>;

    fn update_time_state(&mut self, message: Self::TimeMessage) -> Result<String, Self::Error>;

    fn update_resources_state(
        &mut self,
        message: Self::ResourceMessage,
    ) -> Result<String, Self::Error>;
}
