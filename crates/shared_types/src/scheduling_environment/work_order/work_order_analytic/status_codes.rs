use std::fmt::{self, Display};

use clap::{Args, ValueEnum};
use regex::Regex;
use rust_xlsxwriter::IntoExcelData;
use serde::{Deserialize, Serialize};

use crate::scheduling_environment::{
    time_environment::period::Period, work_order::WorkOrderNumber,
};

// pub material_status: MaterialStatus,
// #[arg(long)]
// pub pcnf: bool,
// #[arg(long)]
// pub awsc: bool,
// #[arg(long)]
// pub well: bool,
// #[arg(long)]
// pub sch: bool,
// #[arg(long)]
// pub sece: bool,
// #[arg(long)]
// pub unloading_point: bool,
// TODO MaterialStatus
// TODO unloading_point

#[derive(Default, Args, Clone, Serialize, Deserialize, Debug)]
pub struct SystemStatusCodes {
    #[arg(long)]
    pub rel: bool,
    #[arg(long)]
    pub prc: bool,
    #[arg(long)]
    pub setc: bool,
    #[arg(long)]
    pub ssap: bool,
    #[arg(long)]
    pub gmps: bool,
    #[arg(long)]
    pub manc: bool,
    #[arg(long)]
    pub crtd: bool,
    #[arg(long)]
    pub nmat: bool,
    #[arg(long)]
    pub teco: bool,
    #[arg(long)]
    pub macm: bool,
    #[arg(long)]
    pub mspt: bool,
    #[arg(long)]
    pub pprt: bool,
    #[arg(long)]
    pub ncmp: bool,
    #[arg(long)]
    pub clsd: bool,
    #[arg(long)]
    pub pcnf: bool,
    #[arg(long)]
    pub cser: bool,
    #[arg(long)]
    pub prt: bool,
    #[arg(long)]
    pub cnf: bool,
    #[arg(long)]
    pub ntup: bool,
    #[arg(long)]
    pub estc: bool,
    #[arg(long)]
    pub relr: bool,
    #[arg(long)]
    pub gmco: bool,
}
impl SystemStatusCodes {
    pub(crate) fn builder() -> SystemStatusCodesBuilder {
        // QUESTION
        // How to handle this?
        // You should use the `Default` trait and then rely on
        // a `from_data` methods when you need to initialize
        // directly from SAP data. I do not see a different
        // way of doing it.
        //
        //
        SystemStatusCodesBuilder(SystemStatusCodes)
    }
}

#[derive(Default, Args, Clone, Serialize, Deserialize, Debug)]
pub struct UserStatusCodes {
    #[arg(long)]
    pub appr: bool,
    #[arg(long)]
    pub smat: bool,
    #[arg(long)]
    pub init: bool,
    #[arg(long)]
    pub rdbl: bool,
    #[arg(long)]
    pub qcap: bool,
    #[arg(long)]
    pub rfrz: bool,
    #[arg(long)]
    pub wmat: bool,
    #[arg(long)]
    pub cmat: bool,
    #[arg(long)]
    pub pmat: bool,
    #[arg(long)]
    pub apog: bool,
    #[arg(long)]
    pub prok: bool,
    #[arg(long)]
    pub wrea: bool,
    #[arg(long)]
    pub exdo: bool,
    #[arg(long)]
    pub swe: bool,
    #[arg(long)]
    pub awdo: bool,
    #[arg(long)]
    pub rout: bool,
    #[arg(long)]
    pub wta: bool,
    #[arg(long)]
    pub sch: bool,
    #[arg(long)]
    pub sece: bool,
    #[arg(long)]
    pub rel: bool,
    #[arg(long)]
    pub rees: bool,
    #[arg(long)]
    pub reap: bool,
    #[arg(long)]
    pub wrel: bool,
    #[arg(long)]
    pub awsd: bool,
    #[arg(long)]
    pub sraa: bool,
    #[arg(long)]
    pub qcrj: bool,
    #[arg(long)]
    pub awsc: bool,
    #[arg(long)]
    pub lprq: bool,
    #[arg(long)]
    pub rrev: bool,
    #[arg(long)]
    pub awca: bool,
    #[arg(long)]
    pub rreq: bool,
    #[arg(long)]
    pub vfal: bool,
    #[arg(long)]
    pub sreq: bool,
    #[arg(long)]
    pub amcr: bool,
    #[arg(long)]
    pub dfrj: bool,
    #[arg(long)]
    pub vpas: bool,
    #[arg(long)]
    pub dfcr: bool,
    #[arg(long)]
    pub ireq: bool,
    #[arg(long)]
    pub atvd: bool,
    #[arg(long)]
    pub awmd: bool,
    #[arg(long)]
    pub dfex: bool,
    #[arg(long)]
    pub dfap: bool,
    #[arg(long)]
    pub awpr: bool,
}

struct UserStatusCodesBuilder(UserStatusCodes);

impl UserStatusCodesBuilder {
    pub fn build(self) -> UserStatusCodes {
        UserStatusCodes {
            appr: self.0.appr,
            smat: self.0.smat,
            init: self.0.init,
            rdbl: self.0.rdbl,
            qcap: self.0.qcap,
            rfrz: self.0.rfrz,
            wmat: self.0.wmat,
            cmat: self.0.cmat,
            pmat: self.0.pmat,
            apog: self.0.apog,
            prok: self.0.prok,
            wrea: self.0.wrea,
            exdo: self.0.exdo,
            swe: self.0.swe,
            awdo: self.0.awdo,
            rout: self.0.rout,
            wta: self.0.wta,
            sch: self.0.sch,
            sece: self.0.sece,
            rel: self.0.rel,
            rees: self.0.rees,
            reap: self.0.reap,
            wrel: self.0.wrel,
            awsd: self.0.awsd,
            sraa: self.0.sraa,
            qcrj: self.0.qcrj,
            awsc: self.0.awsc,
            lprq: self.0.lprq,
            rrev: self.0.rrev,
            awca: self.0.awca,
            rreq: self.0.rreq,
            vfal: self.0.vfal,
            sreq: self.0.sreq,
            amcr: self.0.amcr,
            dfrj: self.0.dfrj,
            vpas: self.0.vpas,
            dfcr: self.0.dfcr,
            ireq: self.0.ireq,
            atvd: self.0.atvd,
            awmd: self.0.awmd,
            dfex: self.0.dfex,
            dfap: self.0.dfap,
            awpr: self.0.awpr,
        }
    }

    // These functions will be crucial for testing! I do not
    pub fn smat(&mut self, smat: bool) -> &mut Self {
        self.0.smat = smat;
        self
    }
}

struct SystemStatusCodesBuilder(SystemStatusCodes);

/// Builder this correctly could be a real hassel
// TODO [ ]
// Add remaining fields that are also needed.
// All these status codes should by definition never change
// there is needed something else.
//
// A builder would primarily be for testing.
impl SystemStatusCodesBuilder {
    pub fn build(self) -> SystemStatusCodes {
        SystemStatusCodes {
            rel: self.0.rel,
            prc: self.0.prc,
            setc: self.0.setc,
            ssap: self.0.ssap,
            gmps: self.0.gmps,
            manc: self.0.manc,
            crtd: self.0.crtd,
            nmat: self.0.nmat,
            teco: self.0.teco,
            macm: self.0.macm,
            mspt: self.0.mspt,
            pprt: self.0.pprt,
            ncmp: self.0.ncmp,
            clsd: self.0.clsd,
            pcnf: self.0.pcnf,
            cser: self.0.cser,
            prt: self.0.prt,
            cnf: self.0.cnf,
            ntup: self.0.ntup,
            estc: self.0.estc,
            relr: self.0.relr,
            gmco: self.0.gmco,
        }
    }
    pub fn rel(&mut self, rel: bool) -> &mut Self {
        self.0.rel = rel;
        self
    }
}

#[derive(Args, Clone, Serialize, Deserialize, Debug)]
pub struct StrategicUserStatusCodes {
    /// Provide the work order number for the work order that you want to change.
    pub work_order_numbers: Vec<WorkOrderNumber>,
    #[arg(long, value_parser = clap::value_parser!(bool))]
    pub sch: Option<bool>,
    #[arg(long, value_parser = clap::value_parser!(bool))]
    pub awsc: Option<bool>,
    #[arg(long, value_parser = clap::value_parser!(bool))]
    pub sece: Option<bool>,
}

impl From<UserStatusCodes> for MaterialStatus {
    fn from(value: UserStatusCodes) -> Self {
        assert!(value.smat as u8 + value.pmat as u8 + value.wmat as u8 + value.cmat as u8 <= 1);

        if value.smat {
            MaterialStatus::Smat
        } else if value.pmat {
            MaterialStatus::Pmat
        } else if value.wmat {
            MaterialStatus::Wmat
        } else if value.cmat {
            MaterialStatus::Cmat
        } else {
            MaterialStatus::Nmat
        }
    }
}

#[derive(ValueEnum, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum MaterialStatus {
    Smat,
    Nmat,
    Cmat,
    Wmat,
    Pmat,
    Unknown,
}

impl MaterialStatus {
    pub fn from_status_code_string(status_codes_string: &str) -> Self {
        // Define individual patterns for clarity and precise matching
        let patterns = vec![
            ("SMAT", MaterialStatus::Smat),
            ("NMAT", MaterialStatus::Nmat),
            ("CMAT", MaterialStatus::Cmat),
            ("WMAT", MaterialStatus::Wmat),
            ("PMAT", MaterialStatus::Pmat),
        ];

        // Check each pattern to see if it matches the status code string
        for (pattern, status) in patterns {
            if Regex::new(pattern).unwrap().is_match(status_codes_string) {
                return status;
            }
        }

        MaterialStatus::Unknown
        // If no patterns match, return the Unknown variant
    }

    pub fn period_delay(&self, periods: &[Period]) -> Option<Period> {
        match self {
            Self::Smat => None,
            Self::Nmat => None,
            Self::Cmat => periods.get(1).cloned(),
            Self::Wmat => periods.get(2).cloned(),
            Self::Pmat => periods.get(2).cloned(),
            Self::Unknown => None,
        }
    }
}

impl Display for MaterialStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MaterialStatus::Smat => write!(f, "SMAT"),
            MaterialStatus::Nmat => write!(f, "NMAT"),
            MaterialStatus::Cmat => write!(f, "CMAT"),
            MaterialStatus::Wmat => write!(f, "WMAT"),
            MaterialStatus::Pmat => write!(f, "PMAT"),
            MaterialStatus::Unknown => write!(f, "----"),
        }
    }
}

impl IntoExcelData for SystemStatusCodes {
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let rel = if self.rel { "REL " } else { "" };
        let prc = if self.prc { "PRC " } else { "" };
        let setc = if self.setc { "SETC " } else { "" };
        let ssap = if self.ssap { "SSAP " } else { "" };
        let gmps = if self.gmps { "GMPS " } else { "" };
        let manc = if self.manc { "MANC " } else { "" };
        let crtd = if self.crtd { "CRTD " } else { "" };
        let nmat = if self.nmat { "NMAT " } else { "" };
        let teco = if self.teco { "TECO " } else { "" };
        let macm = if self.macm { "MACM " } else { "" };
        let mspt = if self.mspt { "MSPT " } else { "" };
        let pprt = if self.pprt { "PPRT " } else { "" };
        let ncmp = if self.ncmp { "NCMP " } else { "" };
        let clsd = if self.clsd { "CLSD " } else { "" };
        let pcnf = if self.pcnf { "PCNF " } else { "" };
        let cser = if self.cser { "CSER " } else { "" };
        let prt = if self.prt { "PRT " } else { "" };
        let cnf = if self.cnf { "CNF " } else { "" };
        let ntup = if self.ntup { "NTUP " } else { "" };
        let estc = if self.estc { "ESTC " } else { "" };
        let relr = if self.relr { "RELR " } else { "" };
        let gmco = if self.gmco { "GMCO " } else { "" };

        let string = String::new();

        let value = string
            + rel
            + prc
            + setc
            + ssap
            + gmps
            + manc
            + crtd
            + nmat
            + teco
            + macm
            + mspt
            + pprt
            + ncmp
            + clsd
            + pcnf
            + cser
            + prt
            + cnf
            + ntup
            + estc
            + relr
            + gmco;

        worksheet.write_string(row, col, value)
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
        format: &rust_xlsxwriter::Format,
    ) -> Result<&'a mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let rel = if self.rel { "REL " } else { "" };
        let prc = if self.prc { "PRC " } else { "" };
        let setc = if self.setc { "SETC " } else { "" };
        let ssap = if self.ssap { "SSAP " } else { "" };
        let gmps = if self.gmps { "GMPS " } else { "" };
        let manc = if self.manc { "MANC " } else { "" };
        let crtd = if self.crtd { "CRTD " } else { "" };
        let nmat = if self.nmat { "NMAT " } else { "" };
        let teco = if self.teco { "TECO " } else { "" };
        let macm = if self.macm { "MACM " } else { "" };
        let mspt = if self.mspt { "MSPT " } else { "" };
        let pprt = if self.pprt { "PPRT " } else { "" };
        let ncmp = if self.ncmp { "NCMP " } else { "" };
        let clsd = if self.clsd { "CLSD " } else { "" };
        let pcnf = if self.pcnf { "PCNF " } else { "" };
        let cser = if self.cser { "CSER " } else { "" };
        let prt = if self.prt { "PRT " } else { "" };
        let cnf = if self.cnf { "CNF " } else { "" };
        let ntup = if self.ntup { "NTUP " } else { "" };
        let estc = if self.estc { "ESTC " } else { "" };
        let relr = if self.relr { "RELR " } else { "" };
        let gmco = if self.gmco { "GMCO " } else { "" };

        let string = String::new();

        let value = string
            + rel
            + prc
            + setc
            + ssap
            + gmps
            + manc
            + crtd
            + nmat
            + teco
            + macm
            + mspt
            + pprt
            + ncmp
            + clsd
            + pcnf
            + cser
            + prt
            + cnf
            + ntup
            + estc
            + relr
            + gmco;

        worksheet.write_string_with_format(row, col, value, format)
    }
}
impl IntoExcelData for UserStatusCodes {
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let appr = if self.appr { "APPR " } else { "" };
        let smat = if self.smat { "SMAT " } else { "" };
        let init = if self.init { "INIT " } else { "" };
        let rdbl = if self.rdbl { "RDBL " } else { "" };
        let qcap = if self.qcap { "QCAP " } else { "" };
        let rfrz = if self.rfrz { "RFRZ " } else { "" };
        let wmat = if self.wmat { "WMAT " } else { "" };
        let cmat = if self.cmat { "CMAT " } else { "" };
        let pmat = if self.pmat { "PMAT " } else { "" };
        let apog = if self.apog { "APOG " } else { "" };
        let prok = if self.prok { "PROK " } else { "" };
        let wrea = if self.wrea { "WREA " } else { "" };
        let exdo = if self.exdo { "EXDO " } else { "" };
        let swe = if self.swe { "SWE " } else { "" };
        let awdo = if self.awdo { "AWDO " } else { "" };
        let rout = if self.rout { "ROUT " } else { "" };
        let wta = if self.wta { "WTA " } else { "" };
        let sch = if self.sch { "SCH " } else { "" };
        let sece = if self.sece { "SECE " } else { "" };
        let rel = if self.rel { "REL " } else { "" };
        let rees = if self.rees { "REES " } else { "" };
        let reap = if self.reap { "REAP " } else { "" };
        let wrel = if self.wrel { "WREL " } else { "" };
        let awsd = if self.awsd { "AWSD " } else { "" };
        let sraa = if self.sraa { "SRAA " } else { "" };
        let qcrj = if self.qcrj { "QCRJ " } else { "" };
        let awsc = if self.awsc { "AWSC " } else { "" };
        let lprq = if self.lprq { "LPRQ " } else { "" };
        let rrev = if self.rrev { "RREV " } else { "" };
        let awca = if self.awca { "AWCA " } else { "" };
        let rreq = if self.rreq { "RREQ " } else { "" };
        let vfal = if self.vfal { "VFAL " } else { "" };
        let sreq = if self.sreq { "SREQ " } else { "" };
        let amcr = if self.amcr { "AMCR " } else { "" };
        let dfrj = if self.dfrj { "DFRJ " } else { "" };
        let vpas = if self.vpas { "VPAS " } else { "" };
        let dfcr = if self.dfcr { "DFCR " } else { "" };
        let ireq = if self.ireq { "IREQ " } else { "" };
        let atvd = if self.atvd { "ATVD " } else { "" };
        let awmd = if self.awmd { "AWMD " } else { "" };
        let dfex = if self.dfex { "DFEX " } else { "" };
        let dfap = if self.dfap { "DFAP " } else { "" };
        let awpr = if self.awpr { "AWPR " } else { "" };

        let string = String::new();

        let value = string
            + appr
            + smat
            + init
            + rdbl
            + qcap
            + rfrz
            + wmat
            + cmat
            + pmat
            + apog
            + prok
            + wrea
            + exdo
            + swe
            + awdo
            + rout
            + wta
            + sch
            + sece
            + rel
            + rees
            + reap
            + wrel
            + awsd
            + sraa
            + qcrj
            + awsc
            + lprq
            + rrev
            + awca
            + rreq
            + vfal
            + sreq
            + amcr
            + dfrj
            + vpas
            + dfcr
            + ireq
            + atvd
            + awmd
            + dfex
            + dfap
            + awpr;

        worksheet.write_string(row, col, value)
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
        format: &rust_xlsxwriter::Format,
    ) -> Result<&'a mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        let appr = if self.appr { "APPR " } else { "" };
        let smat = if self.smat { "SMAT " } else { "" };
        let init = if self.init { "INIT " } else { "" };
        let rdbl = if self.rdbl { "RDBL " } else { "" };
        let qcap = if self.qcap { "QCAP " } else { "" };
        let rfrz = if self.rfrz { "RFRZ " } else { "" };
        let wmat = if self.wmat { "WMAT " } else { "" };
        let cmat = if self.cmat { "CMAT " } else { "" };
        let pmat = if self.pmat { "PMAT " } else { "" };
        let apog = if self.apog { "APOG " } else { "" };
        let prok = if self.prok { "PROK " } else { "" };
        let wrea = if self.wrea { "WREA " } else { "" };
        let exdo = if self.exdo { "EXDO " } else { "" };
        let swe = if self.swe { "SWE " } else { "" };
        let awdo = if self.awdo { "AWDO " } else { "" };
        let rout = if self.rout { "ROUT " } else { "" };
        let wta = if self.wta { "WTA " } else { "" };
        let sch = if self.sch { "SCH " } else { "" };
        let sece = if self.sece { "SECE " } else { "" };
        let rel = if self.rel { "REL " } else { "" };
        let rees = if self.rees { "REES " } else { "" };
        let reap = if self.reap { "REAP " } else { "" };
        let wrel = if self.wrel { "WREL " } else { "" };
        let awsd = if self.awsd { "AWSD " } else { "" };
        let sraa = if self.sraa { "SRAA " } else { "" };
        let qcrj = if self.qcrj { "QCRJ " } else { "" };
        let awsc = if self.awsc { "AWSC " } else { "" };
        let lprq = if self.lprq { "LPRQ " } else { "" };
        let rrev = if self.rrev { "RREV " } else { "" };
        let awca = if self.awca { "AWCA " } else { "" };
        let rreq = if self.rreq { "RREQ " } else { "" };
        let vfal = if self.vfal { "VFAL " } else { "" };
        let sreq = if self.sreq { "SREQ " } else { "" };
        let amcr = if self.amcr { "AMCR " } else { "" };
        let dfrj = if self.dfrj { "DFRJ " } else { "" };
        let vpas = if self.vpas { "VPAS " } else { "" };
        let dfcr = if self.dfcr { "DFCR " } else { "" };
        let ireq = if self.ireq { "IREQ " } else { "" };
        let atvd = if self.atvd { "ATVD " } else { "" };
        let awmd = if self.awmd { "AWMD " } else { "" };
        let dfex = if self.dfex { "DFEX " } else { "" };
        let dfap = if self.dfap { "DFAP " } else { "" };
        let awpr = if self.awpr { "AWPR " } else { "" };

        let string = String::new();

        let value = string
            + appr
            + smat
            + init
            + rdbl
            + qcap
            + rfrz
            + wmat
            + cmat
            + pmat
            + apog
            + prok
            + wrea
            + exdo
            + swe
            + awdo
            + rout
            + wta
            + sch
            + sece
            + rel
            + rees
            + reap
            + wrel
            + awsd
            + sraa
            + qcrj
            + awsc
            + lprq
            + rrev
            + awca
            + rreq
            + vfal
            + sreq
            + amcr
            + dfrj
            + vpas
            + dfcr
            + ireq
            + atvd
            + awmd
            + dfex
            + dfap
            + awpr;

        worksheet.write_string_with_format(row, col, value, format)
    }
}

impl IntoExcelData for MaterialStatus {
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        worksheet.write_string(row, col, self.to_string())
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
        format: &rust_xlsxwriter::Format,
    ) -> Result<&'a mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError> {
        worksheet.write_string_with_format(row, col, self.to_string(), format)
    }
}
