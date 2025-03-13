use std::path::PathBuf;

use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct BaptisteToml {
    pub mid_functional_locations: PathBuf,
    pub mid_operations_status: PathBuf,
    pub mid_secondary_locations: PathBuf,
    pub mid_work_center: PathBuf,
    pub mid_work_operations: PathBuf,
    pub mid_work_orders: PathBuf,
    pub mid_work_orders_status: PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct TomlOperatingTime {
    operating_time: f64,
}
