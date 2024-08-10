use std::{
    fs::File,
    io::Read,
    sync::{Arc, Mutex},
};

use crossbeam_queue::SegQueue;
use midi_fundsp::{
    io::{start_output_thread, Speaker, SynthMsg},
    sounds::options,
};
use midi_note_recorder::Recording;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Usage: fundsp_playback_demo filename")
    }

    let recording: Recording =
        serde_json::from_str(read_file_to_string(args[1].as_str())?.as_str())?;
    let outputs = Arc::new(SegQueue::new());
    let program_table = Arc::new(Mutex::new(options()));
    start_output_thread::<10>(outputs.clone(), program_table.clone());
    recording.playback_loop(outputs, |msg| SynthMsg {
        msg,
        speaker: Speaker::Both,
    });
    Ok(())
}

fn read_file_to_string(filename: &str) -> anyhow::Result<String> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
