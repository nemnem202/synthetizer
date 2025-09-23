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
pub enum WaveType {
    Sine,
    Square,
    Saw,
    Triangle,
}

impl TryFrom<u8> for WaveType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WaveType::Sine),
            1 => Ok(WaveType::Square),
            2 => Ok(WaveType::Saw),
            3 => Ok(WaveType::Triangle),
            _ => Err("Valeur de WaveType invalide"),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct NoteDTO {
    pub value: u8,
    pub velocity: u8,
}
