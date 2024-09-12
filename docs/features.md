---
layout: page
title: Features
permalink: /features
nav_order: 1
---

# Features

The Rust features for the `jack` crate are defined in
<https://github.com/RustAudio/rust-jack/blob/main/Cargo.toml>. To see the
documentation for Rust features in general, see the [Rust
Book](https://doc.rust-lang.org/cargo/reference/features.html).

## Disabling Default Features

The `jack` crate ships with a reasonable set of default features. To enable just
a subset of features, set `default-features` to false and select only the
desired features.

```toml
jack = { version = "..", default-features = false, features = ["log"] }
```

## `log`

Default: Yes

If the [`log` crate](https://crates.io/crates/log) should be used to handle JACK
logging. Requires setting up a logging implementation to make messages
available.

## `dynamic_loading`

Default: Yes

Load `libjack` at runtime as opposed to the standard dynamic linking. This is
preferred as it allows `pw-jack` to intercept the loading at runtime to provide
the Pipewire JACK server implementation.

## `metadata`

Default: No

Provides access to the metadata API. This is experimental. Details on the JACK
metadata API can be found at <https://jackaudio.org/metadata/>.
