extern crate jack;
use std::io;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::str::FromStr;

pub struct SinWave {
    frame_t: f64,
    frequency: f64,
    out_port: jack::Port,
    time: f64,
    receiver: Receiver<f64>,
    sender: Sender<f64>,
}

impl SinWave {
    pub fn new(out_port: jack::Port, freq: f64, sample_rate: f64) -> Self {
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

impl jack::JackHandler for SinWave {
    fn process(&mut self, n_frames: u32) -> jack::JackControl {
        // Get output buffer
        let out: &mut [f32] = unsafe { self.out_port.as_slice_mut(n_frames) };

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
        Err(_) => None
    }
}

fn main() {
    let mut client = jack::Client::open("rust_jack_sine", jack::NO_START_SERVER).unwrap();

    let out_port = client.register_port("sine_out", jack::DEFAULT_AUDIO_TYPE, jack::IS_OUTPUT, None)
        .unwrap();
    let app = SinWave::new(out_port, 220.0, client.sample_rate() as f64);
    let freq_request = app.frequency_requester();
    client.activate(app).unwrap();

    while let Some(f) = read_freq() {
        freq_request.send(f).unwrap();
    }

    client.deactivate().unwrap();
}
