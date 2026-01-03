# JACK (for Rust)

Rust bindings for [JACK Audio Connection Kit](<https://jackaudio.org>).

| [![Crates.io](https://img.shields.io/crates/v/jack.svg)](https://crates.io/crates/jack) | [![License](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)                                                          |
|-----------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [![Docs.rs](https://docs.rs/jack/badge.svg)](https://docs.rs/jack)                      | [![Test](https://github.com/RustAudio/rust-jack/actions/workflows/testing.yml/badge.svg)](https://github.com/RustAudio/rust-jack/actions/workflows/testing.yml) |
| [📚 Documentation](https://rustaudio.github.io/rust-jack)                               | [:heart: Sponsor](<https://github.com/sponsors/wmedrano>)                                                                                                       |

## Using JACK


The JACK server is usually started by the user or system. Clients can request
that the JACK server is started on demand when they connect, but this can be
disabled by creating a client with the `NO_START_SERVER` option or
`ClientOptions::default()`.

-   Linux and BSD users may install JACK1, JACK2 (preferred for low latency), or
    Pipewire JACK (preferred for ease of use) from their system package manager.
-   Windows users may install JACK from the [official
    website](<http://jackaudio.org/downloads/>) or [Chocolatey](<https://community.chocolatey.org/packages/jack>).
-   MacOS users may install JACK from the [official
    website](<http://jackaudio.org/downloads/>) or [Homebrew](<https://formulae.brew.sh/formula/jack>).

Refer to the [docs.rs documentation](<https://docs.rs/jack/>) for details about
the API. For more general documentation, visit <https://rustaudio.github.io/rust-jack>.


## FAQ

### How do I return an `AsyncClient` with many generics?

This is especially useful when using `jack::contrib::ClosureProcessHandler`
which may have an innaccessible type.

```rust
// Shortest and allows access to the underlying client.
fn make_client() -> impl AsRef<jack::Client> {
    todo!()
}

// With extra bounds
fn make_client() -> impl 'static + AsRef<jack::Client> {
    todo!();
}

// For the full async client
fn async_client() -> impl jack::AsyncClient<impl Any, impl Any> {
    todo!();
}
```

# Testing

Testing requires setting up a dummy server and running the tests using a single
thread. `rust-jack` automatically configures `cargo nextest` to use a single
thread.

```sh
# Set up a dummy server for tests. The script is included in this repository.
./dummy_jack_server.sh &
# Run tests
cargo nextest run
```

Note: If cargo nextest is not available, use `RUST_TEST_THREADS=1 cargo test` to
run in single threaded mode.


## Possible Issues

If the tests are failing, a possible gotcha may be timing issues.

1.  If using `cargo test`, try `cargo nextest`. The `cargo nextest`
    configuration is set up to run single threaded and to retry flaky tests.

Another case is that libjack may be broken on your setup. Try using libjack2 or
pipewire-jack.
