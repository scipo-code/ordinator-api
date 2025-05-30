use crate::sap_mapper_and_types::CHAR;
use crate::sap_mapper_and_types::CLNT;
use crate::sap_mapper_and_types::DEC;
use crate::sap_mapper_and_types::INT1;
use crate::sap_mapper_and_types::NUMC;

#[allow(non_snake_case, dead_code)]
struct Afvc {
    MANDT: CLNT,
    AUFPL: NUMC,
    APLZL: NUMC,
    PLNFL: CHAR,
    VORNR: CHAR,
    STEUS: CHAR,
    ARBID: NUMC,
    LTXA1: CHAR,
    ANZMA: DEC,
    ANZZL: INT1,
    PRZNT: INT1,
    LARNT: CHAR,
    RUECK: NUMC,
    RMZHL: NUMC,
    OBJNR: CHAR,
    SPANZ: DEC,
    BEDID: NUMC,
    ANLZU: CHAR,
    NPRIO: CHAR,
    PSPNR: NUMC,
    SCOPE: CHAR,
    NO_DISP: CHAR,
    ARBII: NUMC,
    WERKI: CHAR,
    WEMPF: CHAR,
    ABLAD: CHAR,
    SCHED_END: CHAR,
    PERNR: NUMC,
    OIO_HOLD: CHAR,
    TPLNR: CHAR,
}
