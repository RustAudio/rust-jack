//! Sine wave generator with frequency configuration exposed through standard
//! input.

use crossbeam_channel::bounded;
use std::io;
use std::str::FromStr;

struct WaveGenerator {
    out_port: jack::Port<jack::AudioOut>,
    frequency: f64,
    time_seconds: f64,
    seconds_per_frame: f64,
    frequency_changer: crossbeam_channel::Receiver<f64>,
}

impl jack::ProcessHandler for WaveGenerator {
    fn process(&mut self, _: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        // Get output buffer
        let out = self.out_port.as_mut_slice(ps);

        // Check frequency requests
        while let Ok(f) = self.frequency_changer.try_recv() {
            self.time_seconds = 0.0;
            self.frequency = f;
        }

        // Write output
        for v in out.iter_mut() {
            let x = self.frequency * self.time_seconds * 2.0 * std::f64::consts::PI;
            let y = x.sin();
            *v = y as f32;
            self.time_seconds += self.seconds_per_frame;
        }

        // Continue as normal
        jack::Control::Continue
    }

    fn buffer_size(&mut self, _: &jack::Client, _size: jack::Frames) -> jack::Control {
        jack::Control::Continue
    }
}

fn main() {
    // 1. open a client
    let (client, _status) =
        jack::Client::new("rust_jack_sine", jack::ClientOptions::NO_START_SERVER).unwrap();

    // 2. register port
    let out_port = client
        .register_port("sine_out", jack::AudioOut::default())
        .unwrap();

    // 3. define process callback handler
    let sample_rate = client.sample_rate() as f64;
    let (tx, rx) = bounded(1_000_000);
    let wave_generator = WaveGenerator {
        out_port,
        frequency: 220.0,
        time_seconds: 0.0,
        seconds_per_frame: 1.0 / sample_rate,
        frequency_changer: rx,
    };

    // 4. Activate the client. Also connect the ports to the system audio.
    let active_client = client.activate_async((), wave_generator).unwrap();
    active_client
        .as_client()
        .connect_ports_by_name("rust_jack_sine:sine_out", "system:playback_1")
        .unwrap();
    active_client
        .as_client()
        .connect_ports_by_name("rust_jack_sine:sine_out", "system:playback_2")
        .unwrap();
    // processing starts here

    // 5. wait or do some processing while your handler is running in real time.
    println!("Enter an integer value to change the frequency of the sine wave.");
    while let Some(f) = read_freq() {
        tx.send(f).unwrap();
    }

    // 6. Optional deactivate. Not required since active_client will deactivate on
    // drop, though explicit deactivate may help you identify errors in
    // deactivate.
    active_client.deactivate().unwrap();
}

/// Attempt to read a frequency from standard in. Will block until there is
/// user input. `None` is returned if there was an error reading from standard
/// in, or the retrieved string wasn't a compatible u16 integer.
fn read_freq() -> Option<f64> {
    let mut user_input = String::new();
    match io::stdin().read_line(&mut user_input) {
        Ok(_) => u16::from_str(user_input.trim()).ok().map(|n| n as f64),
        Err(_) => None,
    }
}
