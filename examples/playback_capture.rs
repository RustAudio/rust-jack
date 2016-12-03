//! Takes 2 audio inputs and outputs them to 2 audio outputs.
extern crate jack;
use std::io;

/// jack client that will take to ports as input and play them as
/// output.
pub struct PlaybackCapture<'a> {
    in_ports: [jack::AudioInPort<'a>; 2],
    out_ports: [jack::AudioOutPort<'a>; 2],
}

impl<'a> PlaybackCapture<'a> {
    /// Create a new `PlaybackCapture` that listens on `in_ports` and
    /// outputs on `out_ports`.
    pub fn new(in_ports: [jack::AudioInPort<'a>; 2],
               out_ports: [jack::AudioOutPort<'a>; 2])
               -> Self {
        PlaybackCapture {
            in_ports: in_ports,
            out_ports: out_ports,
        }
    }
}

impl<'a> jack::JackHandler for PlaybackCapture<'a> {
    fn process(&mut self, process_scope: &jack::ProcessScope) -> jack::JackControl {
        // Process the two channels
        for _ in 0..2 {
            // Get input buffer
            let in_data = self.in_ports[0].data(&process_scope);
            let in_buff: &[f32] = in_data.buffer();
            // Get output buffer
            let mut out_data = self.out_ports[0].data(&process_scope);
            let out: &mut [f32] = out_data.buffer();
            // Write output
            out.clone_from_slice(in_buff);
        }
        jack::JackControl::Continue
    }
}

fn main() {
    // Create client
    let (mut client, _status) = jack::Client::open("rust_jack_simple", jack::NO_START_SERVER)
        .unwrap();

    // Create app, which implements the `JackHandler` trait.
    let app = PlaybackCapture::new([client.register_port("pc_in0").unwrap(),
                                    client.register_port("pc_in1").unwrap()],
                                   [client.register_port("pc_out0").unwrap(),
                                    client.register_port("pc_out1").unwrap()]);

    // Activate the client, which starts the processing.
    let active_client = client.activate(app).unwrap();

    // Wait for user input
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}
