use std::any::Any;

use rust_xlsxwriter::ColNum;
use rust_xlsxwriter::Format;
use rust_xlsxwriter::IntoExcelData;
use rust_xlsxwriter::RowNum;
use rust_xlsxwriter::Worksheet;
use rust_xlsxwriter::XlsxError;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Priority {
    Int(u64),
    Char(char),
}

// Is this a better way of doing it? Yes it is a much better way

#[derive(Clone, Serialize, Deserialize, Debug)]
struct IntPriority(u64);

#[derive(Clone, Serialize, Deserialize, Debug)]
struct CharPriority(char);

impl Priority {
    pub fn get_priority_string(&self) -> String {
        match self {
            Priority::Int(priority) => priority.to_string(),
            Priority::Char(priority) => priority.to_string(),
        }
    }

    pub fn dyn_new(input: Box<dyn Any>) -> Self {
        if let Some(int) = input.downcast_ref::<u64>() {
            Priority::Int(*int)
        } else if let Some(char) = input.downcast_ref::<char>() {
            Priority::Char(*char)
        } else {
            let string_value = input.downcast_ref::<String>().unwrap();

            match string_value.parse::<u64>() {
                Ok(int) => Priority::Int(int),
                Err(_) => {
                    let char = string_value
                        .parse::<char>()
                        .expect("priority should be either parsed as an int or a char");

                    Priority::Char(char)
                }
            }
        }
    }

    pub fn new_int(priority: u64) -> Self {
        Self::Int(priority)
    }
}

impl IntoExcelData for Priority {
    fn write(
        self,
        worksheet: &mut Worksheet,
        row: RowNum,
        col: ColNum,
    ) -> Result<&mut Worksheet, XlsxError> {
        let value = self.get_priority_string();
        worksheet.write_string(row, col, value)
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut Worksheet,
        row: RowNum,
        col: ColNum,
        format: &Format,
    ) -> Result<&'a mut Worksheet, XlsxError> {
        let value = self.get_priority_string();
        worksheet.write_string_with_format(row, col, value, format)
    }
}
