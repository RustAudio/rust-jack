//! Sine wave generator with frequency configuration exposed through standard input.
extern crate jack;
use std::io;
use std::str::FromStr;
use std::sync::mpsc::channel;
use jack::prelude::{AudioOutPort, AudioOutSpec, Client, JackControl, ProcessHandler, ProcessScope,
                    ActiveClient, client_options};


/// Attempt to read a frequency from standard in. Will block until there is user input. `None` is
/// returned if there was an error reading from standard in, or the retrieved string wasn't a
/// compatible u16 integer.
fn read_freq() -> Option<f64> {
    let mut user_input = String::new();
    match io::stdin().read_line(&mut user_input) {
        Ok(_) => u16::from_str(&user_input.trim()).ok().map(|n| n as f64),
        Err(_) => None,
    }
}

fn main() {
    // 1. open a client
    let (client, _status) = Client::open("rust_jack_sine", client_options::NO_START_SERVER)
        .unwrap();

    // 2. register port
    let mut out_port = client.register_port("sine_out", AudioOutSpec::default()).unwrap();

    // 3. define process callback handler
    let mut frequency = 220.0;
    let sample_rate = client.sample_rate();
    let frame_t = 1.0 / sample_rate as f64;
    let mut time = 0.0;
    let (tx, rx) = channel();
    let process = ProcessHandler::new(move |_: &Client, ps: &ProcessScope| -> JackControl {
        // Get output buffer
        let mut out_p = AudioOutPort::new(&mut out_port, ps);
        let out: &mut [f32] = &mut out_p;

        // Check frequency requests
        while let Ok(f) = rx.try_recv() {
            time = 0.0;
            frequency = f;
        }

        // Write output
        for v in out.iter_mut() {
            let x = frequency * time * 2.0 * std::f64::consts::PI;
            let y = x.sin();
            *v = y as f32;
            time += frame_t;
        }

        // Continue as normal
        JackControl::Continue
    });

    // 4. activate the client
    let active_client = ActiveClient::new(client, process).unwrap();
    // processing starts here

    // 5. wait or do some processing while your handler is running in real time.
    println!("Enter an integer value to change the frequency of the sine wave.");
    while let Some(f) = read_freq() {
        tx.send(f).unwrap();
    }

    // 6. Optional deactivate. Not required since active_client will deactivate on drop, though
    // explicit deactivate may help you identify errors in deactivate.
    active_client.deactivate().unwrap();
}
