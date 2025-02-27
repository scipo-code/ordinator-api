use rust_xlsxwriter::IntoExcelData;
use serde::{Deserialize, Serialize};
#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct Revision {
    pub revision_code: RevisionCode,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
enum RevisionCode {
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

impl ToString for Revision {
    fn to_string(&self) -> String {
        match &self.revision_code {
            RevisionCode::Ne => "NE".to_string(),
            RevisionCode::Nosd => "NOSD".to_string(),
            RevisionCode::Code(string) => string.to_string(),
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
