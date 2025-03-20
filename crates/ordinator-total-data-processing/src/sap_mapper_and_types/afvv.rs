use crate::sap_mapper_and_types::CHAR;
use crate::sap_mapper_and_types::CLNT;
use crate::sap_mapper_and_types::DATS;
use crate::sap_mapper_and_types::DEC;
use crate::sap_mapper_and_types::FLTP;
use crate::sap_mapper_and_types::NUMC;
use crate::sap_mapper_and_types::QUAN;
use crate::sap_mapper_and_types::TIMS;
use crate::sap_mapper_and_types::UNIT;

#[allow(non_snake_case, dead_code)]
struct Afvv
{
    MANDT: CLNT,
    AUFPL: NUMC,
    APLZL: NUMC,
    MEINH: UNIT,
    DAUNO: QUAN,
    DAUNE: UNIT,
    DAUMI: QUAN,
    DAUME: UNIT,
    EINSA: CHAR,
    EINSE: CHAR,
    ARBEI: QUAN,
    ARBEH: UNIT,
    MGVRG: QUAN,
    ISMNW: QUAN,
    PUFFR: DEC,
    PUFGS: DEC,
    NTANF: DATS,
    NTANZ: TIMS,
    NTEND: DATS,
    NTENZ: TIMS,
    BEARZ: FLTP,
    OFMNW: QUAN,
    AUFKT: DEC,
}
