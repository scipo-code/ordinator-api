pub mod baptiste_csv_reader;
pub mod baptiste_csv_reader_merges;

use chrono::{Datelike, Days, Duration, Timelike, Utc};
use shared_types::scheduling_environment::{
    time_environment::{day::Day, period::Period, TimeEnvironment},
    SchedulingEnvironment,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulingEnvironmentFactoryError {
    #[error("error while creating SchedulingEnvironment from excel file")]
    ExcelError(#[from] calamine::Error),
}

pub trait SchedulingEnvironmentFactory<DataSource> {
    fn create_scheduling_environment(
        file_path: DataSource,
        time_input: TimeInput,
    ) -> Result<SchedulingEnvironment, SchedulingEnvironmentFactoryError>;
}

pub struct TimeInput {
    pub number_of_strategic_periods: u64,
    pub number_of_tactical_periods: u64,
    pub number_of_days: u64,
    pub number_of_supervisor_periods: u64,
}

pub fn create_time_environment(time_input: &TimeInput) -> TimeEnvironment {
    let strategic_periods: Vec<Period> = create_periods(time_input.number_of_strategic_periods);

    let tactical_periods: &Vec<Period> =
        &strategic_periods.clone()[0..time_input.number_of_tactical_periods as usize].to_vec();

    let first_period = strategic_periods.first().unwrap().clone();

    let tactical_days = |number_of_days: u64| -> Vec<Day> {
        let mut days: Vec<Day> = Vec::new();
        let mut date = first_period.start_date().to_owned();
        for day_index in 0..number_of_days {
            days.push(Day::new(day_index as usize, date.to_owned()));
            date = date.checked_add_days(Days::new(1)).unwrap();
        }
        days
    };
    let supervisor_periods =
        strategic_periods.clone()[0..time_input.number_of_supervisor_periods as usize].to_vec();

    TimeEnvironment::new(
        strategic_periods,
        tactical_periods.to_vec(),
        tactical_days(time_input.number_of_days),
        supervisor_periods,
    )
}

fn create_periods(number_of_periods: u64) -> Vec<Period> {
    let mut periods: Vec<Period> = Vec::<Period>::new();
    let mut start_date = Utc::now();

    // Get the ISO week number
    let week_number = start_date.iso_week().week();
    // Determine target week number: If current is even, target is the previous odd
    let target_week = if week_number % 2 == 0 {
        week_number - 1
    } else {
        week_number
    };

    // Compute the offset in days to reach Monday of the target week
    let days_to_offset = (start_date.weekday().num_days_from_monday() as i64)
        + (7 * (week_number - target_week) as i64);

    start_date -= Duration::days(days_to_offset);

    start_date = start_date
        .with_hour(0)
        .and_then(|d| d.with_minute(0))
        .and_then(|d| d.with_second(0))
        .and_then(|d| d.with_nanosecond(0))
        .unwrap();

    let mut end_date = start_date + Duration::weeks(2);

    end_date -= Duration::days(1);

    end_date = end_date
        .with_hour(23)
        .and_then(|d| d.with_minute(59))
        .and_then(|d| d.with_second(59))
        .and_then(|d| d.with_nanosecond(0))
        .unwrap();

    let mut period = Period::new(0, start_date, end_date);
    periods.push(period.clone());
    for _ in 1..number_of_periods {
        period = period + Duration::weeks(2);
        periods.push(period.clone());
    }
    periods
}
