use std::collections::HashMap;

use anyhow::{bail, ensure, Result};
use shared_types::{
    scheduling_environment::{
        work_order::operation::Work, worker_environment::resources::Resources,
    },
    strategic::StrategicResources,
    LoadOperation,
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
        for (resource, periods) in strategic_loading.inner.iter() {
            for (period, work) in periods.0.iter() {
                let capacity = strategic_capacity
                    .inner
                    .get(resource)
                    .unwrap()
                    .0
                    .get(period)
                    .unwrap();
                if work > capacity {
                    event!(
                        Level::ERROR,
                        resource = ?resource,
                        period = ?period,
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
        let mut aggregated_strategic_load = StrategicResources::new(HashMap::new());
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
                        aggregated_strategic_load.update_load(
                            &resource,
                            period,
                            load,
                            LoadOperation::Add,
                        );
                    }
                }
            }
        }

        for (resource, periods) in aggregated_strategic_load.inner {
            for (period, load) in periods.0 {
                match self
                    .resources_loadings()
                    .inner
                    .get(&resource)
                    .unwrap()
                    .0
                    .get(&period)
                {
                    // Some(resource_load) if (*resource_load - load).abs() < 0.005 => continue,
                    Some(resource_load) => {
                        if resource_load.0.round_dp(6) != load.0.round_dp(6) {
                            event!(Level::ERROR, resource = %resource, period = %period, aggregated_load = %load, resource_load = %resource_load);
                            bail!("aggregated load and loading are not the same");
                        }
                    }
                    None => {
                        bail!("aggregated load and resource loading are not identically shaped")
                    }
                }
            }
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
