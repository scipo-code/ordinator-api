use crate::sap_mapper_and_types::CHAR;
use crate::sap_mapper_and_types::CLNT;
use crate::sap_mapper_and_types::NUMC;

#[allow(non_snake_case)]
#[allow(dead_code)]
struct Aufm {
    MANDT: CLNT,
    MBLNR: CHAR,
    MJAHR: NUMC,
    ZEILE: NUMC,
    ABLAD: CHAR,
    AUFNR: CHAR,
}
