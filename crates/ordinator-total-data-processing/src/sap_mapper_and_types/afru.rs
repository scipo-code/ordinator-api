use crate::sap_mapper_and_types::CHAR;
use crate::sap_mapper_and_types::CLNT;
use crate::sap_mapper_and_types::DEC;
use crate::sap_mapper_and_types::NUMC;
use crate::sap_mapper_and_types::QUAN;
use crate::sap_mapper_and_types::UNIT;

#[allow(non_snake_case)]
#[allow(dead_code)]
struct Afru {
    MANDT: CLNT,
    RUECK: NUMC,
    RMZHL: NUMC,
    ARBID: NUMC,
    WERKS: CHAR,
    ISERH: QUAN,
    ZEIER: UNIT,
    ISMNW: QUAN,
    ISMNE: UNIT,
    IDAUR: QUAN,
    IDAUE: UNIT,
    ANZMA: DEC,
    PERNR: NUMC,
    AUFPL: NUMC,
    AUFNR: CHAR,
    VORNR: CHAR,
    OFMNW: QUAN,
    OFMNE: UNIT,
    ODAUR: QUAN,
    ODAUE: UNIT,
    SMENG: QUAN,
}
