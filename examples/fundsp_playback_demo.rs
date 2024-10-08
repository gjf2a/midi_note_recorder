use std::sync::{Arc, Mutex};

use crossbeam_queue::SegQueue;
use midi_fundsp::{
    io::{start_output_thread, Speaker, SynthMsg},
    sounds::options,
};
use midi_note_recorder::Recording;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Usage: fundsp_playback_demo filename [-perpetual:num_secs_delay]")
    }

    let seconds_between_loops = args
        .iter()
        .find(|a| a.starts_with("-perpetual"))
        .map(|a| a.split(":").skip(1).next().unwrap().parse::<f64>().unwrap());

    let recording: Recording = Recording::from_file(args[1].as_str())?;
    let outgoing = Arc::new(SegQueue::new());
    let program_table = Arc::new(Mutex::new(options()));
    start_output_thread::<10>(outgoing.clone(), program_table.clone());
    recording.playback_loop(seconds_between_loops, outgoing, |msg| SynthMsg {
        msg,
        speaker: Speaker::Both,
    });
    Ok(())
}
