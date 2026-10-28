# Changelog

## Unreleased

- `Error` is now `#[non_exhaustive]`. `match`es on it outside of this crate need a wildcard arm.
- Added `Error::PortUnregistrationError`.
- `Client::unregister_port` now returns `Err(Error::PortUnregistrationError)` on failure instead
  of `Err(Error::PortDisconnectionError)`.

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
