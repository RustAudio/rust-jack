# Changelog

## Unreleased

- Ports and transport objects now hold strong references to their JACK client and remain
  usable after `Client` is dropped.
- Added `ClientHandle` with an `as_ptr` method. `Port::from_raw` now takes
  `Arc<ClientHandle>` instead of `Weak<()>` and no longer takes a separate client pointer.
- Callback cleanup now explicitly stops processing and detaches callbacks before freeing
  their handlers, including after failed activation, even if ports or transport keep the
  client open ([#232](https://github.com/RustAudio/rust-jack/issues/232)).
- `Error` is now `#[non_exhaustive]`. `match`es on it outside of this crate need a wildcard arm.
- Added `Error::PortUnregistrationError`.
- `Client::unregister_port` now returns `Err(Error::PortUnregistrationError)` on failure instead
  of `Err(Error::PortDisconnectionError)`.
- `Client::register_port` with the `Unowned` spec now returns `Err(Error::PortRegistrationError)`.
- `Unowned::jack_buffer_size` now returns `0` instead of panicking.
- `MidiIter` no longer dereferences a null port buffer.
- `MidiIter` `next`, `nth`, `count`and `size_hint` no longer overflow or
  underflow when iterating past the end.
- `Client::property_get_subject` now returns an empty `PropertyMap` instead of `None` for a
  subject without properties, on both jackd2 and PipeWire. It also no longer reads an
  uninitialized description.
- Property types reported as an empty string (as PipeWire does for a missing type) are now
  `None` instead of `Some("")`, in `Client::property_get` and `Client::property_get_subject`.
  Non-UTF-8 property types no longer panic.
- `Client::property_get`, `property_get_subject` and `property_get_all` no longer panic on
  non-UTF-8 keys, values or types. Invalid bytes are replaced with `U+FFFD`.
- Actually stop processing if buffer size callback returns Quit. This was not
  respected in Pipewire.

## 0.9.0

- Jack library can be loaded dynamically from libjack.so.0 or libjack.dll.
- Updated dependencies to latest as of 2022/02/05.

## 0.8.2

- `MidiIter` is now `Sync`.

## 0.8.1

- `AsyncClient` and `Client` are `Sync`.

## 0.8.0

- `latency` no longer supported for `NotificationHandler`.
- JACK messages are automatically routed through the `log` crate.
- The following functions have been removed: `set_info_callback`, `info_callback`, `reset_info_callback`, `set_error_callback`, `error_callback`, and `reset_error_callback`.
