use crate::sap_mapper_and_types::{CHAR, CLNT, LANG};

#[allow(dead_code, non_snake_case)]
struct Iflotx {
    MANDT: CLNT,
    TPLNR: CHAR,
    SPRAS: LANG,
    PLTXT: CHAR,
}
