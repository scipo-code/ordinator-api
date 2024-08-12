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
use rust_decimal::Decimal;

pub struct CHAR(String);
pub struct NUMC(u32);
pub struct FLTP();
pub struct DATS(u32);
pub struct TIMS(u32);
pub struct CLNT();
pub struct INT1(u8);
pub struct INT4(u32);
pub struct UNIT(String);
pub struct DEC(Decimal);
pub struct QUAN(u32);
pub struct CURR(Decimal);
pub struct LANG(String);

fn generate_source_file() {}
