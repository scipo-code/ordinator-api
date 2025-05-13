use rust_xlsxwriter::IntoExcelData;
use serde::Deserialize;
use serde::Serialize;

use super::priority::Priority;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum WorkOrderType
{
    Wdf(Priority),
    Wgn(Priority),
    Wpm(Priority),
    Wro(Priority),
    #[default]
    Other,
}

impl WorkOrderType
{
    pub fn get_type_string(&self) -> String
    {
        match self {
            WorkOrderType::Wdf(_) => "WDF".to_owned(),
            WorkOrderType::Wgn(_) => "WGN".to_owned(),
            WorkOrderType::Wpm(_) => "WPM".to_owned(),
            WorkOrderType::Wro(_) => "WRO".to_owned(),
            WorkOrderType::Other => "Other".to_owned(),
        }
    }

    pub fn new(work_order_type_string: &str, priority: Priority) -> Result<Self, String>
    {
        match work_order_type_string {
            "WDF" => Ok(WorkOrderType::Wdf(priority)),
            "WGN" => Ok(WorkOrderType::Wgn(priority)),
            "WPM" => Ok(WorkOrderType::Wpm(priority)),
            "WRO" => Ok(WorkOrderType::Wro(priority)),
            _ => Err(format!(
                "WorkOrderType: {work_order_type_string} is not valid in Ordinator yet",
            )),
        }
    }

    pub fn valid_work_order_type(str: &str) -> bool
    {
        matches!(str, "WDF" | "WGN" | "WPM" | "WRO")
    }
}

impl IntoExcelData for WorkOrderType
{
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError>
    {
        let value = self.get_type_string();
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
        let value = self.get_type_string();
        worksheet.write_string_with_format(row, col, value, format)
    }
}
