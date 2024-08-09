use std::{sync::Arc, time::SystemTime};

use crossbeam_queue::SegQueue;
use midi_msg::MidiMsg;

fn note_velocity_from(msg: &MidiMsg) -> Option<(u8, u8)> {
    if let MidiMsg::ChannelVoice { channel: _, msg } = msg {
        match msg {
            midi_msg::ChannelVoiceMsg::NoteOn { note, velocity }
            | midi_msg::ChannelVoiceMsg::NoteOff { note, velocity } => Some((*note, *velocity)),
            _ => None,
        }
    } else {
        None
    }
}

pub struct PendingNote {
    
}

#[derive(Default, Debug, Clone)]
pub struct Recording {
    notes: Vec<PlayedNote>
}

impl Recording {
    pub fn empty(&self) -> bool {
        self.notes.len() == 0
    }

    pub fn record_loop(incoming: Arc<SegQueue<MidiMsg>>, timeout: f64) -> Self {
        let mut result = Self::default();
        let start = SystemTime::now();
        loop {
            if let Some(msg) = incoming.pop() {
                if let Some((note, velocity)) = note_velocity_from(&msg) {

                }
            }
        }
        result
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct PlayedNote {
    note: u8,
    velocity: u8,
    duration: f64
}

impl PlayedNote {
    pub fn new(note: u8, velocity: u8, duration: f64) -> Self {
        Self {note, velocity, duration}
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
