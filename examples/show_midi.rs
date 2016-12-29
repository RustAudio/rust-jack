//! Creates a jack midi input and output ports. The application prints
//! out all values sent to it through the input port. It also sends a
//! Note On and Off event, once every cycle, on the output port.
extern crate jack;
use std::io;
use jack::{MidiInSpec, MidiInPort, MidiOutSpec, MidiOutPort, RawMidi};

fn main() {
    let (mut client, _status) =
        jack::Client::open("rust_jack_show_midi", jack::client_options::NO_START_SERVER).unwrap();
    let mut maker = client.register_port("rust_midi_maker", MidiOutSpec).unwrap();
    let shower = client.register_port("rust_midi_shower", MidiInSpec).unwrap();
    let cback = move |ps: &jack::ProcessScope| -> jack::JackControl {
        let show_p = MidiInPort::new(&shower, ps);
        for e in show_p.iter() {
            println!("{:?}", e);
        }
        let mut put_p = MidiOutPort::new(&mut maker, ps);
        put_p.write(&RawMidi {
                time: 0,
                bytes: &[0b10010000 /* Note On, channel 1 */,
                         0b01000000 /* Key number */, 0b01111111 /* Velocity */],
            })
            .unwrap();
        put_p.write(&RawMidi {
                time: ps.n_frames() / 2,
                bytes: &[0b10000000 /* Note Off, channel 1 */,
                         0b01000000 /* Key number */, 0b01111111 /* Velocity */],
            })
            .unwrap();
        jack::JackControl::Continue
    };
    let active_client = client.activate(cback).unwrap();
    println!("Press any key to quit");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();
    active_client.deactivate().unwrap();
}
