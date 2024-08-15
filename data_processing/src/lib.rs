pub mod afih;
pub mod afko;
pub mod afru;
pub mod afvc;
pub mod afvv;
pub mod aufk;
pub mod aufm;
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

pub struct CHAR(String);
pub struct NUMC(u32);
pub struct FLTP();
pub struct DATS(pub String);
pub struct TIMS(pub String);
pub struct CLNT();
pub struct INT1(u8);
pub struct INT4(u32);
pub struct UNIT(String);
pub struct DEC(Decimal);
pub struct QUAN(u32);
pub struct CURR(Decimal);
pub struct LANG(String);

impl Into<NaiveDate> for DATS {
    fn into(self) -> NaiveDate {
        let string = self.0;

        dbg!(&string);
        NaiveDate::parse_from_str(string.as_str(), "%Y%m%d").unwrap()
    }
}

impl Into<NaiveTime> for TIMS {
    fn into(self) -> NaiveTime {
        let mut string = self.0;
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

        dbg!(seconds);
        dbg!(minutes);
        dbg!(hours);

        if hours == 24 && minutes == 0 && seconds == 0 {
            return NaiveTime::from_hms_opt(23, 59, 59).unwrap();
        }
        NaiveTime::from_hms_opt(hours, minutes, seconds).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveTime;

    use crate::TIMS;

    #[test]
    fn test_TIMS_into_trait_impl_1() {
        let tims = TIMS("80142".to_string());

        let naive_time: NaiveTime = tims.into();

        assert_eq!(naive_time, NaiveTime::from_hms_opt(8, 1, 42).unwrap())
    }
    #[test]
    fn test_TIMS_into_trait_impl_2() {
        let tims = TIMS("180142".to_string());

        let naive_time: NaiveTime = tims.into();

        assert_eq!(naive_time, NaiveTime::from_hms_opt(18, 1, 42).unwrap())
    }
    #[test]
    fn test_TIMS_into_trait_impl_3() {
        let tims = TIMS("80000".to_string());

        let naive_time: NaiveTime = tims.into();

        assert_eq!(naive_time, NaiveTime::from_hms_opt(8, 0, 0).unwrap())
    }
    #[test]
    fn test_TIMS_into_trait_impl_4() {
        let tims = TIMS("00000".to_string());

        let naive_time: NaiveTime = tims.into();

        assert_eq!(naive_time, NaiveTime::from_hms_opt(0, 0, 0).unwrap())
    }
    #[test]
    fn test_TIMS_into_trait_impl_5() {
        let tims = TIMS("240000".to_string());

        let naive_time: NaiveTime = tims.into();

        assert_eq!(naive_time, NaiveTime::from_hms_opt(23, 59, 59).unwrap())
    }
}
