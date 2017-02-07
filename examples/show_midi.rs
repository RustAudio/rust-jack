// The MIT License (MIT)
//
// Copyright (c) 2017 Will Medrano (will.s.medrano@gmail.com)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

//! Creates a jack midi input and output ports. The application prints
//! out all values sent to it through the input port. It also sends a
//! Note On and Off event, once every cycle, on the output port.
extern crate jack;
use std::io;
use jack::prelude::{Client, JackClient, JackControl, MidiInPort, MidiInSpec, MidiOutPort,
                    MidiOutSpec, ProcessHandler, ProcessScope, RawMidi, WeakClient, client_options};

fn main() {
    // open client
    let (client, _status) = Client::open("rust_jack_show_midi", client_options::NO_START_SERVER)
        .unwrap();

    // process logic
    let mut maker = client.register_port("rust_midi_maker", MidiOutSpec::default()).unwrap();
    let shower = client.register_port("rust_midi_shower", MidiInSpec::default()).unwrap();
    let cback = move |_: &WeakClient, ps: &ProcessScope| -> JackControl {
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
        JackControl::Continue
    };

    // activate
    let process = ProcessHandler::new(cback);
    let active_client = client.activate(process).unwrap();

    // wait
    println!("Press any key to quit");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    // optional deactivation
    active_client.deactivate().unwrap();
}
