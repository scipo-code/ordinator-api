use chrono::DateTime;
use chrono::TimeDelta;
use chrono::Utc;
use serde::Deserialize;

use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Availability {
    pub start_date: chrono::DateTime<Utc>,
    pub end_date: chrono::DateTime<Utc>,
}

impl Availability {
    pub fn duration(&self) -> TimeDelta {
        self.end_date - self.start_date
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TomlAvailability {
    start_date: toml::value::Datetime,
    end_date: toml::value::Datetime,
}

impl From<TomlAvailability> for Availability {
    fn from(value: TomlAvailability) -> Self {
        let start_date_time: DateTime<Utc> =
            DateTime::parse_from_rfc3339(&value.start_date.to_string())
                .unwrap()
                .to_utc();
        let end_date_time: DateTime<Utc> =
            DateTime::parse_from_rfc3339(&value.end_date.to_string())
                .unwrap()
                .to_utc();

        Self {
            start_date: start_date_time,
            end_date: end_date_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use toml::value::Datetime;

    use super::TomlAvailability;

    fn test_toml_availability_from() {
        let toml_availability = TomlAvailability {
            start_date: "2024-05-01T00:00:00Z".parse::<Datetime>().unwrap(),
            end_date: "2024-05-15T00:00:00Z".parse::<Datetime>().unwrap(),
        };
    }
}
