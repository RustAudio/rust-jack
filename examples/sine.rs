//! Sine wave generator with frequency configuration exposed through standard
//! input.

use crossbeam_channel::bounded;
use std::io;
use std::str::FromStr;

fn main() {
    // 1. open a client
    let (client, _status) =
        jack::Client::new("rust_jack_sine", jack::ClientOptions::NO_START_SERVER).unwrap();

    // 2. register port
    let out_port = client
        .register_port("sine_out", jack::AudioOut::default())
        .unwrap();

    // 3. define process callback handler
    let (tx, rx) = bounded(1_000_000);
    struct State {
        out_port: jack::Port<jack::AudioOut>,
        rx: crossbeam_channel::Receiver<f64>,
        frequency: f64,
        frame_t: f64,
        time: f64,
    }
    let process = jack::contrib::ClosureProcessHandler::with_state(
        State {
            out_port,
            rx,
            frequency: 220.0,
            frame_t: 1.0 / client.sample_rate() as f64,
            time: 0.0,
        },
        |state, _, ps| -> jack::Control {
            // Get output buffer
            let out = state.out_port.as_mut_slice(ps);

            // Check frequency requests
            while let Ok(f) = state.rx.try_recv() {
                state.time = 0.0;
                state.frequency = f;
            }

            // Write output
            for v in out.iter_mut() {
                let x = state.frequency * state.time * 2.0 * std::f64::consts::PI;
                let y = x.sin();
                *v = y as f32;
                state.time += state.frame_t;
            }

            // Continue as normal
            jack::Control::Continue
        },
        move |_, _, _| jack::Control::Continue,
    );

    // 4. Activate the client. Also connect the ports to the system audio.
    let active_client = client.activate_async((), process).unwrap();
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
    if let Err(err) = active_client.deactivate() {
        eprintln!("JACK exited with error: {err}");
    };
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
