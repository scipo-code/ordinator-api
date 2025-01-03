use serde::Serialize;

use super::StrategicResources;

#[derive(Serialize)]
pub enum StrategicResponseResources {
    UpdatedResources(u32),
    LoadingAndCapacities(StrategicResources),
    Percentage(StrategicResources, StrategicResources),
}
