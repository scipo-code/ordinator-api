use serde::Serialize;

use crate::agents::strategic::StrategicResources;

#[derive(Serialize)]
pub enum StrategicResponseResources {
    UpdatedResources(u32),
    LoadingAndCapacities(StrategicResources),
    Percentage(StrategicResources, StrategicResources),
}
