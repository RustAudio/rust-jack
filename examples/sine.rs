//! Sine wave generator with frequency configuration exposed through standard input.
extern crate jack;
use std::io;
use std::str::FromStr;
use std::sync::mpsc::{Sender, Receiver, channel};
use jack::JackClient;


pub struct SinWave<'a> {
    frame_t: f64,
    frequency: f64,
    out_port: jack::AudioOutPort<'a>,
    time: f64,
    receiver: Receiver<f64>,
    sender: Sender<f64>,
}

impl<'a> SinWave<'a> {
    pub fn new(out_port: jack::AudioOutPort<'a>, freq: f64, sample_rate: f64) -> Self {
        let (tx, rx) = channel();
        SinWave {
            frame_t: 1.0 / sample_rate,
            frequency: freq,
            out_port: out_port,
            time: 0.0,
            receiver: rx,
            sender: tx,
        }
    }

    pub fn frequency_requester(&self) -> Sender<f64> {
        self.sender.clone()
    }
}

impl<'a> jack::JackHandler for SinWave<'a> {
    fn process(&mut self, process_scope: &jack::ProcessScope) -> jack::JackControl {
        // Get output buffer
        let mut out_data = self.out_port.data(&process_scope);
        let out: &mut [f32] = out_data.buffer();

        // Check frequency requests
        while let Ok(f) = self.receiver.try_recv() {
            self.time = 0.0;
            self.frequency = f;
        }

        // Write output
        for v in out.iter_mut() {
            let x = self.frequency * self.time * 2.0 * std::f64::consts::PI;
            let y = x.sin();
            *v = y as f32;
            self.time += self.frame_t;
        }

        // Continue as normal
        jack::JackControl::Continue
    }
}

fn read_freq() -> Option<f64> {
    let mut user_input = String::new();
    match io::stdin().read_line(&mut user_input) {
        Ok(_) => u16::from_str(&user_input.trim()).ok().map(|n| n as f64),
        Err(_) => None,
    }
}

fn main() {
    let (mut client, _status) =
        jack::Client::open("rust_jack_sine", jack::client_options::NO_START_SERVER).unwrap();

    let out_port = client.register_port("sine_out").unwrap();
    let app = SinWave::new(out_port, 220.0, client.sample_rate() as f64);
    let freq_request = app.frequency_requester();
    let active_client = client.activate(app).unwrap();

    println!("Enter an integer value to change the frequency of the sine wave.");
    while let Some(f) = read_freq() {
        freq_request.send(f).unwrap();
    }

    active_client.deactivate().unwrap();
}
