//! Sine wave generator with frequency configuration exposed through standard
//! input.
extern crate jack;
use std::io;
use std::str::FromStr;
use std::sync::mpsc::channel;

fn main() {
    // 1. open a client
    let (client, _status) =
        jack::Client::new("rust_jack_sine", jack::ClientOptions::NO_START_SERVER).unwrap();

    // 2. register port
    let out_port = client
        .register_port("sine_out", jack::AudioOut::default())
        .unwrap();

    println!("{:?}", &out_port);
    drop(client);
    println!("{:?}", &out_port);
}
