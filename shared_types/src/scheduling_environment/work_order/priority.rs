use std::any::Any;

use rust_xlsxwriter::{ColNum, Format, IntoExcelData, RowNum, Worksheet, XlsxError};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Priority {
    IntValue(u64),
    StringValue(char),
}

impl Priority {
    pub fn get_priority_string(&self) -> String {
        match self {
            Priority::IntValue(priority) => priority.to_string(),
            Priority::StringValue(priority) => priority.to_string(),
        }
    }

    pub fn dyn_new(input: Box<dyn Any>) -> Self {
        if let Some(int) = input.downcast_ref::<u64>() {
            Priority::IntValue(*int)
        } else if let Some(char) = input.downcast_ref::<char>() {
            Priority::StringValue(*char)
        } else {
            panic!("Priority can only be an int or a char")
        }
    }

    pub fn new_int(priority: u64) -> Self {
        Self::IntValue(priority)
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
