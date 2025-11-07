use std::collections::HashMap;
use std::error::Error;

use osc_lib::{OscArg, OscMessage};

pub static OFF_ON: &[&str] = &[" OFF", " ON"];
pub static XAMXGRP: &[&str] = &[" OFF", " X", " Y"];
pub static XCOLORS: &[&str] = &[
    " OFF", " RD", " GN", " YE", " BL", " MG", " CY", " WH", " OFFi", " RDi", " GNi", " YEi",
    " BLi", " MGi", " CYi", " WHi",
];
pub static XMNMODE: &[&str] = &[" LR+M", " LCR"];
pub static XCHMODE: &[&str] = &[" PFL", " AFL"];
pub static XHSLP: &[&str] = &[" 12", " 18", " 24"];
pub static XGMODE: &[&str] = &[" EXP2", " EXP3", " EXP", " GATE", " DUCK"];
pub static XDYMODE: &[&str] = &[" COMP", " EXP"];
pub static XDYDET: &[&str] = &[" PEAK", " RMS"];
pub static XDYENV: &[&str] = &[" LIN", " LOG"];
pub static XDYRAT: &[&str] = &[
    " 1.1", " 1.3", " 1.5", " 2.0", " 2.5", " 3.0", " 4.0", " 5.0", " 7.0", " 10", " 20", " 100",
];
pub static XDYFTYP: &[&str] = &[" LC6", " LC12", " HC6", " HC12", " 1.0", " 2.0", " 3.0", " 5.0", " 10.0"];
pub static XDYPPOS: &[&str] = &[" PRE", " POST"];
pub static XISEL: &[&str] = &[
    " OFF", " FX1L", " FX1R", " FX2L", " FX2R", " FX3L", " FX3R", " FX4L", " FX4R", " FX5L",
    " FX5R", " FX6L", " FX6R", " FX7L", " FX7R", " FX8L", " FX8R", " AUX1", " AUX2", " AUX3",
    " AUX4", " AUX5", " AUX6",
];
pub static XEQTY1: &[&str] = &[" LCut", " LShv", " PEQ", " VEQ", " HShv", " HCut"];
pub static XEQTY2: &[&str] = &[
    " LCut", " LShv", " PEQ", " VEQ", " HShv", " HCut", " BU6", " BU12", " BS12", " LR12",
    " BU18", " BU24", " BS24", " LR24",
];
pub static XMTYPE: &[&str] = &[" IN/LC", " <-EQ", " EQ->", " PRE", " POST", " GRP"];
pub static XTSOURC: &[&str] = &[" INT", " EXT"];
pub static XOSCSEL: &[&str] = &[" F1", " F2"];
pub static XOSCTYP: &[&str] = &[" SINE", " PINK", " WHITE"];
pub static XCFRSW: &[&str] = &[" REC", " PLAY"];
pub static XRTGIN: &[&str] = &[
    " AN1-8", " AN9-16", " AN17-24", " AN25-32", " A1-8", " A9-16", " A17-24", " A25-32",
    " A33-40", " A41-48", " B1-8", " B9-16", " B17-24", " B25-32", " B33-40", " B41-48",
    " CARD1-8", " CARD9-16", " CARD17-24", " CARD25-32",
];
pub static XRTAEA: &[&str] = &[
    " AN1-8", " AN9-16", "AN17-24", " AN25-32", " A1-8", " A9-16", " A17-24", " A25-32",
    " A33-40", " A41-48", " B1-8", " B9-16", " B17-24", " B25-32", " B33-40", " B41-48",
    " CARD1-8", " CARD9-16", " CARD17-24", " CARD25-32", " OUT1-8", " OUT9-16", " P161-8",
    " P169-16", " AUX1-6/Mon", " AuxIN1-6/TB",
];
pub static XRTINA: &[&str] = &[
    " AUX1-4", " AN1-2", " AN1-4", " AN1-6", " A1-2", " A1-4", " A1-6", " B1-2", " B1-4",
    " B1-6", " CARD1-2", " CARD1-4", " CARD1-6",
];
pub static XROUT1: &[&str] = &[
    " AN1-4", " AN9-12", " AN17-20", " AN25-28", " A1-4", " A9-12", " A17-20", " A25-28",
    " A33-36", " A41-44", " B1-4", " B9-12", " B17-20", " B25-28", " B33-36", " B41-44",
    " CARD1-4", " CARD9-12", " CARD17-20", " CARD25-28", " OUT1-4", " OUT9-12", " P161-4",
    " P169-12", " AUX/CR", " AUX/TB",
];
pub static XROUT5: &[&str] = &[
    " AN5-8", " AN13-16", " AN21-24", " AN29-32", " A5-8", " A13-16", " A21-24", " A29-32",
    " A37-40", " A45-48", " B5-8", " B13-16", " B21-24", " B29-32", " B37-40", " B45-48",
    " CARD5-8", " CARD13-16", " CARD21-24", " CARD29-32", " OUT5-8", " OUT13-16", " P165-8",
    " P1613-16", " AUX/CR", " AUX/TB",
];

pub static SFXTYP1: &[&str] = &[
    " HALL", " AMBI", " RPLT", " ROOM", " CHAM", " PLAT", " VREV", " VRM", " GATE", " RVRS",
    " DLY", " 3TAP", " 4TAP", " CRS", " FLNG", " PHAS", " DIMC", " FILT", " ROTA", " PAN",
    " SUB", " D/RV", " CR/R", " FL/R", " D/CR", " D/FL", " MODD", " GEQ2", " TEQ2", " GEQ",
    " TEQ", " DES2", " DES", " P1A2", " P1A", " PQ5S", " PQ5", " WAVD", " LIM", " CMB2",
    " CMB", " FAC2", " FAC1M", " FAC", " LEC2", " LEC", " ULC2", " ULC", " ENH2", " ENH",
    " EXC2", " EXC", " IMG", " EDI", " SON", " AMP2", " AMP", " DRV2", " DRV", " PIT2", " PIT",
];

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SfxTyp1 {
    Hall, Ambi, Rplt, Room, Cham, Plat, Vrev, Vrm, Gate, Rvrs, Dly, Tap3, Tap4, Crs, Flng,
    Phas, Dimc, Filt, Rota, Pan, Sub, DRv, CrR, FlR, DCr, DFl, Modd, Geq2, Teq2, Geq, Teq,
    Des2, Des, P1a2, P1a, Pq5s, Pq5, Wavd, Lim, Cmb2, Cmb, Fac2, Fac1m, Fac, Lec2, Lec,
    Ulc2, Ulc, Enh2, Enh, Exc2, Exc, Img, Edi, Son, Amp2, Amp, Drv2, Drv, Pit2, Pit,
}

pub static SFXTYP2: &[&str] = &[
    " GEQ2", " TEQ2", " GEQ", " TEQ", " DES2", " DES", " P1A", " P1A2", " PQ5", " PQ5S",
    " WAVD", " LIM", " FAC", " FAC1M", " FAC2", " LEC", " LEC2", " ULC", " ULC2", " ENH2",
    " ENH", " EXC2", " EXC", " IMG", " EDI", " SON", " AMP2", " AMP", " DRV2", " DRV", " PHAS",
    " FILT", " PAN", " SUB",
];

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SfxTyp2 {
    Geq2, Teq2, Geq, Teq, Des2, Des, P1a, P1a2, Pq5, Pq5s, Wavd, Lim, Fac, Fac1m, Fac2, Lec,
    Lec2, Ulc, Ulc2, Enh2, Enh, Exc2, Exc, Img, Edi, Son, Amp2, Amp, Drv2, Drv, Phas, Filt,
    Pan, Sub,
}

pub static SFXSRC: &[&str] = &[
    " INS", " MIX1", " MIX2", " MIX3", " MIX4", " MIX5", " MIX6", " MIX7", " MIX8", " MIX9",
    " MIX10", " MIX11", " MIX12", " MIX13", " MIX14", " MIX15", " MIX16", " M/C",
];
pub static XOTPOS: &[&str] = &[
    "IN/LC", "IN/LC+M", "<-EQ", "<-EQ+M", "EQ->", "EQ->+M", "PRE", "PRE+M", "POST",
];
pub static XIQGRP: &[&str] = &[" OFF", " A", " B"];
pub static XIQSPK: &[&str] = &[
    " none", " iQ8", " iQ10", " iQ12", " iQ15", " iQ15B", " iQ18B",
];
pub static XIQEQ: &[&str] = &[" Linear", " Live", " Speech", " Playback", " User"];
pub static PSOURCE: &[&str] = &[" INT", " AES50A", " AES50B", " Exp Card"];
pub static PSCONT: &[&str] = &[" CUES", " SCENES", " SNIPPETS"];
pub static PRPRO: &[&str] = &[" MC", " HUI", " CC"];
pub static PRRATE: &[&str] = &[" 48K", " 44K1"];
pub static PRPOS: &[&str] = &[" PRE", " POST"];
pub static PRMODE: &[&str] = &[" BAR", " SPEC"];
pub static PRDET: &[&str] = &[" RMS", " PEAK"];
pub static PRPORT: &[&str] = &[" MIDI", " CARD", " RTP"];
pub static PCTYPE: &[&str] = &[" FW", " USB", " unk", " unk", " unk", " unk"];
pub static PUSBMOD: &[&str] = &[" 32/32", " 16/16", " 32/8", " 8/32", " 8/8", " 2/2"];
pub static PUFMODE: &[&str] = &[" 32/32", " 16/16", " 32/8", " 8/32"];
pub static PCAW: &[&str] = &[" IN", " OUT"];
pub static PCAS: &[&str] = &[" WC", " ADAT1", " ADAT2", " ADAT3", " ADAT4"];
pub static PMDMODE: &[&str] = &[" 56", " 64"];
pub static PCMADI: &[&str] = &[" 1-32", " 9-40", " 17-48", " 25-56", " 33-64"];
pub static PCMADO: &[&str] = &[" OFF", "1-32", " 9-40", " 17-48", " 25-56", " 33-64"];
pub static PMADSRC: &[&str] = &[" OFF", " OPT", " COAX", " BOTH"];
pub static PURECTK: &[&str] = &[" 32Ch", " 16Ch", " 8Ch"];
pub static PURPLBK: &[&str] = &[" SD", " USB"];
pub static PURSDSL: &[&str] = &[" SD1", " SD2"];
pub static PURRCTL: &[&str] = &[" USB", " XLIVE"];
pub static PINVMUT: &[&str] = &[" NORM", " INV"];
pub static PCLKMOD: &[&str] = &[" 24h", " 12h"];
pub static PURERPA: &[&str] = &[" REC", " PLAY", " AUTO"];
pub static PRTAVIS: &[&str] = &[
    " OFF", " 25%", " 30%", " 35%", " 40%", "45%", "50%", " 55%", " 60%", "65%", "70%", "75%",
    "80%",
];
pub static PRTAPH: &[&str] = &[" OFF", " 1", " 2", " 3", " 4", "5", "6", " 7", "8"];
pub static UBAT: &[&str] = &[" NONE", " GOOD", " LOW"];
pub static USDC: &[&str] = &[" NONE", " READY", " PROTECT", " ERROR"];
pub static SSELIDX: &[&str] = &[
    " Ch01", " Ch02", " Ch03", " Ch04", " Ch05", " Ch06", " Ch07", " Ch08", " Ch09", " Ch10",
    " Ch11", " Ch12", " Ch13", " Ch14", " Ch15", " Ch16", " Ch17", " Ch18", " Ch19", " Ch20",
    " Ch21", " Ch22", " Ch23", " Ch24", " Ch25", " Ch26", " Ch27", " Ch28", " Ch29", " Ch30",
    " Ch31", " Ch32", " Aux1", " Aux2", " Aux3", " Aux4", " Aux5", " Aux6", " Aux7", " Aux8",
    " Fx1L", " Fx1R", " Fx2L", " Fx2R", " Fx3L", " Fx3R", " Fx4L", " Fx4R", " Bus1", " Bus2",
    " Bus3", " Bus4", " Bus5", " Bus6", " Bus7", " Bus8", " Bus9", " Bs10", " Bs11", " Bs12",
    " Bs13", " Bs14", " Bs15", " Bs16", " Mtx1", " Mtx2", " Mtx3", " Mtx4", " Mtx5", " Mtx6",
    " LR", " M/C",
];
pub static SSCRN: &[&str] = &[
    " CHAN", " METERS", " ROUTE", " SETUP", " LIB", " FX", " MON", " USB", " SCENE", " ASSIGN",
    " LOCK",
];
pub static SCHAL: &[&str] = &[" HOME", " CONFIG", " GATE", " DYN", " EQ", "MIX", " MAIN"];
pub static SMETL: &[&str] = &[" CHANNEL", " MIXBUS", " AUX/FX", " IN/OUT", " RTA"];
pub static SROUL: &[&str] = &[
    " HOME", " AES50A", " AES50B", " CARDOUT", "XLROUT", " ANAOUT", "AUXOUT", "P16OUT",
    "USER",
];
pub static SSETL: &[&str] = &[
    " GLOB", " CONF", " REMOTE", " NETW", "NAMES", "PREAMPS", " CARD",
];
pub static SLIBL: &[&str] = &[" CHAN", " EFFECT", " ROUTE"];
pub static SFXL: &[&str] = &[
    " HOME", " FX1", " FX2", " FX3", "FX4", "FX5", " FX6", " FX7", " FX8",
];
pub static SMONL: &[&str] = &[" MONITOR", " TALKA", " TALKB", " OSC"];
pub static SUSBL: &[&str] = &[" HOME", " CONFIG"];
pub static SSCEL: &[&str] = &[
    " HOME", " SCENES", " BITS", " PARSAFE", "CHNSAFE", "MIDI",
];
pub static SASSL: &[&str] = &[" HOME", " SETA", " SETB", " SETC"];
pub static STAPL: &[&str] = &[" STOP", " PPAUSE", " PLAY", " RPAUSE", "RECORD", "FF", "REW"];

pub static R00: &[&str] = &["OFF", "ON"];
pub static R01: &[&str] = &["FRONT", "REAR"];
pub static R02: &[&str] = &["ST", "M/S"];
pub static R03: &[&str] = &["2", "8", "12", "20", "ALL"];
pub static R04: &[&str] = &["COMP", "LIM"];
pub static R05: &[&str] = &["GR", "SBC", "PEAK"];
pub static R06: &[&str] = &["0", "1"];
pub static R07: &[&str] = &["OFF", "Bd1", "Bd2", "Bd3", "Bd4", "Bd5"];
pub static R08: &[&str] = &["12", "48"];
pub static R09: &[&str] = &[
    "1.1", "1.2", "1.3", "1.5", "1.7", "2", "2.5", "3", "3.5", "4", "5", "7", "10", "LIM",
];
pub static R10: &[&str] = &["1k5", "2k", "3k", "4k", "5k"];
pub static R11: &[&str] = &["200", "300", "500", "700", "1k", "1k5", "2k", "3k", "4k", "5k", "7k"];
pub static R12: &[&str] = &["200", "300", "500", "700", "1000"];
pub static R13: &[&str] = &["5k", "10k", "20k"];
pub static R14: &[&str] = &["3k", "4k", "5k", "8k", "10k", "12k", "16k"];
pub static R15: &[&str] = &["20", "30", "60", "100"];
pub static R16: &[&str] = &["FEM", "MALE"];
pub static R17: &[&str] = &["AMB", "CLUB", "HALL"];
pub static R18: &[&str] = &["PAR", "SER"];
pub static R19: &[&str] = &["1", "1/2", "2/3", "3/2"];
pub static R20: &[&str] = &[
    "1/4", "1/3", "3/8", "1/2", "2/3", "3/4", "1", "1/4X", "1/3X", "3/8X", "1/2X", "2/3X",
    "3/4X", "1X",
];
pub static R21: &[&str] = &["LO", "MID", "HI"];
pub static R22: &[&str] = &["TRI", "SIN", "SAW", "SAW-", "RMP", "SQU", "RND"];
pub static R23: &[&str] = &["LP", "HP", "BP", "NO"];
pub static R24: &[&str] = &["M", "ST"];
pub static R25: &[&str] = &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"];
pub static R26: &[&str] = &["2POL", "4POL"];
pub static R27: &[&str] = &["RUN", "STOP"];
pub static R28: &[&str] = &["SLOW", "FAST"];
pub static R29: &[&str] = &["ST", "MS"];

// Simplified version of the `enum types` from X32.c
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CommandType {
    Nil,
    I32,
    F32,
    S32,
    B32,
    E32,
    P32,
    Fx32,
    OffOn,
    CMono,
    CSolo,
    CTalk,
    CTalkAB,
    COsc,
    CRoutSw,
    CRoutIn,
    CRoutAc,
    CRoutOt,
    CRoutPlay,
    CCtrl,
    CEnc,
    CTape,
    CMix,
    ChCo,
    ChDe,
    ChPr,
    ChGa,
    ChGf,
    ChDy,
    ChDf,
    ChIn,
    ChEq,
    ChMx,
    ChMo,
    ChMe,
    ChGrp,
    ChAmix,
    AxPr,
    BsCo,
    MxPr,
    MxDy,
    MsMx,
    FxTyp1,
    FxSrc,
    FxPar1,
    FxTyp2,
    FxPar2,
    OMain,
    OMain2,
    OP16,
    OMainD,
    HAmp,
    Prefs,
    PIr,
    PIq,
    PCard,
    PRta,
    PIp,
    PAddr,
    PMask,
    PGway,
    Stat,
    SScreen,
    SCha,
    SMet,
    SRou,
    SSet,
    SLib,
    SFx,
    SMon,
    SUsb,
    SSce,
    SAss,
    SSoloSw,
    SAes,
    STape,
    SOsc,
    STalk,
    Usb,
    SNam,
    SCue,
    SScn,
    SSnp,
    Ha,
    Action,
    URec,
    SLibs,
    D48,
    D48A,
    D48G,
    URouO,
    URouI,
    PKey,
}

// Represents the `union value` from the C struct
#[derive(Debug, PartialEq, Clone)]
pub enum CommandValue {
    Int(i32),
    Float(f32),
    String(String),
    Blob(Vec<u8>),
    None,
}

// Represents the `union format` from the C struct
#[derive(Debug, PartialEq, Clone)]
pub enum CommandFormat {
    Simple(CommandType),
    Composite(String), // For format strings like ",sfi"
}

// Represents the `X32command` struct from X32.c
#[derive(Debug, Clone)]
pub struct X32Command {
    pub command: String,
    pub format: CommandFormat,
    pub flags: u32,
    pub value: CommandValue,
    pub node: Option<Vec<&'static str>>,
}

#[derive(Debug, Clone)]
pub struct MixerState {
    values: HashMap<String, OscArg>,
}

impl MixerState {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn set(&mut self, path: &str, arg: OscArg) {
        self.values.insert(path.to_string(), arg);
    }

    pub fn get(&self, path: &str) -> Option<&OscArg> {
        self.values.get(path)
    }
}

pub struct Mixer {
    state: MixerState,
}

impl Mixer {
    pub fn new() -> Self {
        Self {
            state: MixerState::new(),
        }
    }

    pub fn seed_from_lines(&mut self, lines: Vec<&str>) {
        for line in lines {
            let parts: Vec<&str> = line.splitn(2, ',').collect();
            if parts.len() == 2 {
                let path = parts[0].trim();
                let arg_parts: Vec<&str> = parts[1].trim().splitn(2, '\t').collect();
                if arg_parts.len() == 2 {
                    let arg_type = arg_parts[0];
                    let arg_value = arg_parts[1];
                    let arg = match arg_type {
                        "i" => OscArg::Int(arg_value.parse().unwrap()),
                        "f" => OscArg::Float(arg_value.parse().unwrap()),
                        "s" => OscArg::String(arg_value.to_string()),
                        _ => continue,
                    };
                    self.state.set(path, arg);
                }
            }
        }
    }

    pub fn dispatch(&mut self, msg: &[u8]) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        let osc_msg = OscMessage::from_bytes(msg)?;

        if osc_msg.path == "/info" {
            let response = OscMessage {
                path: "/info".to_string(),
                args: vec![
                    OscArg::String("V2.07".to_string()),
                    OscArg::String("X32 Emulator".to_string()),
                    OscArg::String("X32".to_string()),
                    OscArg::String("4.06".to_string()),
                ],
            };
            return Ok(Some(response.to_bytes()?));
        }

        if osc_msg.args.is_empty() {
            // It's a request for a value
            if let Some(arg) = self.state.get(&osc_msg.path) {
                let response = OscMessage {
                    path: osc_msg.path.clone(),
                    args: vec![arg.clone()],
                };
                return Ok(Some(response.to_bytes()?));
            }
        } else {
            // It's a command to set a value
            if let Some(arg) = osc_msg.args.get(0) {
                self.state.set(&osc_msg.path, arg.clone());
            }
        }

        Ok(None)
    }
}
