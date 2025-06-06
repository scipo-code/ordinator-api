use rust_xlsxwriter::IntoExcelData;
use serde::Deserialize;
use serde::Serialize;

use crate::Asset;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionalLocation {
    pub string: String,
    pub asset: Asset,
}

impl FunctionalLocation {
    pub fn new(functional_location: &str) -> Self {
        let asset_string = functional_location
            .split(' ')
            .next()
            .expect("All work orders need to have an Asset");
        let asset = Asset::new_from_string(asset_string).unwrap_or(Asset::Unknown);
        Self {
            string: functional_location.to_string(),
            asset,
        }
    }

    pub fn sector(&self) -> Option<&str> {
        self.string.split('/').nth(1)
    }

    pub fn system(&self) -> Option<char> {
        self.string.split('/').nth(2)?.chars().nth(1)
    }

    pub fn subsystem(&self) -> Option<char> {
        self.string.split('/').nth(2)?.chars().nth(1)
    }

    pub fn equipment_tag(&self) -> Option<&str> {
        self.string.split('/').nth(3)
    }
}

impl Default for FunctionalLocation {
    fn default() -> Self {
        FunctionalLocation {
            string: "Unknown".to_string(),
            asset: Asset::Unknown,
        }
    }
}

impl IntoExcelData for FunctionalLocation {
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
