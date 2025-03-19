use chrono::DateTime;
use chrono::Days;
use chrono::NaiveTime;
use chrono::TimeDelta;
use chrono::Utc;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de;

use self::day::Day;
use self::period::Period;

pub mod day;
pub mod period;

// WARN: Make the fields private. It does not make sense to change these
// individually. FIX
// All Periods here refer to the same thing. You should use references
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct TimeEnvironment {
    pub strategic_periods: Vec<Period>,
    pub tactical_periods: Vec<Period>,
    pub tactical_days: Vec<Day>,
    pub supervisor_periods: Vec<Period>,
}

#[derive(Deserialize)]
pub struct MaterialToPeriod {
    pub nmat: usize,
    pub smat: usize,
    pub cmat: usize,
    pub pmat: usize,
    pub wmat: usize,
}

impl TimeEnvironment {
    pub fn new(
        strategic_periods: Vec<Period>,
        tactical_periods: Vec<Period>,
        tactical_days: Vec<Day>,
        supervisor_periods: Vec<Period>,
    ) -> Self {
        TimeEnvironment {
            strategic_periods,
            tactical_periods,
            tactical_days,
            supervisor_periods,
        }
    }

    pub fn builder() -> TimeEnvironmentBuilder {
        TimeEnvironmentBuilder::default()
    }
}

#[derive(Default)]
pub struct TimeEnvironmentBuilder {
    pub strategic_periods: Option<Vec<Period>>,
    pub supervisor_periods: Option<Vec<Period>>,
    pub tactical_days: Option<Vec<Day>>,
    pub tactical_periods: Option<Vec<Period>>,
}

impl TimeEnvironmentBuilder {
    pub fn build(self) -> TimeEnvironment {
        TimeEnvironment {
            strategic_periods: self.strategic_periods.unwrap_or_default(),
            tactical_periods: self.tactical_periods.unwrap_or_default(),
            tactical_days: self.tactical_days.unwrap_or_default(),
            supervisor_periods: self.supervisor_periods.unwrap_or_default(),
        }
    }

    pub fn strategic_periods(&mut self, strategic_periods: Vec<Period>) -> &mut Self {
        self.strategic_periods = Some(strategic_periods);
        self
    }

    pub fn tactical_periods(&mut self, tactical_periods: Vec<Period>) -> &mut Self {
        self.tactical_periods = Some(tactical_periods);
        self
    }

    pub fn tactical_days(&mut self, first_day: &str, number_of_tactical_days: u64) -> &mut Self {
        let mut first_day: DateTime<Utc> =
            first_day.parse().expect("You did not provide a valid date");
        let mut tactical_days = |number_of_tactical_days: u64| -> Vec<Day> {
            let mut days: Vec<Day> = Vec::new();
            for day_index in 0..number_of_tactical_days {
                days.push(Day::new(day_index as usize, first_day.to_owned()));
                first_day = first_day.checked_add_days(Days::new(1)).unwrap();
            }
            days
        };
        self.tactical_days = Some(tactical_days(number_of_tactical_days));
        self
    }

    pub fn supervisor_periods(&mut self, supervisor_periods: Vec<Period>) -> &mut Self {
        self.supervisor_periods = Some(supervisor_periods);
        self
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct TimeInterval {
    #[serde(deserialize_with = "deserialize_time_interval")]
    pub start: NaiveTime,
    #[serde(deserialize_with = "deserialize_time_interval")]
    pub end: NaiveTime,
}

impl TimeInterval {
    pub fn new(start: NaiveTime, end: NaiveTime) -> Self {
        assert_ne!(start, end);
        Self { start, end }
    }

    pub fn from_date_times(
        start_date_time: DateTime<Utc>,
        finish_date_time: DateTime<Utc>,
    ) -> Self {
        Self {
            start: start_date_time.time(),
            end: finish_date_time.time(),
        }
    }

    pub fn contains(&self, date_time: &DateTime<Utc>) -> bool {
        let time = date_time.time();

        if self.start > self.end {
            (self.start <= time && time <= NaiveTime::from_hms_opt(23, 59, 59).unwrap())
                || (NaiveTime::from_hms_opt(0, 0, 0).unwrap() <= time && time < self.end)
        } else {
            self.start <= time && time < self.end
        }
    }

    pub fn duration(&self) -> TimeDelta {
        if self.end < self.start {
            TimeDelta::new(86400, 0).unwrap() - (self.end - self.start).abs()
        } else {
            (self.end - self.start).abs()
        }
    }

    pub fn invert(&self) -> TimeInterval {
        let inverted_start = self.end;
        let inverted_end = self.start;

        let inverted_time_interval = TimeInterval {
            start: inverted_start,
            end: inverted_end,
        };
        assert_eq!(self.duration(), inverted_time_interval.duration());
        inverted_time_interval
    }
}
fn deserialize_time_interval<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
where
    D: Deserializer<'de>,
{
    let time_str: String = Deserialize::deserialize(deserializer)?;
    NaiveTime::parse_from_str(&time_str, "%H:%M:%S").map_err(de::Error::custom)
}
