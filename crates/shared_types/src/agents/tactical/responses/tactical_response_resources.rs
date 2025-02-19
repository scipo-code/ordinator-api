use serde::Serialize;

use crate::agents::tactical::TacticalResources;

#[derive(Debug, Serialize)]
pub enum TacticalResourceResponse {
    UpdatedResources(u32),
    Loading(TacticalResources),
    Capacity(TacticalResources),
    Percentage((TacticalResources, TacticalResources)),
}
