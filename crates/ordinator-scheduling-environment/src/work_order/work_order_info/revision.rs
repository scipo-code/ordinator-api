use std::fmt::Display;

use rust_xlsxwriter::IntoExcelData;
use serde::{Deserialize, Serialize};
#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct Revision {
    pub revision_code: RevisionCode,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub enum RevisionCode {
    #[default]
    Ne,
    Nosd,
    Code(String),
}

impl Revision {
    pub fn new(str: &str) -> Self {
        let revision_code = match str {
            "NE" => RevisionCode::Ne,
            "NOSD" => RevisionCode::Nosd,
            _ => RevisionCode::Code(str.to_string()),
        };
        Revision { revision_code }
    }

    pub fn shutdown(&self) -> bool {
        match self.revision_code {
            // Careful here talk with [[Brian Friis Nielsen]]
            RevisionCode::Ne => false,
            RevisionCode::Nosd => false,
            RevisionCode::Code(_) => true,
        }
    }
}

impl Display for Revision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.revision_code {
            RevisionCode::Ne => write!(f, "NE"),
            RevisionCode::Nosd => write!(f, "NOSD"),
            RevisionCode::Code(string) => write!(f, "{}", string),
        }
    }
}

impl IntoExcelData for Revision {
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = self.to_string();
        worksheet.write_string(row, col, value)
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
        format: &rust_xlsxwriter::Format,
    ) -> Result<&'a mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = self.to_string();
        worksheet.write_string_with_format(row, col, value, format)
    }
}
