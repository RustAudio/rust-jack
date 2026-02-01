# Controller

The controller module provides utilities for building controllable JACK processors
with lock-free communication. This is useful when you need to send commands to or
receive notifications from your audio processor without blocking the real-time thread.

## Overview

The controller pattern separates your audio processing into two parts:

1. **Processor** - Runs in the real-time audio thread and handles audio/midi processing
2. **Controller** - Runs outside the real-time thread and can send commands or receive notifications

Communication between them uses lock-free ring buffers, making it safe for real-time audio.

## Basic Usage

Implement the `ControlledProcessorTrait` to create a controllable processor:

```rust
use jack::contrib::controller::{
    ControlledProcessorTrait, ProcessorChannels, ProcessorHandle,
};

// Define your command and notification types
enum Command {
    SetVolume(f32),
    Mute,
    Unmute,
}

enum Notification {
    ClippingDetected,
    VolumeChanged(f32),
}

// Define your processor state
struct VolumeProcessor {
    output: jack::Port<jack::AudioOut>,
    input: jack::Port<jack::AudioIn>,
    volume: f32,
    muted: bool,
}

impl ControlledProcessorTrait for VolumeProcessor {
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
        while let Ok(cmd) = channels.commands.pop() {
            match cmd {
                Command::SetVolume(v) => {
                    self.volume = v;
                    let _ = channels.notifications.push(Notification::VolumeChanged(v));
                }
                Command::Mute => self.muted = true,
                Command::Unmute => self.muted = false,
            }
        }

        // Process audio
        let input = self.input.as_slice(scope);
        let output = self.output.as_mut_slice(scope);
        let gain = if self.muted { 0.0 } else { self.volume };

        for (out, inp) in output.iter_mut().zip(input.iter()) {
            *out = inp * gain;
        }

        jack::Control::Continue
    }
}
```

## Creating and Using the Processor

Use the `instance` method to create both the processor and its control handle:

```rust
let (client, _status) =
    jack::Client::new("controlled", jack::ClientOptions::default()).unwrap();

let input = client.register_port("in", jack::AudioIn::default()).unwrap();
let output = client.register_port("out", jack::AudioOut::default()).unwrap();

let processor = VolumeProcessor {
    input,
    output,
    volume: 1.0,
    muted: false,
};

// Create the processor instance and control handle
// Arguments: notification channel size, command channel size
let (processor_instance, handle) = processor.instance(16, 16);

// Activate the client with the processor
let active_client = client.activate_async((), processor_instance).unwrap();

// Now you can control the processor from any thread
handle.commands.push(Command::SetVolume(0.5)).unwrap();

// And receive notifications
while let Ok(notification) = handle.notifications.pop() {
    match notification {
        Notification::ClippingDetected => println!("Clipping detected!"),
        Notification::VolumeChanged(v) => println!("Volume changed to {}", v),
    }
}
```

## Channel Capacities

When calling `instance`, you specify the capacity of both ring buffers:

- `notification_channel_size` - How many notifications can be queued from processor to controller
- `command_channel_size` - How many commands can be queued from controller to processor

Choose sizes based on your expected message rates. If a channel is full, `push` will fail,
so handle this appropriately in your code.

## Transport Sync

If your processor needs to respond to JACK transport changes, implement the `sync` method
and optionally set `SLOW_SYNC`:

```rust
impl ControlledProcessorTrait for MyProcessor {
    // ...

    const SLOW_SYNC: bool = true; // Set if sync may take multiple cycles

    fn sync(
        &mut self,
        _client: &jack::Client,
        state: jack::TransportState,
        pos: &jack::TransportPosition,
        channels: &mut ProcessorChannels<Self::Command, Self::Notification>,
    ) -> bool {
        // Handle transport state changes
        // Return true when ready to play
        true
    }
}
```
