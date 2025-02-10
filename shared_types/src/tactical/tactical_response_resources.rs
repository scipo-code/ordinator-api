use serde::Serialize;

use super::TacticalResources;

#[derive(Debug, Serialize)]
pub enum TacticalResourceResponse {
    UpdatedResources(u32),
    Loading(TacticalResources),
    Capacity(TacticalResources),
    Percentage((TacticalResources, TacticalResources)),
}
