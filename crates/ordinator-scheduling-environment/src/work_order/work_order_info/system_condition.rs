use std::fmt::Display;
use std::str::FromStr;
use std::string::ParseError;

use rust_xlsxwriter::IntoExcelData;
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub enum SystemCondition
{
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    #[default]
    Unknown,
}

impl FromStr for SystemCondition
{
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let system_condition = match s {
            "A" => SystemCondition::A,
            "B" => SystemCondition::B,
            "C" => SystemCondition::C,
            "D" => SystemCondition::D,
            "E" => SystemCondition::E,
            "F" => SystemCondition::F,
            "G" => SystemCondition::G,
            "H" => SystemCondition::H,
            "I" => SystemCondition::I,
            "J" => SystemCondition::J,
            _ => {
                dbg!(&s);
                panic!("SystemCondition should be a capital character between [A-J]");
            }
        };
        Ok(system_condition)
    }
}

impl Display for SystemCondition
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let string = match self {
            SystemCondition::A => "A".to_string(),
            SystemCondition::B => "B".to_string(),
            SystemCondition::C => "C".to_string(),
            SystemCondition::D => "D".to_string(),
            SystemCondition::E => "E".to_string(),
            SystemCondition::F => "F".to_string(),
            SystemCondition::G => "G".to_string(),
            SystemCondition::H => "H".to_string(),
            SystemCondition::I => "I".to_string(),
            SystemCondition::J => "J".to_string(),
            SystemCondition::Unknown => "Unknown".to_string(),
        };
        write!(f, "{string}")
    }
}
impl IntoExcelData for SystemCondition
{
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError>
    {
        let value = self.to_string();
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
        let value = self.to_string();
        worksheet.write_string_with_format(row, col, value, format)
    }
}
