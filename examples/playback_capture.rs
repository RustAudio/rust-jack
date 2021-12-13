//! Takes 2 audio inputs and outputs them to 2 audio outputs.
//! All JACK notifications are also printed out.
use std::io;

struct PlaybackCapture {
    out_a: jack::Port<jack::AudioOut>,
    out_b: jack::Port<jack::AudioOut>,
    in_a: jack::Port<jack::AudioIn>,
    in_b: jack::Port<jack::AudioIn>,
}

impl jack::ProcessHandler for PlaybackCapture {
    fn process(&mut self, _: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        let out_a = self.out_a.as_mut_slice(ps);
        let out_b = self.out_b.as_mut_slice(ps);
        let in_a = self.in_a.as_slice(ps);
        let in_b = self.in_b.as_slice(ps);
        out_a.clone_from_slice(in_a);
        out_b.clone_from_slice(in_b);
        jack::Control::Continue
    }

    fn buffer_size(&mut self, _: &jack::Client, _size: jack::Frames) -> jack::Control {
        jack::Control::Continue
    }
}

fn main() {
    // Create client
    let (client, _status) =
        jack::Client::new("rust_jack_simple", jack::ClientOptions::NO_START_SERVER).unwrap();

    // Register ports. They will be used in a callback that will be
    // called when new data is available.
    let in_a = client
        .register_port("rust_in_l", jack::AudioIn::default())
        .unwrap();
    let in_b = client
        .register_port("rust_in_r", jack::AudioIn::default())
        .unwrap();
    let out_a = client
        .register_port("rust_out_l", jack::AudioOut::default())
        .unwrap();
    let out_b = client
        .register_port("rust_out_r", jack::AudioOut::default())
        .unwrap();

    // Create the processor.
    let playback_capture = PlaybackCapture {
        out_a,
        out_b,
        in_a,
        in_b,
    };

    // Activate the client, which starts the processing.
    let active_client = client
        .activate_async(Notifications, playback_capture)
        .unwrap();

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        println!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
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

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun occurred");
        jack::Control::Continue
    }
}
