use crate::sap_mapper_and_types::CHAR;
use crate::sap_mapper_and_types::CLNT;
use crate::sap_mapper_and_types::LANG;

#[allow(dead_code, non_snake_case)]
struct Iflotx
{
    MANDT: CLNT,
    TPLNR: CHAR,
    SPRAS: LANG,
    PLTXT: CHAR,
}
