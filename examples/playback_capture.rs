extern crate jack;
use std::io;

pub struct PlaybackCapture<'a> {
    in_port: jack::AudioInPort<'a>,
    out_port: jack::AudioOutPort<'a>,
}

impl<'a> PlaybackCapture<'a> {
    pub fn new(in_port: jack::AudioInPort<'a>, out_port: jack::AudioOutPort<'a>) -> Self {
        PlaybackCapture {
            in_port: in_port,
            out_port: out_port,
        }
    }
}

impl<'a> jack::JackHandler for PlaybackCapture<'a> {
    fn process(&mut self, process_scope: &jack::ProcessScope) -> jack::JackControl {
        // Get output buffer
        let mut out_data = self.out_port.data(&process_scope);
        let out: &mut [f32] = out_data.buffer();
        // Get input buffer
        let in_data = self.in_port.data(&process_scope);
        let in_buff: &[f32] = in_data.buffer();

        // Write output
        out.clone_from_slice(in_buff);
        jack::JackControl::Continue
    }
}

fn main() {
    let (mut client, _status) = jack::Client::open("rust_jack_pc", jack::NO_START_SERVER).unwrap();

    let out_port = client.register_port("pc_out").unwrap();
    let in_port = client.register_port("pc_in").unwrap();
    let app = PlaybackCapture::new(in_port, out_port);
    let active_client = client.activate(app).unwrap();

    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}
