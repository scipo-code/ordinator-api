pub mod operational_request_resource;
pub mod operational_request_scheduling;
pub mod operational_request_status;
pub mod operational_request_time;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum OperationalResourceRequest {}
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum OperationalSchedulingRequest
{
    OperationalIds,
    OperationalState(String),
}
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum OperationalStatusRequest
{
    General,
}
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum OperationalTimeRequest {}
