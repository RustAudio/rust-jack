//! Takes 2 audio inputs and outputs them to 2 audio outputs.
extern crate jack;
use std::io;

fn main() {
    // Create client
    let (mut client, _status) =
        jack::Client::open("rust_jack_simple", jack::client_options::NO_START_SERVER).unwrap();

    // Register ports. They will be used in a callback that will be
    // called when new data is available.
    let in_a = client.register_port("rust_in_l", jack::AudioInSpec::default()).unwrap();
    let in_b = client.register_port("rust_in_r", jack::AudioInSpec::default()).unwrap();
    let mut out_a = client.register_port("rust_out_l", jack::AudioOutSpec::default()).unwrap();
    let mut out_b = client.register_port("rust_out_r", jack::AudioOutSpec::default()).unwrap();
    let process_callback = move |ps: &jack::ProcessScope| -> jack::JackControl {
        let mut out_a_p = jack::AudioOutPort::new(&mut out_a, ps);
        let mut out_b_p = jack::AudioOutPort::new(&mut out_b, ps);
        let in_a_p = jack::AudioInPort::new(&in_a, ps);
        let in_b_p = jack::AudioInPort::new(&in_b, ps);
        out_a_p.clone_from_slice(&in_a_p);
        out_b_p.clone_from_slice(&in_b_p);
        jack::JackControl::Continue
    };
    // Activate the client, which starts the processing.
    let active_client = client.activate(process_callback).unwrap();

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}
