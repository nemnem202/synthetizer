use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventType {
    NoteOff = 0,
    NoteOn = 1,
}

impl TryFrom<u8> for EventType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EventType::NoteOff),
            1 => Ok(EventType::NoteOn),
            _ => Err("Valeur d'événement MIDI inconnue"),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct NoteDTO {
    pub value: u8,
    pub velocity: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct Mix {
    pub dry: f32,
    pub wet: f32,
}

pub struct FxEventDto {
    pub id: u32,
    pub event_type: u32,
    pub params: Vec<f32>,
}

#[derive(Default)]
pub struct SampleEvent {
    pub sample_event_index: u32,
    pub sampler_id: u32,
    pub sample_id: u32,
    pub length: u32,
    pub channels: u8,
    pub hq: u8,
}

pub struct Sample {
    pub id: u32,
    pub values: Box<[f32]>,
    pub hq: u8,
}
