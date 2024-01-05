use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::ops::Add;

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Debug, Clone)]
pub struct Period {
    id: u32,
    period_string: String,
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
        let mut year = parts[0].parse::<i32>().map_err(|_| "Invalid year")?;

        let start_week = if parts[1].len() == 2 {
            parts[1][1..2]
                .parse::<u32>()
                .map_err(|_| "Invalid start week")?
        } else {
            parts[1][1..3]
                .parse::<u32>()
                .map_err(|_| "Invalid start week")?
        };
        let mut end_week = parts[2].parse::<u32>().map_err(|_| "Invalid end week")?;

        // Convert week number to a DateTime<Utc>

        let local = NaiveDate::from_isoywd_opt(year, start_week, chrono::Weekday::Mon).unwrap();
        let local_datetime = local.and_hms_opt(0, 0, 0).unwrap();
        let start_date = Utc.from_local_datetime(&local_datetime);

        let end_date = if end_week == 52
            || (end_week == 53 && NaiveDate::from_isoywd_opt(year, 53, Weekday::Mon).is_some())
        {
            // Last moment of week 52 or 53
            let local = NaiveDate::from_isoywd_opt(year, end_week, Weekday::Sun).unwrap();
            let local_datetime = local.and_hms_opt(23, 59, 59);
            Utc.from_local_datetime(&local_datetime.unwrap()).unwrap()
        } else {
            // Handle rollover to the next year
            if end_week > 52 {
                end_week = 1;
                year += 1;
            }
            // Moment just before the start of the next week
            let local = NaiveDate::from_isoywd_opt(year, end_week + 1, Weekday::Mon).unwrap();
            let local_datetime = local.and_hms_opt(0, 0, 0);
            Utc.from_local_datetime(&local_datetime.unwrap()).unwrap() - Duration::seconds(1)
        };

        // Create Period
        Ok(Period {
            id: 0, // Assuming default value for id, modify as needed
            period_string: period_string.to_string(),
            start_date: start_date.unwrap(),
            end_date,
        })
    }
}

impl Period {
    pub fn get_period_string(&self) -> String {
        self.period_string.clone()
    }

    pub fn get_end_date(&self) -> DateTime<Utc> {
        self.end_date
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn add_one_period(&self) -> Period {
        let start_date = self.end_date + chrono::Duration::seconds(1);
        let end_date = start_date + chrono::Duration::weeks(2) - chrono::Duration::seconds(1);
        Period::new(self.id, start_date, end_date)
    }
}

impl Add<Duration> for Period {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        let id = self.id + 1;
        let start_date = self.start_date + rhs;
        let end_date = self.end_date + rhs;
        Period::new(id, start_date, end_date)
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
    fn test_new_from_string_0() {
        let period = Period::new_from_string("2021-W01-02");

        assert_eq!(period.unwrap().period_string, "2021-W01-02".to_string());
    }

    #[test]
    fn test_new_from_string_1() {
        let period = Period::new_from_string("2023-W49-50");

        assert_eq!(
            period.clone().unwrap().period_string,
            "2023-W49-50".to_string()
        );
        assert_eq!(
            period.clone().unwrap().start_date,
            Utc.with_ymd_and_hms(2023, 12, 4, 0, 0, 0).unwrap()
        );
        assert_eq!(
            period.unwrap().end_date,
            Utc.with_ymd_and_hms(2023, 12, 17, 23, 59, 59).unwrap()
        );
    }

    #[test]
    fn test_new_from_string_2() {
        let period = Period::new_from_string("2023-W51-52");

        assert_eq!(
            period.clone().unwrap().period_string,
            "2023-W51-52".to_string()
        );
        assert_eq!(
            period.clone().unwrap().start_date,
            Utc.with_ymd_and_hms(2023, 12, 18, 0, 0, 0).unwrap()
        );
        assert_eq!(
            period.unwrap().end_date,
            Utc.with_ymd_and_hms(2023, 12, 31, 23, 59, 59).unwrap()
        );
    }

    #[test]
    fn test_new_from_string_3() {
        let period = Period::new_from_string("2023-W1-2");

        assert_eq!(
            period.clone().unwrap().period_string,
            "2023-W1-2".to_string()
        );
        assert_eq!(
            period.clone().unwrap().start_date,
            Utc.with_ymd_and_hms(2023, 1, 2, 0, 0, 0).unwrap()
        );
        assert_eq!(
            period.unwrap().end_date,
            Utc.with_ymd_and_hms(2023, 1, 15, 23, 59, 59).unwrap()
        );
    }

    #[test]
    fn test_from_isoywd_opt() {
        let year = 2023;
        let week = 52;

        let date = NaiveDate::from_isoywd_opt(year, week, chrono::Weekday::Mon);

        assert_eq!(date.unwrap().year(), year);
        assert_eq!(date.unwrap().iso_week().week(), week);

        assert_eq!(date.unwrap().day(), 25);
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
