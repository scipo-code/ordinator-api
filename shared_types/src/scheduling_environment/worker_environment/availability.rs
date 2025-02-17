use chrono::DateTime;
use chrono::TimeDelta;
use chrono::Utc;
use serde::de;
use serde::Deserialize;

use serde::Deserializer;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Availability {
    #[serde(deserialize_with = "chrono_datetime_deserialize")]
    pub start_date: chrono::DateTime<Utc>,
    #[serde(deserialize_with = "chrono_datetime_deserialize")]
    pub finish_date: chrono::DateTime<Utc>,
}

impl Availability {
    pub fn new(start_date: chrono::DateTime<Utc>, end_date: chrono::DateTime<Utc>) -> Self {
        Self {
            start_date,
            finish_date: end_date,
        }
    }

    pub fn duration(&self) -> TimeDelta {
        self.finish_date - self.start_date
    }
}

fn chrono_datetime_deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let datetime_str: String = Deserialize::deserialize(deserializer)?;

    let datetime = DateTime::parse_from_rfc3339(&datetime_str).map_err(de::Error::custom)?;
    Ok(datetime.to_utc())
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
            finish_date: end_date_time,
        }
    }
}
