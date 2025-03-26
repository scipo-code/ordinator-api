use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum OperationalResourceRequest {}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum OperationalSchedulingRequest
{
    OperationalIds,
    OperationalState(String),
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum OperationalStatusRequest
{
    General,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum OperationalTimeRequest {}
