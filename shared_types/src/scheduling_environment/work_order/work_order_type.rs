use rust_xlsxwriter::IntoExcelData;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WorkOrderType {
    Wdf(WDFPriority),
    Wgn(WGNPriority),
    Wpm(WPMPriority),
    Wro(WROPriority),
    Other,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum WDFPriority {
    One,
    Two,
    Three,
    Four,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum WGNPriority {
    One,
    Two,
    Three,
    Four,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum WROPriority {
    One,
    Two,
    Three,
    Four,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum WPMPriority {
    A,
    B,
    C,
    D,
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
}

impl WDFPriority {
    pub fn new(priority: u32) -> Self {
        match priority {
            1 => Self::One,
            2 => Self::Two,
            3 => Self::Three,
            4 => Self::Four,
            _ => Self::Four,
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