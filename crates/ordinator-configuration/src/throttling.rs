use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug)]
pub struct Throttling
{
    strategic_throttling: u64,
    tactical_throttling: u64,
    supervisor_throttling: u64,
    operational_throttling: u64,
}
