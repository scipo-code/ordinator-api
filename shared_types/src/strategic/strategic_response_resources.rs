use serde::Serialize;

use super::StrategicResources;

#[derive(Serialize)]
pub enum StrategicResponseResources {
    UpdatedResources(u32),
    Loading(StrategicResources),
    Percentage(StrategicResources, StrategicResources),
}
