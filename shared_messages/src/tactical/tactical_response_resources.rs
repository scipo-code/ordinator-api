use serde::{Deserialize, Serialize};

use super::TacticalResources;

#[derive(Serialize)]
pub enum TacticalResponseResources {
    UpdatedResources(u32),
    Loading(TacticalResources),
    Capacity(TacticalResources),
    Percentage(TacticalResources),
}
