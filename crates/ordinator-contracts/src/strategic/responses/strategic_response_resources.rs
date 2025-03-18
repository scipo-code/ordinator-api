use serde::Serialize;

// This is a low level type and it should not be exposed here
// TODO [ ] FIX [ ]
// Make a custom type for the StrategicResourcesApi
#[derive(Serialize)]
pub enum StrategicResponseResources {
    UpdatedResources(u32),
    LoadingAndCapacities(StrategicResourcesApi),
    Percentage(StrategicResourcesApi, StrategicResourcesApi),
}
#[derive(Serialize)]
struct StrategicResourcesApi {}
