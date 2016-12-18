//! Takes 2 audio inputs and outputs them to 2 audio outputs.
extern crate jack;
use std::io;

fn main() {
    // Create client
    let (mut client, _status) =
        jack::Client::open("rust_jack_simple", jack::client_options::NO_START_SERVER).unwrap();

    // Register ports, that will be used in a callback that will be
    // called when new data is available.
    let mut in_a: jack::AudioInPort = client.register_port("rust_in_l").unwrap();
    let mut in_b: jack::AudioInPort = client.register_port("rust_in_r").unwrap();
    let mut out_a: jack::AudioOutPort = client.register_port("rust_out_l").unwrap();
    let mut out_b: jack::AudioOutPort = client.register_port("rust_out_r").unwrap();
    let process_callback = move |ps: &jack::ProcessScope| -> jack::JackControl {
        out_a.data(ps).buffer().clone_from_slice(in_a.data(ps).buffer());
        out_b.data(ps).buffer().clone_from_slice(in_b.data(ps).buffer());
        jack::JackControl::Continue
    };
    // Activate the client, which starts the processing.
    let active_client = client.activate(process_callback).unwrap();

    // Wait for user input
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}
