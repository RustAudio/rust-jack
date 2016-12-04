//! Creates a jack midi input port. The application prints out all
//! values sent to it.
extern crate jack;
use std::io;
use jack::{JackClient, MidiStream};

#[derive(Debug)]
pub struct MidiShow {
    midi_input: jack::MidiInPort,
}

impl MidiShow {
    pub fn new(midi: jack::MidiInPort) -> Self {
        MidiShow { midi_input: midi }
    }
}

impl JackHandler for MidiShow {
    fn process(&mut self, process_scope: &jack::ProcessScope) -> jack::JackControl {
        for e in self.midi_input.data(process_scope).iter() {
            // This is a toy program, ideally, we shouldn't be writing
            // to stdout inside the `process` callback.
            println!("{:?}", &e);
        }
        jack::JackControl::Continue
    }
}

fn main() {
    let (mut client, _status) = jack::Client::open("rust_jack_show_midi", jack::NO_START_SERVER)
        .unwrap();
    let shower = MidiShow::new(client.register_port("midi0").unwrap());
    let active_client = client.activate(shower).unwrap();
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();
    active_client.deactivate().unwrap();
}
