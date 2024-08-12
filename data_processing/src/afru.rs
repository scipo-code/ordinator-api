use crate::{CHAR, CLNT, DEC, NUMC, QUAN, UNIT};

#[allow(non_snake_case)]
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
