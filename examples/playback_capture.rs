extern crate jack;
use std::io;

fn main() {
    // Create client
    let (mut client, _status) = jack::Client::open("rust_jack_simple", jack::NO_START_SERVER)
        .unwrap();

    // Create app, which implements the `JackHandler` trait.
    let mut in_a: jack::AudioInPort = client.register_port("pc_in0").unwrap();
    let mut in_b: jack::AudioInPort = client.register_port("pc_in1").unwrap();
    let mut out_a: jack::AudioOutPort = client.register_port("pc_out0").unwrap();
    let mut out_b: jack::AudioOutPort = client.register_port("pc_out0").unwrap();
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
