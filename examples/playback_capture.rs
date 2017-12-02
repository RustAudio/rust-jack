//! Takes 2 audio inputs and outputs them to 2 audio outputs.
//! All JACK notifications are also printed out.
extern crate jack;
use jack::prelude as j;
use std::io;

struct Notifications;

impl j::NotificationHandler for Notifications {
    fn thread_init(&self, _: &j::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: j::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status,
            reason
        );
    }

    fn freewheel(&mut self, _: &j::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn buffer_size(&mut self, _: &j::Client, sz: j::JackFrames) -> j::JackControl {
        println!("JACK: buffer size changed to {}", sz);
        j::JackControl::Continue
    }

    fn sample_rate(&mut self, _: &j::Client, srate: j::JackFrames) -> j::JackControl {
        println!("JACK: sample rate changed to {}", srate);
        j::JackControl::Continue
    }

    fn client_registration(&mut self, _: &j::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &j::Client, port_id: j::JackPortId, is_reg: bool) {
        println!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &j::Client,
        port_id: j::JackPortId,
        old_name: &str,
        new_name: &str,
    ) -> j::JackControl {
        println!(
            "JACK: port with id {} renamed from {} to {}",
            port_id,
            old_name,
            new_name
        );
        j::JackControl::Continue
    }

    fn ports_connected(
        &mut self,
        _: &j::Client,
        port_id_a: j::JackPortId,
        port_id_b: j::JackPortId,
        are_connected: bool,
    ) {
        println!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &j::Client) -> j::JackControl {
        println!("JACK: graph reordered");
        j::JackControl::Continue
    }

    fn xrun(&mut self, _: &j::Client) -> j::JackControl {
        println!("JACK: xrun occurred");
        j::JackControl::Continue
    }

    fn latency(&mut self, _: &j::Client, mode: j::LatencyType) {
        println!(
            "JACK: {} latency has changed",
            match mode {
                j::LatencyType::Capture => "capture",
                j::LatencyType::Playback => "playback",
            }
        );
    }
}

fn main() {
    // Create client
    let (client, _status) = j::Client::new("rust_jack_simple", j::client_options::NO_START_SERVER)
        .unwrap();

    // Register ports. They will be used in a callback that will be
    // called when new data is available.
    let in_a = client
        .register_port("rust_in_l", j::AudioInSpec::default())
        .unwrap();
    let in_b = client
        .register_port("rust_in_r", j::AudioInSpec::default())
        .unwrap();
    let mut out_a = client
        .register_port("rust_out_l", j::AudioOutSpec::default())
        .unwrap();
    let mut out_b = client
        .register_port("rust_out_r", j::AudioOutSpec::default())
        .unwrap();
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

    // Activate the client, which starts the processing.
    let active_client = j::AsyncClient::new(client, Notifications, process).unwrap();

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}
