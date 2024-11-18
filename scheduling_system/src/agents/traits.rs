use anyhow::Result;

#[allow(dead_code)]
pub trait LargeNeighborHoodSearch {
    type BetterSolution;
    type SchedulingRequest;
    type SchedulingResponse;
    type ResourceRequest;
    type ResourceResponse;
    type TimeRequest;
    type TimeResponse;

    type SchedulingUnit;

    fn calculate_objective_value(&mut self) -> Self::BetterSolution;

    fn schedule(&mut self) -> Result<()>;

    fn unschedule(&mut self, message: Self::SchedulingUnit) -> Result<()>;

    fn update_scheduling_state(
        &mut self,
        message: Self::SchedulingRequest,
    ) -> Result<Self::SchedulingResponse>;

    fn update_time_state(&mut self, message: Self::TimeRequest) -> Result<Self::TimeResponse>;

    fn update_resources_state(
        &mut self,
        message: Self::ResourceRequest,
    ) -> Result<Self::ResourceResponse>;
}
