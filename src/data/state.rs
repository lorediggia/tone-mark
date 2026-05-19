#[derive(PartialEq, Clone, Copy)]
pub enum TargetChannel {
    Panel,
    Ch1,
    Ch2,
    Ch3,
    Ch4,
}

impl TargetChannel {
    pub fn msb(self) -> u8 {
        match self {
            TargetChannel::Panel => 0x60,
            TargetChannel::Ch1 => 0x10,
            TargetChannel::Ch2 => 0x11,
            TargetChannel::Ch3 => 0x12,
            TargetChannel::Ch4 => 0x13,
        }
    }

    pub fn program_change(self) -> Option<u8> {
        match self {
            TargetChannel::Panel => None,
            TargetChannel::Ch1 => Some(0),
            TargetChannel::Ch2 => Some(1),
            TargetChannel::Ch3 => Some(2),
            TargetChannel::Ch4 => Some(3),
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            TargetChannel::Panel => "PANEL",
            TargetChannel::Ch1 => "CH1",
            TargetChannel::Ch2 => "CH2",
            TargetChannel::Ch3 => "CH3",
            TargetChannel::Ch4 => "CH4",
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum AppTab {
    Library,
    Editor,
    Settings,
}

#[derive(Clone, Copy)]
pub struct AmpState {
    pub type_idx: usize,
    pub gain: u8,
    pub bass: u8,
    pub mid: u8,
    pub treble: u8,
    pub pres: u8,
    pub vol: u8,
    pub bright: bool,
    pub sag: u8,
    pub res: u8,
}

impl Default for AmpState {
    fn default() -> Self {
        Self {
            type_idx: 1,
            gain: 60,
            bass: 60,
            mid: 60,
            treble: 60,
            pres: 60,
            vol: 70,
            bright: false,
            sag: 60,
            res: 60,
        }
    }
}

#[derive(Clone, Copy)]
pub struct FxBlock {
    pub on: bool,
    pub type_idx: usize,
    pub p1: u8,
    pub p2: u8,
    pub p3: u8,
    pub p4: u8,
}

impl FxBlock {
    pub fn new(on: bool, type_idx: usize) -> Self {
        Self {
            on,
            type_idx,
            p1: 50,
            p2: 50,
            p3: 50,
            p4: 60,
        }
    }
}

#[derive(Clone, Copy)]
pub struct DelayState {
    pub on: bool,
    pub type_idx: usize,
    pub time: u8,
    pub feedback: u8,
    pub level: u8,
    pub e_level: u8,
}

impl Default for DelayState {
    fn default() -> Self {
        Self {
            on: false,
            type_idx: 0,
            time: 50,
            feedback: 35,
            level: 50,
            e_level: 50,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ReverbState {
    pub on: bool,
    pub type_idx: usize,
    pub time: u8,
    pub pre: u8,
    pub density: u8,
    pub level: u8,
}

impl Default for ReverbState {
    fn default() -> Self {
        Self {
            on: false,
            type_idx: 1,
            time: 50,
            pre: 20,
            density: 60,
            level: 50,
        }
    }
}

#[derive(Clone, Copy)]
pub struct NsState {
    pub on: bool,
    pub threshold: u8,
    pub release: u8,
}

impl Default for NsState {
    fn default() -> Self {
        Self {
            on: false,
            threshold: 30,
            release: 40,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Snapshot {
    pub amp: AmpState,
    pub booster: FxBlock,
    pub mod_fx: FxBlock,
    pub fx: FxBlock,
    pub delay: DelayState,
    pub reverb: ReverbState,
    pub ns: NsState,
}

pub struct PatchInfo {
    pub name: String,
    pub memo: String,
    pub block_count: usize,
}
