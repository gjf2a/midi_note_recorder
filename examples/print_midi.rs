use std::{fs::File, io::Read};

use midi_note_recorder::{note_velocity_from, Recording};

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Usage: print_midi filename")
    }
    let recording: Recording =
        serde_json::from_str(read_file_to_string(args[1].as_str())?.as_str())?;
    for (t, msg) in recording.midi_queue() {
        if let Some((n, v)) = note_velocity_from(&msg) {
            println!("{t:.3}\t{n:>4}\t{v:>4}");
        }
    }
    Ok(())
}

fn read_file_to_string(filename: &str) -> anyhow::Result<String> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
