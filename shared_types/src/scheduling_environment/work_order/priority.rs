use rust_xlsxwriter::{IntoExcelData, Worksheet};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Priority {
    IntValue(u64),
    StringValue(String),
}

impl Priority {
    pub fn get_priority_string(&self) -> String {
        match self {
            Priority::IntValue(priority) => priority.to_string(),
            Priority::StringValue(priority) => priority.to_string(),
        }
    }
}

impl Priority {
    pub fn new_int(priority: u64) -> Self {
        Self::IntValue(priority)
    }
}

impl IntoExcelData for Priority {
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = self.get_priority_string();
        worksheet.write_string(row, col, value)
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
        format: &rust_xlsxwriter::Format,
    ) -> Result<&'a mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = self.get_priority_string();
        worksheet.write_string_with_format(row, col, value, format)
    }
}
