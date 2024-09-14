---
layout: page
title: Logging
permalink: /logging
nav_order: 2
---

# Logging

JACK can communicate info and error messages. By default, the [log
crate](https://github.com/rust-lang/log) is hooked up to output
messages. However, other logging methods can be used with the
[`set_logger`](https://docs.rs/jack/latest/jack/fn.set_logger.html) function.

## No Logging

Logging from `jack` can be disabled entirely by setting the logger to `None`.

```rust
jack::set_logger(jack::LoggerType::None);
```

## Log Crate (default)

The log crate is the default logger if the `log` feature is enabled, which is
enabled by default. The `log` crate provides a *facade* for logging; it provides
macros to perform logging, but another mechanism or crate is required to
actually perform the logging.

In the example below, we use the [`env_logger`
crate](https://crates.io/crates/env_logger) to display logging for info and
error severity level messages.

```rust
env_logger::builder().filter(None, log::LevelFilter::Info).init();

// JACK may log things to `info!` or `error!`.
let (client, _status) =
      jack::Client::new("rust_jack_simple", jack::ClientOptions::default()).unwrap();
```


## Stdio

If the `log` feature is not enabled, then `jack` will log info messages to
`stdout` and error messages to `stderr`. These usually show up in the terminal.

```rust
jack::set_logger(jack::LoggerType::Stdio);
```

## Custom

`jack::LoggerType::Custom` can be used to set a custom logger. Here is
stdout/stderr implemented as a custom logger:

```rust
fn main() {
    jack::set_logger(jack::LoggerType::Custom{info: stdout_handler, error: stderr_handler});
    ...
}

unsafe extern "C" fn stdout_handler(msg: *const libc::c_char) {
    let res = std::panic::catch_unwind(|| match std::ffi::CStr::from_ptr(msg).to_str() {
        Ok(msg) => println!("{}", msg),
        Err(err) => println!("failed to log to JACK info: {:?}", err),
    });
    if let Err(err) = res {
        eprintln!("{err:?}");
        std::mem::forget(err); // Prevent from rethrowing panic.
    }
}

unsafe extern "C" fn stderr_handler(msg: *const libc::c_char) {
    let res = std::panic::catch_unwind(|| match std::ffi::CStr::from_ptr(msg).to_str() {
        Ok(msg) => eprintln!("{}", msg),
        Err(err) => eprintln!("failed to log to JACK error: {:?}", err),
    });
    if let Err(err) = res {
        eprintln!("{err:?}");
        std::mem::forget(err); // Prevent from rethrowing panic.
    }
}
```
