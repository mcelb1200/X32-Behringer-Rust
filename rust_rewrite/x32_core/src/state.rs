use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Channel {
    pub config: Config,
    pub grp: Group,
    pub preamp: Preamp,
    pub delay: Delay,
    pub insert: Insert,
    pub gate: Gate,
    pub dynamics: Dynamics,
    pub eq: Eq,
    pub mix: Mix,
    pub automix: Automix,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config {
    pub name: String,
    pub icon: i32,
    pub color: i32,
    pub source: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Group {
    pub dca: i32,
    pub mute: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Preamp {
    pub trim: f32,
    pub invert: i32,
    pub hpon: i32,
    pub hpslope: i32,
    pub hpf: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Delay {
    pub on: i32,
    pub time: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Insert {
    pub on: i32,
    pub pos: i32,
    pub sel: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Gate {
    pub on: i32,
    pub mode: i32,
    pub thr: f32,
    pub range: f32,
    pub attack: f32,
    pub hold: f32,
    pub release: f32,
    pub keysrc: i32,
    pub filter: Filter,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Dynamics {
    pub on: i32,
    pub mode: i32,
    pub det: i32,
    pub env: i32,
    pub thr: f32,
    pub ratio: i32,
    pub knee: f32,
    pub mgain: f32,
    pub attack: f32,
    pub hold: f32,
    pub release: f32,
    pub pos: i32,
    pub keysrc: i32,
    pub mix: f32,
    pub auto: i32,
    pub filter: Filter,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Filter {
    pub on: i32,
    pub filter_type: i32,
    pub f: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Eq {
    pub on: i32,
    pub bands: [EqBand; 4],
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EqBand {
    pub eq_type: i32,
    pub f: f32,
    pub g: f32,
    pub q: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Mix {
    pub on: i32,
    pub fader: f32,
    pub st: i32,
    pub pan: f32,
    pub mono: i32,
    pub mlevel: f32,
    pub sends: [MixSend; 16],
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MixSend {
    pub on: i32,
    pub level: f32,
    pub pan: f32,
    pub send_type: i32,
    pub pan_follow: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Automix {
    pub group: i32,
    pub weight: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Preferences {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Save {
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LinkConfig {
    pub ch: [bool; 16],
    pub aux: [bool; 4],
    pub fx: [bool; 4],
    pub bus: [bool; 8],
    pub mtx: [bool; 3],
    pub hadly: bool,
    pub eq: bool,
    pub dyn: bool,
    pub fdrmute: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MuteGroup {
    pub on: [bool; 6],
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SoloConfig {
    pub level: f32,
    pub source: i32,
    pub sourcetrim: f32,
    pub chmode: i32,
    pub busmode: i32,
    pub dcamode: i32,
    pub exclusive: bool,
    pub followsel: bool,
    pub followsolo: bool,
    pub dimatt: f32,
    pub dim: bool,
    pub mono: bool,
    pub delay: bool,
    pub delaytime: f32,
    pub masterctrl: bool,
    pub mute: bool,
    pub dimpfl: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TalkbackConfig {
    pub enable: bool,
    pub source: i32,
    pub a: Talkback,
    pub b: Talkback,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Talkback {
    pub level: f32,
    pub dim: bool,
    pub latch: bool,
    pub destmap: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OscConfig {
    pub level: f32,
    pub f1: f32,
    pub f2: f32,
    pub fsel: i32,
    pub osc_type: i32,
    pub dest: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserRouting {
    pub input: [i32; 32],
    pub output: [i32; 48],
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Routing {
    pub routswitch: i32,
    pub input: [i32; 5],
    pub aes50a: [i32; 6],
    pub aes50b: [i32; 6],
    pub card: [i32; 4],
    pub output: [i32; 4],
    pub play: [i32; 5],
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserCtrl {
    pub color: i32,
    pub enc: [String; 4],
    pub btn: [String; 8],
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TapeConfig {
    pub gain_l: f32,
    pub gain_r: f32,
    pub autoplay: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AutomixConfig {
    pub x: bool,
    pub y: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Dp48Config {
    pub scope: i32,
    pub broadcast: i32,
    pub assign: [i32; 48],
    pub grpname: [String; 12],
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MainBus {
    pub config: Config,
    pub dyn: Dynamics,
    pub insert: Insert,
    pub eq: Eq,
    pub mix: Mix,
    pub grp: Group,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MixerState {
    pub channels: [Channel; 32],
    pub auxin: [Channel; 8],
    pub bus: [MainBus; 16],
    pub mtx: [MainBus; 6],
    pub main: MainBus,
    pub mono: MainBus,
    pub link_config: LinkConfig,
    pub mute_group: MuteGroup,
    pub solo_config: SoloConfig,
    pub talkback_config: TalkbackConfig,
    pub osc_config: OscConfig,
    pub user_routing: UserRouting,
    pub routing: Routing,
    pub user_ctrl: [UserCtrl; 3],
    pub tape_config: TapeConfig,
    pub automix_config: AutomixConfig,
    pub dp48_config: Dp48Config,
    pub preferences: Preferences,
    pub scenes: Vec<Save>,
    pub snippets: Vec<Save>,
    pub channel_presets: Vec<Save>,
    pub fx_presets: Vec<Save>,
    pub routing_presets: Vec<Save>,
}
