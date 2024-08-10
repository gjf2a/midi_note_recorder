use std::{
    fs::File,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use crossbeam_queue::SegQueue;
use crossbeam_utils::atomic::AtomicCell;
use midi_fundsp::{
    io::{choose_midi_device, start_midi_input_thread, start_output_thread, Speaker, SynthMsg},
    sounds::options,
};
use midi_msg::{MidiMsg, SystemRealTimeMsg};
use midi_note_recorder::Recording;
use midir::MidiInput;
use read_input::{shortcut::input, InputBuild};
use std::io::Write;

fn main() -> anyhow::Result<()> {
    let reset = Arc::new(AtomicCell::new(false));

    let mut midi_in = MidiInput::new("midir reading input")?;
    let in_port = choose_midi_device(&mut midi_in)?;
    let inputs = Arc::new(SegQueue::new());
    let outputs = Arc::new(SegQueue::new());
    start_midi_input_thread(inputs.clone(), midi_in, in_port, reset.clone());
    let program_table = Arc::new(Mutex::new(options()));
    start_output_thread::<10>(outputs.clone(), program_table.clone());
    let recording_handle = recording_thread(inputs.clone(), outputs.clone());
    input::<String>().msg("Press any key to exit\n").get();
    reset.store(true);
    let recording = recording_handle.join().unwrap();
    let filename = input::<String>().msg("Enter filename for recording:").get();
    let mut file = File::create(filename)?;
    writeln!(file, "{}", serde_json::to_string(&recording)?)?;
    println!("File written; exiting...");
    Ok(())
}

fn recording_thread(
    incoming: Arc<SegQueue<MidiMsg>>,
    outgoing: Arc<SegQueue<SynthMsg>>,
) -> JoinHandle<Recording> {
    std::thread::spawn(move || {
        Recording::record_loop(incoming, outgoing, |msg| SynthMsg {
            msg,
            speaker: Speaker::Both,
        })
    })
}
