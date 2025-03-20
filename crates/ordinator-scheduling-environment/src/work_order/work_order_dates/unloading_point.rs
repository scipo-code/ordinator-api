use clap::Args;
use rust_xlsxwriter::IntoExcelData;
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Args, Clone, Serialize, Deserialize, Debug)]
pub struct UnloadingPoint
{
    pub string: String,
}
// This field simply needs to be derived when we want a period. That is the only
// way of implementing it in a way that will scale into the future. You cannot
// afford not to FIX things that are in error.
impl UnloadingPoint
{
    pub fn new(string: String) -> Self
    {
        UnloadingPoint {
            string: string.clone(),
        }
    }
}

impl IntoExcelData for UnloadingPoint
{
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError>
    {
        let value = self.string;
        worksheet.write_string(row, col, value)
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
        format: &rust_xlsxwriter::Format,
    ) -> Result<&'a mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError>
    {
        let value = self.string;
        worksheet.write_string_with_format(row, col, value, format)
    }
}
