use serde::Deserialize;

use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Availability {
    start_date: toml::value::Datetime,
    end_date: toml::value::Datetime,
}
