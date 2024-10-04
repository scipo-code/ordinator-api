use rust_xlsxwriter::IntoExcelData;
use serde::{Deserialize, Serialize};

use super::priority::Priority;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WorkOrderType {
    Wdf(Priority),
    Wgn(Priority),
    Wpm(Priority),
    Wro(Priority),
    Other,
}

impl WorkOrderType {
    pub fn get_type_string(&self) -> String {
        match self {
            WorkOrderType::Wdf(_) => "WDF".to_owned(),
            WorkOrderType::Wgn(_) => "WGN".to_owned(),
            WorkOrderType::Wpm(_) => "WPM".to_owned(),
            WorkOrderType::Wro(_) => "WRO".to_owned(),
            WorkOrderType::Other => "Other".to_owned(),
        }
    }

    pub fn new(work_order_type_string: &str, priority: Priority) -> Self {
        match work_order_type_string {
            "WDF" => WorkOrderType::Wdf(priority),
            "WGN" => WorkOrderType::Wdf(priority),
            "WPM" => WorkOrderType::Wdf(priority),
            "WRO" => WorkOrderType::Wdf(priority),
            _ => todo!("Missing implementation for work order type"),
        }
    }
}

impl IntoExcelData for WorkOrderType {
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = self.get_type_string();
        worksheet.write_string(row, col, value)
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
        format: &rust_xlsxwriter::Format,
    ) -> Result<&'a mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = self.get_type_string();
        worksheet.write_string_with_format(row, col, value, format)
    }
}
