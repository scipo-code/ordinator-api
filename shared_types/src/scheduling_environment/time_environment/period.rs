use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc, Weekday};
use clap::Args;
use rust_xlsxwriter::IntoExcelData;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::ops::{Add, Sub};
use std::str::FromStr;

#[derive(Args, Serialize, Deserialize, Eq, PartialEq, Hash, Debug, Clone, PartialOrd, Ord)]
pub struct Period {
    id_internal: i32,
    period_string: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    pub year: i32,
    pub start_week: u32,
    pub end_week: u32,
}

impl Period {
    pub fn new(id: i32, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Period {
        let mut year = start_date.year();

        if is_last_three_days_of_year(start_date.naive_utc().date()) {
            year += 1;
        }

        let start_week: u32 = start_date.iso_week().week();
        let end_week: u32 = if end_date.iso_week().week() == 1 {
            52
        } else {
            end_date.iso_week().week()
        };

        let period_string = format!("{}-W{}-{}", year, start_week, end_week,);

        Period {
            id_internal: id,
            period_string,
            start_date,
            end_date,
            year,
            start_week,
            end_week,
        }
    }

    pub fn contains_date(&self, date: NaiveDate) -> bool {
        self.start_date.date_naive() <= date && date <= self.end_date.date_naive()
    }
}

impl Period {
    pub fn period_string(&self) -> String {
        self.period_string.clone()
    }

    pub fn start_date(&self) -> &DateTime<Utc> {
        &self.start_date
    }

    pub fn end_date(&self) -> &DateTime<Utc> {
        &self.end_date
    }

    pub fn id(&self) -> &i32 {
        &self.id_internal
    }
}

impl Add<Duration> for Period {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        let id = self.id_internal + 1;
        let start_date = self.start_date + rhs;
        let end_date = self.end_date + rhs;
        Period::new(id, start_date, end_date)
    }
}

impl Sub<Duration> for Period {
    type Output = Period;

    fn sub(self, rhs: Duration) -> Self::Output {
        let id = self.id_internal - 1;
        let start_date = self.start_date - rhs;
        let end_date = self.end_date - rhs;
        Period::new(id, start_date, end_date)
    }
}

impl Add<Duration> for &Period {
    type Output = Period;

    fn add(self, rhs: Duration) -> Self::Output {
        let id = self.id_internal - 1;
        let start_date = self.start_date + rhs;
        let end_date = self.end_date + rhs;
        Period::new(id, start_date, end_date)
    }
}

fn is_last_three_days_of_year(date: NaiveDate) -> bool {
    let year = date.year();
    let dec_29 = NaiveDate::from_ymd_opt(year, 12, 29);
    let dec_30 = NaiveDate::from_ymd_opt(year, 12, 30);
    let dec_31 = NaiveDate::from_ymd_opt(year, 12, 31);

    date == dec_29.unwrap() || date == dec_30.unwrap() || date == dec_31.unwrap()
}

impl Display for Period {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let print_string = self.period_string.clone();
        write!(f, "{}", print_string)
    }
}

impl Default for Period {
    fn default() -> Self {
        Period::new(
            0,
            Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2021, 1, 14, 23, 59, 59).unwrap(),
        )
    }
}

impl FromStr for Period {
    type Err = String;

    fn from_str(period_string: &str) -> Result<Self, Self::Err> {
        // Parse the string
        let parts: Vec<&str> = period_string.split('-').collect();
        if parts.len() != 3 {
            return Err("Invalid period string format".to_string());
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
            id_internal: 0, // Assuming default value for id, modify as needed
            period_string: period_string.to_string(),
            start_date: start_date.unwrap(),
            end_date,
            year,
            start_week,
            end_week,
        })
    }
}

impl IntoExcelData for Period {
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = self.period_string;
        worksheet.write_string(row, col, value)
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
        format: &rust_xlsxwriter::Format,
    ) -> Result<&'a mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = self.period_string;
        worksheet.write_string_with_format(row, col, value, format)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_period_add_duration_1() {
        // Setup initial period
        let initial_id = 1;
        let initial_start_date = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
        let initial_end_date = Utc.with_ymd_and_hms(2023, 1, 14, 23, 59, 59).unwrap();
        let period = Period::new(initial_id, initial_start_date, initial_end_date);

        // Define the duration to add (e.g., 1 month)
        let duration_to_add = Duration::weeks(2); // Adjust as per your Duration type

        // Perform the addition
        let new_period = period + duration_to_add;

        // Assert that the new period has the expected values
        assert_eq!(new_period.id_internal, initial_id + 1);
        assert_eq!(new_period.start_date, initial_start_date + duration_to_add);
        assert_eq!(new_period.end_date, initial_end_date + duration_to_add);
    }

    #[test]
    fn test_period_add_duration_2() {
        // Setup initial period
        let initial_id = 1;
        let initial_start_date = Utc.with_ymd_and_hms(2023, 12, 18, 0, 0, 0).unwrap();
        let initial_end_date = Utc.with_ymd_and_hms(2023, 12, 31, 23, 59, 59).unwrap();
        let period = Period::new(initial_id, initial_start_date, initial_end_date);

        // Define the duration to add (e.g., 1 month)
        let duration_to_add = Duration::weeks(2); // Adjust as per your Duration type

        // Perform the addition
        let new_period = period + duration_to_add;

        // Assert that the new period has the expected values
        assert_eq!(new_period.id_internal, initial_id + 1);
        assert_eq!(new_period.start_date, initial_start_date + duration_to_add);
        assert_eq!(new_period.end_date, initial_end_date + duration_to_add);
        assert_eq!(new_period.period_string, "2024-W1-2".to_string());
    }

    #[test]
    fn test_new_from_string_0() {
        let period = Period::from_str("2021-W01-02");

        assert_eq!(period.unwrap().period_string, "2021-W01-02".to_string());
    }

    #[test]
    fn test_new_from_string_1() {
        let period = Period::from_str("2023-W49-50");

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
        let period = Period::from_str("2023-W51-52");

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
        let period = Period::from_str("2023-W1-2");

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

    #[test]
    fn test_period_new() {
        let period = Period::from_str("2024-W51-52").unwrap();

        let new_period = Period::new(
            1,
            period.start_date().to_owned() + Duration::weeks(2),
            period.end_date().to_owned() + Duration::weeks(2),
        );

        assert_eq!(new_period.period_string, "2025-W1-2".to_string());
    }
    // fn period_strategy() -> impl Strategy<Value = Period> {
    //     (any::<i32>(), any::<String>(), any::<u32>(), any::<u32>()).prop_map(
    //         |(id_internal, period_string, start_week, end_week)| {

    //             Period {
    //                 id_internal,
    //                 period_string,
    //                 start_date,
    //                 end_date,
    //                 start_week,
    //                 end_week,
    //             }
    //         },
    //     )
    // }

    // proptest! {
    //     #[test]
    //     fn test_period_initialization(period in period_strategy()) {

    //         period.period_string
    //     }
    // }
}
