use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Debug, Clone)]
pub struct Period {
    id: u32,
    pub period_string: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

impl Period {
    pub fn new(id: u32, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Period {
        let period_string = format!(
            "{}-W{}-{}",
            start_date.year(),
            start_date.iso_week().week(),
            if end_date.iso_week().week() == 1 {
                52
            } else {
                end_date.iso_week().week()
            }
        );
        Period {
            id,
            period_string,
            start_date,
            end_date,
        }
    }

    pub fn new_from_string(period_string: &str) -> Result<Period, &'static str> {
        // Parse the string
        let parts: Vec<&str> = period_string.split('-').collect();
        if parts.len() != 3 {
            return Err("Invalid period string format");
        }

        // Parse year and weeks
        let year = parts[0].parse::<i32>().map_err(|_| "Invalid year")?;

        let start_week = parts[1][1..3]
            .parse::<u32>()
            .map_err(|_| "Invalid start week")?;
        let mut end_week = parts[2].parse::<u32>().map_err(|_| "Invalid end week")?;

        // Convert week number to a DateTime<Utc>

        let local = NaiveDate::from_isoywd_opt(year, start_week, chrono::Weekday::Mon).unwrap();
        let local_datetime = local.and_hms_opt(0, 0, 0).unwrap();
        let start_date = Utc.from_local_datetime(&local_datetime);

        let end_date = if end_week == 52 {
            let local = NaiveDate::from_isoywd_opt(year, end_week, chrono::Weekday::Mon).unwrap();
            let local_datetime = local.and_hms_opt(23, 59, 59).unwrap();
            Utc.from_local_datetime(&local_datetime)
        } else {
            if end_week > 52 {
                end_week = 1
            }
            let local =
                NaiveDate::from_isoywd_opt(year, end_week + 1, chrono::Weekday::Mon).unwrap();
            let local_datetime = local.and_hms_opt(0, 0, 0).unwrap();
            Utc.from_local_datetime(&(local_datetime - chrono::Duration::seconds(1)))
        };

        // Create Period
        Ok(Period {
            id: 0, // Assuming default value for id, modify as needed
            period_string: period_string.to_string(),
            start_date: start_date.unwrap(),
            end_date: end_date.unwrap(),
        })
    }
}

impl Period {
    #[cfg(test)]
    pub fn get_string(&self) -> String {
        self.period_string.clone()
    }

    pub fn get_end_date(&self) -> DateTime<Utc> {
        self.end_date
    }
}

impl Display for Period {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Period: {}", self.period_string)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use chrono::Utc;

    #[test]
    fn test_new_from_string() {
        let period = Period::new_from_string("2021-W01-02");

        assert_eq!(period.unwrap().period_string, "2021-W01-02".to_string());
    }

    #[test]
    fn test_parse() {
        match "01".parse::<u32>() {
            Ok(n) => assert_eq!(n, 1),
            Err(_) => panic!(),
        }
    }

    impl Period {
        pub fn new_test() -> Self {
            Period::new(
                0,
                Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 14, 23, 59, 59).unwrap(),
            )
        }
    }
}
