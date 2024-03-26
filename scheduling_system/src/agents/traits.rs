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
