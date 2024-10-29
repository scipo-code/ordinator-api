use anyhow::{bail, Result};
use shared_types::strategic::StrategicResources;
use tracing::{event, Level};

use super::StrategicAlgorithm;

pub trait StrategicAlgorithmAssertions {
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
}

impl StrategicAlgorithmAssertions for StrategicAlgorithm {}
