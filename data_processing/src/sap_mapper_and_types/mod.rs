pub mod afih;
pub mod afko;
pub mod afru;
pub mod afvc;
pub mod afvv;
pub mod aufk;
pub mod aufm;
pub mod iflot;
pub mod iflotx;
pub mod iloa;
pub mod t352r;
pub mod tj02;
pub mod tj02t;
pub mod tj20;
pub mod tj30;
pub mod tj30t;

use chrono::{NaiveDate, NaiveTime};
use rust_decimal::Decimal;
use rust_xlsxwriter::IntoExcelData;

#[allow(dead_code)]
pub struct CHAR(String);
#[allow(dead_code)]
pub struct NUMC(u32);
#[allow(dead_code)]
pub struct FLTP();
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DATS(pub String);
#[allow(dead_code)]
pub struct TIMS(pub String);
#[allow(dead_code)]
pub struct CLNT();
#[allow(dead_code)]
pub struct INT1(u8);
#[allow(dead_code)]
pub struct INT4(u32);
#[allow(dead_code)]
pub struct UNIT(String);
#[allow(dead_code)]
pub struct DEC(Decimal);
#[allow(dead_code)]
pub struct QUAN(u32);
#[allow(dead_code)]
pub struct CURR(Decimal);
#[allow(dead_code)]
pub struct LANG(String);

impl TryFrom<DATS> for NaiveDate {
    type Error = String;

    fn try_from(value: DATS) -> Result<NaiveDate, Self::Error> {
        let string = value.0.trim_end_matches(".0");

        if string == "0" {
            dbg!(string, &value.0);
            return Err("Empty DATS value".to_string());
        };

        let naive_date_result = NaiveDate::parse_from_str(&string, "%Y%m%d");

        match naive_date_result {
            Ok(naive_date) => Ok(naive_date),
            Err(_) => {
                dbg!(&string);
                panic!("DATS can, according to SAP documentation only be of the form 'YYYYMMDD' or '0' for empty");
            }
        }
    }
}

impl From<NaiveDate> for DATS {
    fn from(value: NaiveDate) -> Self {
        let string = value.to_string();

        // string.remove_matches("-");
        Self(string)
    }
}

impl From<TIMS> for NaiveTime {
    fn from(value: TIMS) -> Self {
        let mut string = value.0;
        let mut seconds = vec![];
        let mut minutes = vec![];
        let mut hours = vec![];

        let mut count = 0;
        while !string.is_empty() {
            let letter = string.pop().unwrap();
            if [0, 1].contains(&count) {
                seconds.push(letter);
            } else if [2, 3].contains(&count) {
                minutes.push(letter);
            } else {
                hours.push(letter);
            }
            count += 1;
        }

        seconds.reverse();
        let seconds: u32 = seconds.iter().collect::<String>().parse::<u32>().unwrap();
        minutes.reverse();
        let minutes: u32 = match minutes.iter().collect::<String>().parse::<u32>() {
            Ok(minutes) => minutes,
            Err(_) => {
                return NaiveTime::from_hms_opt(0, 0, seconds).unwrap();
            }
        };
        hours.reverse();

        let hours: u32 = match hours.iter().collect::<String>().parse::<u32>() {
            Ok(hours) => hours,
            Err(_) => {
                return NaiveTime::from_hms_opt(0, minutes, seconds).unwrap();
            }
        };

        if hours == 24 && minutes == 0 && seconds == 0 {
            return NaiveTime::from_hms_opt(23, 59, 59).unwrap();
        }
        NaiveTime::from_hms_opt(hours, minutes, seconds).unwrap()
    }
}
impl IntoExcelData for DATS {
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = self.0;
        worksheet.write_string(row, col, value)
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
        format: &rust_xlsxwriter::Format,
    ) -> Result<&'a mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let value = self.0;
        worksheet.write_string_with_format(row, col, value, format)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveTime};

    use crate::sap_mapper_and_types::TIMS;

    use super::DATS;

    #[test]
    #[allow(non_snake_case)]
    fn test_TIMS_into_trait_impl_1() {
        let tims = TIMS("80142".to_string());

        let naive_time: NaiveTime = tims.into();

        assert_eq!(naive_time, NaiveTime::from_hms_opt(8, 1, 42).unwrap())
    }
    #[test]
    #[allow(non_snake_case)]
    fn test_TIMS_into_trait_impl_2() {
        let tims = TIMS("180142".to_string());

        let naive_time: NaiveTime = tims.into();

        assert_eq!(naive_time, NaiveTime::from_hms_opt(18, 1, 42).unwrap())
    }
    #[test]
    #[allow(non_snake_case)]
    fn test_TIMS_into_trait_impl_3() {
        let tims = TIMS("80000".to_string());

        let naive_time: NaiveTime = tims.into();

        assert_eq!(naive_time, NaiveTime::from_hms_opt(8, 0, 0).unwrap())
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_TIMS_into_trait_impl_4() {
        let tims = TIMS("00000".to_string());

        let naive_time: NaiveTime = tims.into();

        assert_eq!(naive_time, NaiveTime::from_hms_opt(0, 0, 0).unwrap())
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_TIMS_into_trait_impl_5() {
        let tims = TIMS("240000".to_string());

        let naive_time: NaiveTime = tims.into();

        assert_eq!(naive_time, NaiveTime::from_hms_opt(23, 59, 59).unwrap())
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_DATS() {
        let dats = DATS("20250103".to_string());

        let naive_date: NaiveDate = dats.try_into().unwrap();

        assert_eq!(naive_date, NaiveDate::from_ymd_opt(2025, 1, 3).unwrap());
    }
}
