use std::fmt::{self, Display};

use chrono::{DateTime, Utc};
use rust_xlsxwriter::IntoExcelData;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Hash, Clone, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Day {
    day_index: usize,
    date: DateTime<Utc>,
}

impl Day {
    pub fn new(day_index: usize, date: DateTime<Utc>) -> Self {
        Day { day_index, date }
    }

    pub fn date(&self) -> &DateTime<Utc> {
        &self.date
    }

    pub fn day_index(&self) -> &usize {
        &self.day_index
    }
}

impl Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.date.date_naive())
    }
}

#[derive(Debug, Clone)]
pub struct OptionDay(pub Option<Day>);

impl IntoExcelData for OptionDay {
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = match self.0 {
            Some(day) => day.date.to_string(),
            None => "".to_string(),
        };

        worksheet.write_string(row, col, value)
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
        format: &rust_xlsxwriter::Format,
    ) -> Result<&'a mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = match self.0 {
            Some(day) => day.date.to_string(),
            None => "".to_string(),
        };

        worksheet.write_string_with_format(row, col, value, format)
    }
}
