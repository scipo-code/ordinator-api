use crate::time_environment::period::Period;
use clap::Args;
use rust_xlsxwriter::IntoExcelData;
use serde::{Deserialize, Serialize};

#[derive(Default, Args, Clone, Serialize, Deserialize, Debug)]
pub struct UnloadingPoint {
    pub string: String,
    pub period: Option<Period>,
}

impl UnloadingPoint {
    pub fn new(string: String, periods: &[Period]) -> Self {
        let start_year_and_weeks = extract_year_and_weeks(&string);
        UnloadingPoint {
            string: string.clone(),
            period: periods
                .iter()
                .find(|&period| {
                    if start_year_and_weeks.0.is_some() {
                        period.year == start_year_and_weeks.0.unwrap() + 2000
                            && (period.start_week == start_year_and_weeks.1.unwrap_or(0)
                                || period.finish_week == start_year_and_weeks.1.unwrap_or(0))
                    } else {
                        period.start_week == start_year_and_weeks.1.unwrap_or(0)
                            || period.finish_week == start_year_and_weeks.1.unwrap_or(0)
                    }
                })
                .cloned(),
        }
    }
}

impl IntoExcelData for UnloadingPoint {
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = self.string;
        worksheet.write_string(row, col, value)
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
        format: &rust_xlsxwriter::Format,
    ) -> Result<&'a mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = self.string;
        worksheet.write_string_with_format(row, col, value, format)
    }
}
fn extract_year_and_weeks(input_string: &str) -> (Option<i32>, Option<u32>, Option<u32>) {
    let re = regex::Regex::new(r"(\d{2})?-?[W|w](\d+)-?[W|w]?(\d+)").unwrap();
    let captures = re.captures(input_string);

    match captures {
        Some(cap) => (
            cap.get(1).map_or("", |m| m.as_str()).parse().ok(),
            cap.get(2).map_or("", |m| m.as_str()).parse().ok(),
            cap.get(3).map_or("", |m| m.as_str()).parse().ok(),
        ),
        None => (None, None, None),
    }
}
