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
    type TimeUnit;
    type ResourceUnit;
    type ScheduleUnit;

    fn get_objective_value(&self) -> f64;

    fn schedule(&mut self);

    fn unschedule(&mut self, message: u32);

    fn update_scheduling_state(
        &mut self,
        message: StrategicSchedulingMessage,
    ) -> Result<String, AgentError>;

    fn update_time_state(&mut self, message: StrategicTimeMessage) -> Result<String, AgentError>;

    fn update_resources_state(
        &mut self,
        message: StrategicResourceMessage,
    ) -> Result<String, AgentError>;
}
