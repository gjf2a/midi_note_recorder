use std::{collections::VecDeque, sync::Arc, time::{Duration, Instant}};

use crossbeam_queue::SegQueue;
use midi_msg::{MidiMsg, SystemRealTimeMsg};
use serde::{Deserialize, Serialize};

pub fn note_velocity_from(msg: &MidiMsg) -> Option<(u8, u8)> {
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

pub fn seconds_since(timestamp: Instant) -> f64 {
    Instant::now().duration_since(timestamp).as_secs_f64()
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct Recording {
    records: VecDeque<(f64, Vec<u8>)>,
}

impl Recording {
    pub fn record_loop<M, F: Fn(MidiMsg) -> M>(
        incoming: Arc<SegQueue<MidiMsg>>,
        outgoing: Arc<SegQueue<M>>,
        outgoing_func: F,
    ) -> Self {
        let mut result = Self::default();
        let mut timestamp_reference = Instant::now();
        let mut first_message_received = false;
        loop {
            if let Some(msg) = incoming.pop() {
                if note_velocity_from(&msg).is_some() {
                    if !first_message_received {
                        timestamp_reference = Instant::now();
                        first_message_received = true;
                    }
                    result
                        .records
                        .push_back((seconds_since(timestamp_reference), msg.to_midi()));
                } else if msg
                    == (MidiMsg::SystemRealTime {
                        msg: SystemRealTimeMsg::SystemReset,
                    })
                {
                    return result;
                }
                outgoing.push(outgoing_func(msg))
            }
        }
    }

    pub fn playback_loop<M, F: Fn(MidiMsg) -> M>(
        &self,
        seconds_between_loops: Option<f64>,
        outgoing: Arc<SegQueue<M>>,
        outgoing_func: F,
    ) {
        loop {
            let mut playback_queue = self.records.clone();
            let kickoff = Instant::now();

            while playback_queue.len() > 0 {
                let (goal, _) = playback_queue[0];
                if Instant::now().duration_since(kickoff).as_secs_f64() > goal {
                    let (_, pn) = playback_queue.pop_front().unwrap();
                    let (deserialized, _) = MidiMsg::from_midi(&pn).unwrap();
                    outgoing.push(outgoing_func(deserialized));
                }
            }

            match seconds_between_loops {
                None => break,
                Some(secs) => std::thread::sleep(Duration::from_secs_f64(secs))
            }
        }
    }
}
