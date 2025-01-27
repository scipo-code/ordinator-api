use std::collections::{hash_map::Entry, HashMap};

use anyhow::{bail, ensure, Result};
use shared_types::{
    scheduling_environment::{
        work_order::operation::Work, worker_environment::resources::Resources,
    },
    strategic::StrategicResources,
};
use strum::IntoEnumIterator;
use tracing::{event, Level};

use super::StrategicAlgorithm;

#[allow(dead_code)]
pub trait StrategicAssertions {
    fn assert_that_capacity_is_respected(
        strategic_loading: &StrategicResources,
        strategic_capacity: &StrategicResources,
    ) -> Result<()>;
    fn assert_aggregated_load(&self) -> Result<()>;
    fn assert_excluded_periods(&self) -> Result<()>;
}

impl StrategicAssertions for StrategicAlgorithm {
    fn assert_that_capacity_is_respected(
        strategic_loading: &StrategicResources,
        strategic_capacity: &StrategicResources,
    ) -> Result<()> {
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

    fn assert_aggregated_load(&self) -> Result<()> {
        // let mut aggregated_strategic_load = StrategicResources::default();
        let mut aggregated_strategic_load = HashMap::new();
        for period in self.periods() {
            for (work_order_number, strategic_solution) in
                self.strategic_solution.strategic_periods.iter()
            {
                let strategic_parameter = self
                    .strategic_parameters
                    .strategic_work_order_parameters
                    .get(work_order_number)
                    .unwrap();
                if strategic_solution.as_ref().unwrap() == &period.clone() {
                    let work_load = &strategic_parameter.work_load;
                    for resource in Resources::iter() {
                        let load: Work =
                            work_load.get(&resource).cloned().unwrap_or(Work::from(0.0));
                        // We just need to test that the total hours are correct. We do not have to focus
                        // on the individual resources. We can handle that in another assert function.

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
                .resources_loadings()
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

    fn assert_excluded_periods(&self) -> Result<()> {
        for (work_order_number, strategic_parameter) in
            &self.strategic_parameters.strategic_work_order_parameters
        {
            let excluded_periods = &strategic_parameter.excluded_periods;
            let locked_in_period = &strategic_parameter.locked_in_period;

            let scheduled_period = self
                .strategic_solution
                .strategic_periods
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
