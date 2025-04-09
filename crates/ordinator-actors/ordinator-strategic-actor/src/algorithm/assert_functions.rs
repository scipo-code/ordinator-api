use std::collections::HashMap;
use std::collections::hash_map::Entry;

use anyhow::Result;
use anyhow::bail;
use anyhow::ensure;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_orchestrator_actor_traits::SharedSolutionTrait;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use priority_queue::PriorityQueue;
use strum::IntoEnumIterator;
use tracing::Level;
use tracing::event;

use super::strategic_parameters::StrategicParameters;
use super::strategic_resources::StrategicResources;
use super::strategic_solution::StrategicSolution;

#[allow(dead_code)]
pub trait StrategicAssertions
{
    fn assert_that_capacity_is_respected(
        strategic_loading: &StrategicResources,
        strategic_capacity: &StrategicResources,
    ) -> Result<()>;
    fn assert_aggregated_load(&self) -> Result<()>;
    fn assert_excluded_periods(&self) -> Result<()>;
}

impl<Ss> StrategicAssertions
    for Algorithm<StrategicSolution, StrategicParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>
where
    Ss: SharedSolutionTrait,
{
    fn assert_that_capacity_is_respected(
        strategic_loading: &StrategicResources,
        strategic_capacity: &StrategicResources,
    ) -> Result<()>
    {
        for (period, operational_resources) in strategic_loading.0.iter() {
            for (operational_id, work) in operational_resources.iter() {
                let capacity = strategic_capacity
                    .0
                    .get(period)
                    .unwrap()
                    .get(operational_id)
                    .unwrap()
                    .total_hours;
                if work.total_hours > capacity {
                    event!(
                        Level::ERROR,
                        resource = ?period,
                        period = ?operational_id,
                        capacity = ?capacity,
                        loading = ? work,
                        "strategic_resources_exceeded"

                    );
                    bail!("Capacity exceeded")
                }
            }
        }
        Ok(())
    }

    fn assert_aggregated_load(&self) -> Result<()>
    {
        // let mut aggregated_strategic_load = StrategicResources::default();
        let mut aggregated_strategic_load = HashMap::new();
        for period in &self.parameters.strategic_periods {
            for (work_order_number, strategic_solution) in
                self.solution.strategic_scheduled_work_orders.iter()
            {
                let strategic_parameter = self
                    .parameters
                    .strategic_work_order_parameters
                    .get(work_order_number)
                    .unwrap();
                if strategic_solution.as_ref().unwrap() == &period.clone() {
                    let work_load = &strategic_parameter.work_load;
                    for resource in Resources::iter() {
                        let load: Work =
                            work_load.get(&resource).cloned().unwrap_or(Work::from(0.0));
                        // We just need to test that the total hours are correct. We do not have to
                        // focus on the individual resources. We can handle
                        // that in another assert function.

                        match aggregated_strategic_load.entry((period, resource)) {
                            Entry::Occupied(mut occupied_entry) => {
                                *occupied_entry.get_mut() += load;
                            }
                            Entry::Vacant(vacant_entry) => {
                                vacant_entry.insert(load);
                            }
                        }
                    }
                }
            }
        }

        // We should match that all the aggregated total_hours add up to
        // all the total hours for the actual loadings.
        for (resource, total_work) in aggregated_strategic_load {
            let loadings = self
                .solution
                .strategic_loadings
                .0
                .get(resource.0)
                .unwrap()
                .values()
                .fold(Work::from(0.0), |mut acc, or| {
                    acc += or.skill_hours.get(&resource.1).unwrap_or(&Work::from(0.0));
                    acc
                });

            ensure!(loadings == total_work);
        }
        Ok(())
    }

    fn assert_excluded_periods(&self) -> Result<()>
    {
        for (work_order_number, strategic_parameter) in
            &self.parameters.strategic_work_order_parameters
        {
            let excluded_periods = &strategic_parameter.excluded_periods;
            let locked_in_period = &strategic_parameter.locked_in_period;

            let scheduled_period = self
                .solution
                .strategic_scheduled_work_orders
                .get(work_order_number)
                .unwrap();

            if let Some(period) = scheduled_period {
                ensure!(
                    !excluded_periods.contains(period),
                    "\n{:#?}\nscheduled in:{:#?}\nlocked_in_period\n{:#?}\nwhich is part of the excluded periods:\n{:#?}",
                    work_order_number,
                    period,
                    locked_in_period,
                    excluded_periods,
                );
            }
            if let Some(locked_in_period) = locked_in_period {
                ensure!(!excluded_periods.contains(locked_in_period))
            }
        }
        Ok(())
    }
}
