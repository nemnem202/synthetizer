use crate::{
    dsp_effects::{Echo, EchoParams},
    toolkit::ToolKit,
    types::Mix,
};

pub const FLAG_INDEX: u32 = 0;
pub const READ_INDEX: u32 = 1;
pub const WRITE_INDEX: u32 = 2;
pub const HEADERS_SIZE_BYTES: u32 = 3 * 4;

pub const MIDI_EVENT_SIZE: u32 = 4;
pub const MIDI_QUEUE_CAPACITY: u32 = 64;
pub const MIDI_WRITE_INDEX: u32 = 0;
pub const MIDI_READ_INDEX: u32 = 1;

pub const FX_QUEUE_CAPACITY: u32 = 64;
pub const FX_EVENT_SIZE_INT: u32 = 3;
pub const FX_EVENT_SIZE_FLOAT: u32 = 1;
pub const FX_WRITE_INDEX: u32 = 0;
pub const FX_READ_INDEX: u32 = 1;

pub const SAMPLE_RATE: f32 = 44100.0;
pub const FREQ_A4: f32 = 440.0;

pub const OSC_QUEUE_CAPACITY: u32 = 100;
