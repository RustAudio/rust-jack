//! Sine wave generator using the controller pattern for lock-free communication.
//!
//! This example demonstrates how to use `ControlledProcessorTrait` to build
//! a controllable audio processor with commands and notifications.

use jack::contrib::controller::{ControlledProcessorTrait, ProcessorChannels};
use std::f64::consts::PI;
use std::io;
use std::str::FromStr;

/// Commands that can be sent to the audio processor.
enum Command {
    SetFrequency(f64),
    SetVolume(f32),
    Mute,
    Unmute,
}

/// Notifications sent from the audio processor.
enum Notification {
    FrequencyChanged(f64),
    VolumeChanged(f32),
    MuteStateChanged(bool),
}

/// The audio processor state.
struct SineProcessor {
    out_port: jack::Port<jack::AudioOut>,
    frequency: f64,
    volume: f32,
    muted: bool,
    frame_t: f64,
    time: f64,
}

impl ControlledProcessorTrait for SineProcessor {
    type Command = Command;
    type Notification = Notification;

    fn buffer_size(
        &mut self,
        _client: &jack::Client,
        _size: jack::Frames,
        _channels: &mut ProcessorChannels<Self::Command, Self::Notification>,
    ) -> jack::Control {
        jack::Control::Continue
    }

    fn process(
        &mut self,
        _client: &jack::Client,
        scope: &jack::ProcessScope,
        channels: &mut ProcessorChannels<Self::Command, Self::Notification>,
    ) -> jack::Control {
        // Handle incoming commands
        while let Some(cmd) = channels.recv_command() {
            match cmd {
                Command::SetFrequency(f) => {
                    self.frequency = f;
                    self.time = 0.0;
                    let _ = channels.try_notify(Notification::FrequencyChanged(f));
                }
                Command::SetVolume(v) => {
                    self.volume = v;
                    let _ = channels.try_notify(Notification::VolumeChanged(v));
                }
                Command::Mute => {
                    self.muted = true;
                    let _ = channels.try_notify(Notification::MuteStateChanged(true));
                }
                Command::Unmute => {
                    self.muted = false;
                    let _ = channels.try_notify(Notification::MuteStateChanged(false));
                }
            }
        }

        // Generate sine wave
        let out = self.out_port.as_mut_slice(scope);
        let gain = if self.muted { 0.0 } else { self.volume };

        for sample in out.iter_mut() {
            let x = self.frequency * self.time * 2.0 * PI;
            *sample = (x.sin() as f32) * gain;
            self.time += self.frame_t;
        }

        jack::Control::Continue
    }
}

fn main() {
    // 1. Open a client
    let (client, _status) =
        jack::Client::new("controlled_sine", jack::ClientOptions::default()).unwrap();

    // 2. Register port
    let out_port = client
        .register_port("sine_out", jack::AudioOut::default())
        .unwrap();

    // 3. Create the processor
    let processor = SineProcessor {
        out_port,
        frequency: 220.0,
        volume: 0.5,
        muted: false,
        frame_t: 1.0 / client.sample_rate() as f64,
        time: 0.0,
    };

    // 4. Create the processor instance and control handle
    let (processor_instance, mut handle) = processor.instance(16, 16);

    // 5. Activate the client
    let active_client = client.activate_async((), processor_instance).unwrap();
    active_client
        .as_client()
        .connect_ports_by_name("controlled_sine:sine_out", "system:playback_1")
        .unwrap();
    active_client
        .as_client()
        .connect_ports_by_name("controlled_sine:sine_out", "system:playback_2")
        .unwrap();

    // 6. Interactive control loop
    println!("Controlled Sine Wave Generator");
    println!("Commands:");
    println!("  <number>  - Set frequency in Hz (e.g., 440)");
    println!("  v<number> - Set volume 0-100 (e.g., v50)");
    println!("  m         - Mute");
    println!("  u         - Unmute");
    println!("  q         - Quit");
    println!();

    loop {
        // Check for notifications
        for notification in handle.drain_notifications() {
            match notification {
                Notification::FrequencyChanged(f) => println!("-> Frequency: {f} Hz"),
                Notification::VolumeChanged(v) => println!("-> Volume: {:.0}%", v * 100.0),
                Notification::MuteStateChanged(muted) => {
                    println!("-> {}", if muted { "Muted" } else { "Unmuted" })
                }
            }
        }

        // Read user input
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();

        if input.eq_ignore_ascii_case("q") {
            break;
        } else if input.eq_ignore_ascii_case("m") {
            let _ = handle.send_command(Command::Mute);
        } else if input.eq_ignore_ascii_case("u") {
            let _ = handle.send_command(Command::Unmute);
        } else if let Some(vol_str) = input.strip_prefix('v').or_else(|| input.strip_prefix('V')) {
            if let Ok(vol) = u8::from_str(vol_str) {
                let volume = (vol.min(100) as f32) / 100.0;
                let _ = handle.send_command(Command::SetVolume(volume));
            } else {
                println!("Invalid volume. Use v0-v100");
            }
        } else if let Ok(freq) = f64::from_str(input) {
            let _ = handle.send_command(Command::SetFrequency(freq));
        } else if !input.is_empty() {
            println!("Unknown command: {input}");
        }
    }

    // 7. Deactivate
    if let Err(err) = active_client.deactivate() {
        eprintln!("JACK exited with error: {err}");
    }
}
