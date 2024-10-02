use std::{
    collections::VecDeque,
    fs::File,
    io::Read,
    sync::Arc,
    time::{Duration, Instant},
};

use crossbeam_queue::SegQueue;
use midi_msg::{Channel, MidiMsg, SystemRealTimeMsg};
use serde::{Deserialize, Serialize};
use std::io::Write;

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

pub fn midi_msg_from(channel: Channel, note: u8, velocity: u8) -> MidiMsg {
    MidiMsg::ChannelVoice {
        channel,
        msg: if velocity == 0 {
            midi_msg::ChannelVoiceMsg::NoteOff { note, velocity }
        } else {
            midi_msg::ChannelVoiceMsg::NoteOn { note, velocity }
        },
    }
}

pub fn is_system_reset(msg: &MidiMsg) -> bool {
    *msg == (MidiMsg::SystemRealTime {
        msg: SystemRealTimeMsg::SystemReset,
    })
}

pub fn seconds_since(timestamp: Instant) -> f64 {
    Instant::now().duration_since(timestamp).as_secs_f64()
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct Recording {
    records: Vec<(f64, Vec<u8>)>,
}

impl Recording {
    pub fn from_file(filename: &str) -> anyhow::Result<Self> {
        Self::from_string(read_file_to_string(filename)?.as_str())
    }

    pub fn from_string(s: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(s)?)
    }

    pub fn from_sequence<S: AsRef<[(f64, MidiMsg)]>>(seq: &S) -> Self {
        let mut result = Self::default();
        for (time, msg) in seq.as_ref() {
            result.add_message(*time, msg);
        }
        result
    }

    pub fn to_file(&self, filename: &str) -> anyhow::Result<()> {
        let mut file = File::create(filename)?;
        writeln!(file, "{}", serde_json::to_string(self)?)?;
        Ok(())
    }

    pub fn midi_queue(&self) -> VecDeque<(f64, MidiMsg)> {
        self.records
            .iter()
            .map(|(t, v)| (*t, MidiMsg::from_midi(v).unwrap().0))
            .collect()
    }

    pub fn add_message(&mut self, time: f64, msg: &MidiMsg) {
        assert!(self.records.len() == 0 || self.records.last().unwrap().0 < time);
        self.records.push((time, msg.to_midi()));
    }

    pub fn record_loop(incoming: Arc<SegQueue<MidiMsg>>, outgoing: Arc<SegQueue<MidiMsg>>) -> Self {
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
                    result.add_message(seconds_since(timestamp_reference), &msg);
                } else if is_system_reset(&msg) {
                    return result;
                }
                outgoing.push(msg)
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
            let mut playback_queue = self.midi_queue();
            let kickoff = Instant::now();

            while playback_queue.len() > 0 {
                check_play_next_note(
                    &mut playback_queue,
                    kickoff,
                    outgoing.clone(),
                    &outgoing_func,
                );
            }

            match seconds_between_loops {
                None => break,
                Some(secs) => std::thread::sleep(Duration::from_secs_f64(secs)),
            }
        }
    }
}

fn check_play_next_note<M, F: Fn(MidiMsg) -> M>(
    note_queue: &mut VecDeque<(f64, MidiMsg)>,
    start_time: Instant,
    outgoing: Arc<SegQueue<M>>,
    outgoing_func: &F,
) {
    if note_queue.len() > 0 {
        let (goal, _) = note_queue[0];
        if Instant::now().duration_since(start_time).as_secs_f64() > goal {
            let (_, note) = note_queue.pop_front().unwrap();
            outgoing.push(outgoing_func(note));
        }
    }
}

pub fn stereo_playback<M, L: Fn(MidiMsg) -> M, R: Fn(MidiMsg) -> M>(
    left: &Recording,
    right: &Recording,
    outgoing: Arc<SegQueue<M>>,
    left_msg: L,
    right_msg: R,
) {
    let mut left_queue = left.midi_queue();
    let mut right_queue = right.midi_queue();
    let start_time = Instant::now();

    while left_queue.len() + right_queue.len() > 0 {
        check_play_next_note(&mut left_queue, start_time, outgoing.clone(), &left_msg);
        check_play_next_note(&mut right_queue, start_time, outgoing.clone(), &right_msg);
    }
}

fn read_file_to_string(filename: &str) -> anyhow::Result<String> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

#[cfg(test)]
mod tests {
    use crate::{read_file_to_string, Recording};

    #[test]
    fn test_ascending_timestamps() {
        let testee: Recording =
            serde_json::from_str(read_file_to_string("lean_on_me").unwrap().as_str()).unwrap();
        let mut queue = testee.midi_queue();
        let mut prev = None;
        while let Some((t, _)) = queue.pop_front() {
            if let Some(prev_time) = prev {
                assert!(prev_time < t);
            }
            prev = Some(t);
        }
    }
}
