use crate::sap_mapper_and_types::CHAR;
use crate::sap_mapper_and_types::CLNT;
use crate::sap_mapper_and_types::CURR;
use crate::sap_mapper_and_types::DATS;
use crate::sap_mapper_and_types::NUMC;
use crate::sap_mapper_and_types::QUAN;
use crate::sap_mapper_and_types::TIMS;

#[allow(non_snake_case)]
#[allow(dead_code)]
struct Aufk {
    MANDT: CLNT,
    AUFNR: CHAR,
    AUART: CHAR,
    AUTYP: NUMC,
    REFNR: CHAR,
    ERDAT: DATS,
    AENAM: CHAR,
    KTEXT: CHAR,
    LTEXT: CHAR,
    WERKS: CHAR,
    KOSTV: CHAR,
    STORT: CHAR,
    SOWRK: CHAR,
    ASTNR: NUMC,
    PHAS0: CHAR,
    PHAS1: CHAR,
    PHAS2: CHAR,
    PHAS3: CHAR,
    IDAT1: DATS,
    USER4: CURR,
    USER9: CHAR,
    OBJNR: CHAR,
    PSPEL: NUMC,
    ERFZEIT: TIMS,
    AEZEIT: TIMS,
    YYAWSC: CHAR,
    YYHOURS: QUAN,
    ZZGSTRP: DATS,
    ZZGLTRP: DATS,
    ZZ_OLAFD: DATS,
    ZZ_LAFD: DATS,
    ZZ_EASD: DATS,
    VAPLZ: CHAR,
}
