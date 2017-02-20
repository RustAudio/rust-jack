//! Takes 2 audio inputs and outputs them to 2 audio outputs.
//! All JACK notifications are also printed out.
extern crate jack;
use jack::prelude as j;
use std::io;

fn main() {
    // Create client
    let (client, _status) = j::Client::new("rust_jack_simple", j::client_options::NO_START_SERVER)
        .unwrap();

    // Register ports. They will be used in a callback that will be
    // called when new data is available.
    let in_a = client.register_port("rust_in_l", j::AudioInSpec::default()).unwrap();
    let in_b = client.register_port("rust_in_r", j::AudioInSpec::default()).unwrap();
    let mut out_a = client.register_port("rust_out_l", j::AudioOutSpec::default()).unwrap();
    let mut out_b = client.register_port("rust_out_r", j::AudioOutSpec::default()).unwrap();
    let process_callback = move |_: &j::Client, ps: &j::ProcessScope| -> j::JackControl {
        let mut out_a_p = j::AudioOutPort::new(&mut out_a, ps);
        let mut out_b_p = j::AudioOutPort::new(&mut out_b, ps);
        let in_a_p = j::AudioInPort::new(&in_a, ps);
        let in_b_p = j::AudioInPort::new(&in_b, ps);
        out_a_p.clone_from_slice(&in_a_p);
        out_b_p.clone_from_slice(&in_b_p);
        j::JackControl::Continue
    };
    let process = j::ClosureProcessHandler::new(process_callback);

    let notifications = j::ClosureNotificationHandler {
        thread_init_fn: move |_| {
            println!("JACK: thread initialized");
        },
        shutdown_fn: move |_, _| {
            println!("JACK: shut down");
        },
        freewheel_fn: move |_, is_on| {
            println!("JACK: freewheel mode is {}",
                     if is_on { "on" } else { "of" });
        },
        buffer_size_fn: move |_, sz| {
            println!("JACK: buffer size changed to {}", sz);
            j::JackControl::Continue
        },
        sample_rate_fn: move |_, sr| {
            println!("JACK: sample rate changed to {}", sr);
            j::JackControl::Continue
        },
        client_registration_fn: move |_, name, is_reg| {
            println!("JACK: client {} has {}",
                     name,
                     if is_reg { "registered" } else { "unregistered" });
        },
        port_registration_fn: |_, id, is_reg| {
            println!("JACK: port with id {} has been {}",
                     id,
                     if is_reg { "registered" } else { "unregistered" });
        },
        port_rename_fn: |_, id, old, new| {
            println!("JACK: port with id {} has been renamed from {} to {}",
                     id,
                     old,
                     new);
            j::JackControl::Continue
        },
        ports_connected_fn: |_, pa, pb, are_connected| {
            println!("JACK: ports with ids {} and {} have been {}",
                     pa,
                     pb,
                     if are_connected {
                         "connected"
                     } else {
                         "disconnected"
                     });
        },
        graph_reorder_fn: |_| {
            println!("JACK: graph reordered");
            j::JackControl::Continue
        },
        xrun_fn: |_| {
            println!("JACK: xrun occurred");
            j::JackControl::Continue
        },
        latency_fn: |_, lt| {
            println!("JACK: latency for {:?} has changed", lt);
        },
    };

    // Activate the client, which starts the processing.
    let active_client = j::AsyncClient::new(client, notifications, process).unwrap();

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}
