use crate::sap_mapper_and_types::CHAR;
use crate::sap_mapper_and_types::CLNT;
use crate::sap_mapper_and_types::DATS;
use crate::sap_mapper_and_types::FLTP;
use crate::sap_mapper_and_types::INT4;
use crate::sap_mapper_and_types::NUMC;
use crate::sap_mapper_and_types::TIMS;
use crate::sap_mapper_and_types::UNIT;

#[allow(non_snake_case)]
#[allow(dead_code)]
struct Afih
{
    MANDT: CLNT,
    AUFNR: CHAR,
    ARTPR: CHAR,
    PRIOK: CHAR,
    EQUNR: CHAR,
    BAUTL: CHAR,
    ILOAN: CHAR,
    ILOAI: CHAR,
    ANLZU: CHAR,
    IWERK: CHAR,
    APGRP: CHAR,
    GEWRK: NUMC,
    ANING: CHAR,
    GAUZT: FLTP,
    GAUEH: UNIT,
    INSPK: CHAR,
    DATAN: DATS,
    WARPL: CHAR,
    ABNUM: INT4,
    WAPOS: CHAR,
    LAUFN: CHAR,
    OBKNR: INT4,
    REVNR: CHAR,
    ADDAT: DATS,
    ADUHR: TIMS,
    SERMAT: CHAR,
}
