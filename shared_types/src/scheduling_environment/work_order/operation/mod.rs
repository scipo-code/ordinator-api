pub mod operation_analytic;
pub mod operation_info;

use crate::scheduling_environment::work_order::operation::operation_info::OperationInfo;
use crate::scheduling_environment::{
    time_environment::day::Day, work_order::operation::operation_analytic::OperationAnalytic,
};

use crate::scheduling_environment::worker_environment::resources::Resources;
use chrono::{DateTime, Utc};
use rust_decimal::prelude::*;
use serde::de::{self, Deserialize, Error, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, SerializeTupleStruct};
use std::fmt::Display;
use std::num::ParseFloatError;
use std::str::FromStr;

use self::operation_info::NumberOfPeople;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Operation {
    pub activity: ActivityNumber,
    pub resource: Resources,
    pub operation_info: OperationInfo,
    pub operation_analytic: OperationAnalytic,
    pub operation_dates: OperationDates,
}

#[derive(Hash, Eq, PartialOrd, Ord, PartialEq, Debug, Clone)]
pub struct Work(Decimal);

impl FromStr for Work {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<f64>()?;

        Ok(Work::from(value))
    }
}
impl Work {
    pub fn from(work: f64) -> Self {
        let u32_f32 = Decimal::from_f64(work).unwrap();
        Work(u32_f32)
    }

    pub(crate) fn work(&self) -> Decimal {
        self.0
    }

    pub fn in_seconds(&self) -> u64 {
        (self.0 * Decimal::from_u64(3600).unwrap())
            .to_u64()
            .unwrap()
    }

    pub fn to_f64(&self) -> f64 {
        self.0.to_f64().unwrap()
    }

    pub fn cal_duration(&self, number: u64) -> Work {
        Work(self.0 / Decimal::from_u64(number).unwrap())
    }
}

impl std::ops::Add for Work {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let value: Decimal = self.work() + rhs.work();
        Self(value)
    }
}
impl std::ops::Add for &Work {
    type Output = Work;

    fn add(self, rhs: Self) -> Self::Output {
        let value: Decimal = self.work() + rhs.work();
        Work(value)
    }
}

impl std::ops::AddAssign for Work {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl std::ops::AddAssign for &mut Work {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl std::ops::Sub for Work {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let value = self.0 - rhs.0;
        Work(value)
    }
}

impl std::ops::Sub for &Work {
    type Output = Work;

    fn sub(self, rhs: Self) -> Self::Output {
        let value = self.0 - rhs.0;
        Work(value)
    }
}

impl std::ops::SubAssign for Work {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}

impl std::ops::Sub<&Work> for &mut Work {
    type Output = Work;

    fn sub(self, rhs: &Work) -> Work {
        Work(self.0 - rhs.0)
    }
}

impl std::ops::SubAssign<&Work> for Work {
    fn sub_assign(&mut self, rhs: &Work) {
        self.0 -= rhs.0
    }
}

impl std::ops::Add<&Work> for &mut Work {
    type Output = Work;

    fn add(self, rhs: &Work) -> Work {
        Work(self.0 + rhs.0)
    }
}

impl std::ops::AddAssign<&Work> for Work {
    fn add_assign(&mut self, rhs: &Work) {
        self.0 += rhs.0
    }
}

impl Serialize for Work {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Work", 2)?;
        s.serialize_field("type", "Decimal")?;
        s.serialize_field("value", &self.0.to_f64().unwrap())?;
        s.end()
    }
}

// pub struct Work(Decimal);

impl<'de> Deserialize<'de> for Work {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct WorkVisitor;

        impl<'de> Visitor<'de> for WorkVisitor {
            type Value = Work;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("An object with a type: Decimal, and value f64 that serializes into a Decimal type in Rust")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut value = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "value" => {
                            if value.is_some() {
                                return Err(de::Error::duplicate_field("value"));
                            }

                            let value_float: f64 = map.next_value()?;

                            value = Decimal::from_f64_retain(value_float);
                        }
                        "type" => {
                            if value.is_some() {
                                return Err(de::Error::duplicate_field("type"));
                            }
                            let value_str: String = map.next_value()?;
                            assert_eq!(value_str, "Decimal".to_string());
                        }
                        _ => return Err(de::Error::unknown_field(&key, &["type", "value"])),
                    }
                }
                let fixed_val = value.ok_or_else(|| de::Error::missing_field("value"))?;
                Ok(Work(fixed_val))
            }
        }
        deserializer.deserialize_struct("Work", &["type", "value"], WorkVisitor)
    }
}

impl Display for Work {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Operation {
    pub fn new(
        activity: ActivityNumber,
        resource: Resources,
        operation_info: OperationInfo,
        operation_analytic: OperationAnalytic,
        operation_dates: OperationDates,
    ) -> Self {
        Operation {
            activity,
            resource,
            operation_info,
            operation_analytic,
            operation_dates,
        }
    }

    pub fn work_remaining(&self) -> &Work {
        self.operation_info.work_remaining()
    }

    pub fn resource(&self) -> &Resources {
        &self.resource
    }

    pub fn number(&self) -> NumberOfPeople {
        self.operation_info.number()
    }

    pub fn duration(&self) -> &Work {
        &self.operation_analytic.duration
    }

    pub fn operating_time(&self) -> &Work {
        self.operation_info.operating_time()
    }
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub struct ActivityNumber(pub u64);

impl From<u64> for ActivityNumber {
    fn from(value: u64) -> Self {
        ActivityNumber(value)
    }
}

impl Serialize for ActivityNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for ActivityNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let activity_number_string = String::deserialize(deserializer).unwrap();
        let activity_number_primitive = activity_number_string.parse::<u64>().unwrap();

        Ok(ActivityNumber(activity_number_primitive))
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct OperationDates {
    pub possible_start: Day,
    pub target_finish: Day,
    pub earliest_start_datetime: DateTime<Utc>,
    pub earliest_finish_datetime: DateTime<Utc>,
}

impl OperationDates {
    pub fn new(
        possible_start: Day,
        target_finish: Day,
        earliest_start_datetime: DateTime<Utc>,
        earliest_finish_datetime: DateTime<Utc>,
    ) -> Self {
        assert!(possible_start <= target_finish);
        OperationDates {
            possible_start,
            target_finish,
            earliest_start_datetime,
            earliest_finish_datetime,
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "    Activity: {:>8?}    |{:>11}|{:>14}|{:>8}|{:>6}|",
            self.activity,
            self.resource.to_string(),
            self.operation_info.work_remaining().work(),
            self.operation_analytic.duration.work(),
            self.operation_info.number(),
        )
    }
}

impl Operation {
    pub fn builder(
        activity: ActivityNumber,
        resource: Resources,
        work_remaining: Work,
    ) -> OperationBuilder {
        let operation_info = OperationInfo::new(
            1,
            work_remaining,
            Work::from(0.0),
            Work::from(0.0),
            Work::from(6.0),
        );

        let operation_analytic = OperationAnalytic::new(Work::from(1.0), Work::from(6.0));

        let operation_dates = OperationDates::new(
            Day::new(0, Utc::now()),
            Day::new(0, Utc::now()),
            Utc::now(),
            Utc::now(),
        );

        OperationBuilder {
            activity,
            resource,
            operation_info,
            operation_analytic,
            operation_dates,
        }
    }
}

pub struct OperationBuilder {
    activity: ActivityNumber,
    resource: Resources,
    operation_info: OperationInfo,
    operation_analytic: OperationAnalytic,
    operation_dates: OperationDates,
}

#[allow(dead_code)]
impl OperationBuilder {
    fn with_operation_info(
        mut self,
        number: NumberOfPeople,
        work_remaining: Work,
        work_performed: Work,
        work_adjusted: Work,
        operating_time: Work,
    ) -> Self {
        let operation_info = OperationInfo::new(
            number,
            work_remaining,
            work_performed,
            work_adjusted,
            operating_time,
        );

        self.operation_info = operation_info;
        self
    }

    fn with_operation_dates(mut self) -> Self {
        let operation_dates = OperationDates::new(
            Day::new(0, Utc::now()),
            Day::new(0, Utc::now()),
            Utc::now(),
            Utc::now(),
        );

        self.operation_dates = operation_dates;
        self
    }

    pub fn build(self) -> Operation {
        Operation {
            activity: self.activity,
            resource: self.resource,
            operation_info: self.operation_info,
            operation_analytic: self.operation_analytic,
            operation_dates: self.operation_dates,
        }
    }
}
