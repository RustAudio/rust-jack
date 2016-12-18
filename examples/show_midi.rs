//! Creates a jack midi input port. The application prints out all
//! values sent to it.
extern crate jack;
extern crate rimd;
use std::io;
use jack::MidiStream;

fn main() {
    let (mut client, _status) =
        jack::Client::open("rust_jack_show_midi", jack::client_options::NO_START_SERVER).unwrap();
    let mut maker: jack::MidiOutPort = client.register_port("rust_midi_maker").unwrap();
    let mut shower: jack::MidiInPort = client.register_port("rust_midi_shower").unwrap();
    let cback = move |ps: &jack::ProcessScope| -> jack::JackControl {
        let show_data = shower.data(ps);
        for e in show_data.iter() {
            println!("{:?}", e);
        }
        let put_data = maker.data(ps);
        put_data.write(&[jack::MidiEvent::new(rimd::MidiMessage { data: vec![13, 14] },
                                              ps.n_frames() / 2)]);
        jack::JackControl::Continue
    };
    let active_client = client.activate(cback).unwrap();
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();
    active_client.deactivate().unwrap();
}
