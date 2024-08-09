use std::{collections::VecDeque, sync::Arc, time::Instant};

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

#[derive(Copy, Clone, PartialEq)]
pub struct PendingNote {
    note: u8,
    velocity: u8,
    timestamp: f64,
    start: Instant,
}

impl PendingNote {
    pub fn new(note: u8, velocity: u8, timestamp_reference: Instant) -> Self {
        let start = Instant::now();
        Self {
            note,
            velocity,
            timestamp: start.duration_since(timestamp_reference).as_secs_f64(),
            start,
        }
    }

    pub fn finished_playing(&self) -> PlayedNote {
        PlayedNote {
            note: self.note,
            velocity: self.velocity,
            duration: Instant::now().duration_since(self.start).as_secs_f64(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Recording {
    notes: VecDeque<(f64, PlayedNote)>
}

impl Recording {
    pub fn empty(&self) -> bool {
        self.notes.len() == 0
    }

    pub fn record_loop(incoming: Arc<SegQueue<MidiMsg>>, timeout: f64) -> Self {
        let mut num_pending = 0;
        let mut pending_notes: [Option<PendingNote>; 128] = [None; 128];
        let mut result = Self::default();
        let timestamp_reference = Instant::now();
        let mut most_recent = None;
        let mut base_time = 0.0;
        loop {
            if let Some(msg) = incoming.pop() {
                if let Some((note, velocity)) = note_velocity_from(&msg) {
                    if most_recent.is_none() {
                        base_time = Instant::now().duration_since(timestamp_reference).as_secs_f64();
                    }
                    if let Some(pending) = pending_notes[note as usize] {
                        result
                            .notes
                            .push_back((pending.timestamp - base_time, pending.finished_playing()));
                        num_pending -= 1;
                    }
                    pending_notes[note as usize] = if velocity > 0 {
                        num_pending += 1;
                        Some(PendingNote::new(note, velocity, timestamp_reference))
                    } else {
                        None
                    };
                    most_recent = Some(Instant::now());
                }
            }
            if let Some(recent) = most_recent {
                if num_pending == 0 && Instant::now().duration_since(recent).as_secs_f64() > timeout
                {
                    return result;
                }
            }
        }
    }

    pub fn playback_loop(&self, outgoing: Arc<SegQueue<PlayedNote>>) {
        let mut playback_queue = self.notes.clone();
        let kickoff = Instant::now();
        while playback_queue.len() > 0 {
            let (goal, _) = playback_queue[0];
            if Instant::now().duration_since(kickoff).as_secs_f64() > goal {
                let (_, pn) = playback_queue.pop_front().unwrap();
                outgoing.push(pn);
            }
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct PlayedNote {
    note: u8,
    velocity: u8,
    duration: f64,
}
