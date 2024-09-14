---
layout: page
title: Closure Callbacks
parent: Contrib
permalink: /closure-callbacks
nav_order: 1
---

# Closure Callbacks

Closure callbacks allow you to define functionality inline.

## Process Closure

The typical use case for a process closure involves creating a closure that
contains captures the required state and then activating it.

```rust
// 1. Create the client.
let (client, _status) =
    jack::Client::new("silence", jack::ClientOptions::NO_START_SERVER).unwrap();

// 2. Define the state.
let mut output = client.register_port("out", jack::AudioOut::default());
let silence_value = 0.0;

// 3. Define the closure. Use `move` to capture the required state.
let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
    output.as_mut_slice(ps).fill(silence_value);
    jack::Control::Continue
};

// 4. Start processing.
let process = jack::contrib::ClosureProcessHandler::new(process_callback);
let active_client = client.activate_async((), process).unwrap();
```

## State + Process Closure + Buffer Closure

`jack::contrib::ClosureProcessHandler` also allows defining a buffer size
callback that can share state with the process callback. The buffer size
callback is useful as it allows the handler to adapt to any changes in the
buffer size.

```rust
// 1. Create the client.
let (client, _status) =
    jack::Client::new("silence", jack::ClientOptions::NO_START_SERVER).unwrap();

// 2. Define the state.
struct State {
    silence: Vec<f32>,
    output: jack::Port<jack::AudioOut>,
}
let state = State {
    silence: Vec::new(),
    output: client
        .register_port("out", jack::AudioOut::default())
        .unwrap(),
};

// 3. Define the state and closure.
let process_callback = |state: &mut State, _: &jack::Client, ps: &jack::ProcessScope| {
    state
        .output
        .as_mut_slice(ps)
        .copy_from_slice(state.silence.as_slice());
    jack::Control::Continue
};
let buffer_callback = |state: &mut State, _: &jack::Client, len: jack::Frames| {
    state.silence = vec![0f32; len as usize];
    jack::Control::Continue
};

// 4. Start processing.
let process =
    jack::contrib::ClosureProcessHandler::with_state(state, process_callback, buffer_callback);
let active_client = client.activate_async((), process).unwrap();
```
