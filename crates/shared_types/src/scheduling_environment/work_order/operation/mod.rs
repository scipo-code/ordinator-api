pub mod operation_analytic;
pub mod operation_info;

use anyhow::Context;
use chrono::{DateTime, Utc};
use colored::Colorize;
use operation_info::OperationInfoBuilder;
use rust_decimal::prelude::*;
use rust_xlsxwriter::IntoExcelData;
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fmt::Display;
use std::num::ParseFloatError;
use std::str::FromStr;

use crate::scheduling_environment::time_environment::day::Day;
use crate::scheduling_environment::worker_environment::resources::Resources;

use self::operation_analytic::OperationAnalytic;
use self::operation_info::OperationInfo;

use super::work_order_dates::unloading_point::UnloadingPoint;
use super::ActivityRelation;

pub type ActivityNumber = u64;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Operations(pub HashMap<ActivityNumber, Operation>);

impl Operations {
    pub fn relations(&self) -> Vec<ActivityRelation> {
        todo!()
    }

    pub(crate) fn builder() -> OperationsBuilder {
        OperationsBuilder(Operations::default())
    }
}

// QUESTION [ ]
// Should it be possible for there to be one `Operations`?
// No it should not be possible
pub struct OperationsBuilder(Operations);

impl Operation {
    pub fn builder(operations_number: ActivityNumber, resource: Resources) -> OperationBuilder {
        todo!()
    }
}

impl OperationBuilder {
    pub fn build(self) -> Operation {
        Operation {
            activity: self.activity,
            resource: self.resource,
            unloading_point: self.unloading_point.unwrap_or_default(),
            operation_info: self.operation_info.unwrap_or_default(),
            operation_analytic: self.operation_analytic.unwrap_or_default(),
            operation_dates: self.operation_dates.unwrap_or_default(),
        }
    }
}

impl OperationsBuilder {
    pub fn build(self) -> Operations {
        Operations(self.0 .0)
    }

    // This should insert values into the `Operations` if there are no one there.
    pub fn operations_builder<F>(
        &mut self,
        operation_number: u64,
        resource: Resources,
        f: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut OperationBuilder) -> &mut OperationBuilder,
    {
        let mut operations_builder = Operation::builder(operation_number, resource);

        f(&mut operations_builder);

        self.0
             .0
            .insert(operations_builder.activity, operations_builder.build());

        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Operation {
    pub activity: ActivityNumber,
    pub resource: Resources,
    pub unloading_point: UnloadingPoint,
    pub operation_info: OperationInfo,
    pub operation_analytic: OperationAnalytic,
    pub operation_dates: OperationDates,
}

pub struct OperationBuilder {
    activity: ActivityNumber,
    resource: Resources,
    unloading_point: Option<UnloadingPoint>,
    operation_info: Option<OperationInfo>,
    operation_analytic: Option<OperationAnalytic>,
    operation_dates: Option<OperationDates>,
}

impl OperationBuilder {
    pub fn activity(&mut self, activity: u64) -> &mut Self {
        self.activity = activity;
        self
    }
    pub fn resource(&mut self, resource: Resources) -> &mut Self {
        self.resource = resource;
        self
    }
    pub fn unloading_point(&mut self, unloading_point: UnloadingPoint) -> &mut Self {
        self.unloading_point = Some(unloading_point);
        self
    }
    pub fn operation_info<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut OperationInfoBuilder) -> &mut OperationInfoBuilder,
    {
        let mut operation_info_builder = OperationInfo::builder();

        f(&mut operation_info_builder);

        self.operation_info = Some(operation_info_builder.build());
        self
    }
    pub fn operation_analytic(&mut self, operation_analytic: OperationAnalytic) -> &mut Self {
        self.operation_analytic = Some(operation_analytic);
        self
    }
    pub fn operation_dates(&mut self, operation_dates: OperationDates) -> &mut Self {
        self.operation_dates = Some(operation_dates);
        self
    }
}

#[derive(Copy, Default, Hash, Eq, PartialOrd, Ord, PartialEq, Clone)]
pub struct Work(pub Decimal);

impl std::fmt::Debug for Work {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{}", format!("Work({})", self.0).bright_yellow())
        } else {
            f.debug_struct("Work").field("", &self.0).finish()
        }
    }
}

impl FromStr for Work {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<f64>()?;

        Ok(Work::from(value))
    }
}
impl Work {
    pub fn from(work: f64) -> Self {
        let u32_f32 = Decimal::from_f64(work)
            .with_context(|| {
                format!(
                    "\nTried to create a {} from f64\n{}",
                    std::any::type_name::<Work>(),
                    work
                )
            })
            .unwrap();
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

    pub fn divide_work(&self, work: Work) -> Work {
        let value = self.0 / work.0;
        Work(value)
    }

    pub fn equal(&self, aggregate_strategic_resource: Work) -> bool {
        self.0.round_dp(5) == aggregate_strategic_resource.0.round_dp(5)
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
        s.serialize_field("work_type", "Decimal")?;
        s.serialize_field("work_value", &self.0.to_f64().unwrap())?;
        s.end()
    }
}

// pub struct Work(Decimal);

impl<'de> Deserialize<'de> for Work {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct WorkVisitorMap;

        impl<'de> Visitor<'de> for WorkVisitorMap {
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
                        "work_value" => {
                            if value.is_some() {
                                return Err(de::Error::duplicate_field("work_value"));
                            }

                            let value_float: f64 = map.next_value()?;

                            value = Decimal::from_f64_retain(value_float);
                        }
                        "work_type" => {
                            if value.is_some() {
                                return Err(de::Error::duplicate_field("work_type"));
                            }
                            let value_str: String = map.next_value()?;
                            assert_eq!(value_str, "Decimal".to_string());
                        }
                        _ => {
                            return Err(de::Error::unknown_field(
                                &key,
                                &["work_type", "work_value"],
                            ))
                        }
                    }
                }

                let fixed_val = value.ok_or_else(|| de::Error::missing_field("work_value"))?;
                Ok(Work(fixed_val))
            }
        }

        deserializer.deserialize_struct("Work", &["work_type", "work_value"], WorkVisitorMap)
    }
}

impl Display for Work {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
            "    Activity: {:>8?}    |{:>11}|{:>14?}|{:>8}|{:>6}|",
            self.activity,
            self.resource.to_string(),
            self.operation_info.work_remaining,
            self.operation_analytic.duration.as_ref().unwrap().work(),
            self.operation_info.number,
        )
    }
}

impl IntoExcelData for Work {
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = self.0.to_f64().unwrap();
        worksheet.write_number(row, col, value)
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
        format: &rust_xlsxwriter::Format,
    ) -> Result<&'a mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = self.0.to_f64().unwrap();
        worksheet.write_number_with_format(row, col, value, format)
    }
}
