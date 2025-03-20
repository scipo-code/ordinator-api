use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TacticalStatusMessage
{
    General,
    Day(String),
}
