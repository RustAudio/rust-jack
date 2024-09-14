//! Takes 2 audio inputs and outputs them to 2 audio outputs.
//! All JACK notifications are also printed out.
use std::io;

fn main() {
    // Create client
    let (client, _status) = jack::Client::new(
        "rust_jack_internal_client_tester",
        jack::ClientOptions::default(),
    )
    .unwrap();

    let int_client = client
        .load_internal_client("Jack Profiler (rust-jack test)", "profiler", "-c -p -e")
        .expect("Failed to Load Client");

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    let _ = client.unload_internal_client(int_client);
}
