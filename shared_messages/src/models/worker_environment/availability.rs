



use serde::Deserialize;

use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct Availability {
    start_date: toml::value::Datetime,
    end_date: toml::value::Datetime,
}
