use crate::scheduling_environment::time_environment::period::Period;
use clap::Args;
use rust_xlsxwriter::IntoExcelData;
use serde::{Deserialize, Serialize};

#[derive(Default, Args, Clone, Serialize, Deserialize, Debug)]
pub struct UnloadingPoint {
    pub string: String,
    pub period: Option<Period>,
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
