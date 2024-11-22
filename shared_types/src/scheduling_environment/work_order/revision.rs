use rust_xlsxwriter::IntoExcelData;
use serde::{Deserialize, Serialize};
#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct Revision {
    pub string: String,
    pub shutdown: bool,
}

impl Revision {
    pub fn new(string: String) -> Self {
        Revision {
            string: string.clone(),
            shutdown: !string.contains("NOSD"),
        }
    }

    pub fn new_with_shutdown(string: String, shutdown: bool) -> Self {
        Revision { string, shutdown }
    }
}

impl IntoExcelData for Revision {
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
