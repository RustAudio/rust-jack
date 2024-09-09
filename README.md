
# Table of Contents

1.  [Overview](#Overview-9s7h6d81ktj0)
2.  [Testing](#Testing-7y451e81ktj0)
    1.  [Possible Issues](#TestingPossibleIssues-8u551e81ktj0)

- [![img](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
- [![img](https://github.com/RustAudio/rust-jack/workflows/Rust/badge.svg)](https://github.com/RustAudio/rust-jack/actions)
- [![img](https://img.shields.io/crates/v/jack.svg)](https://crates.io/crates/jack)
- [![img](https://docs.rs/jack/badge.svg)](https://docs.rs/jack)


<a id="Overview-9s7h6d81ktj0"></a>

# Overview

Rust bindings for the [JACK Audio Connection Kit](<https://jackaudio.org>). These bindings work on every
operating system that JACK does.

The JACK server is usually started by the user or system. Clients can request that the JACK server is
started on demand when they connect, but this can be disabled by the user and is the recommended
configuration.

-   Linux and BSD users may install JACK1, JACK2, or Pipewire JACK from their
    system package manager.
-   Windows users may install JACK from the

[official website](<http://jackaudio.org/downloads/>) or
[Chocolatey](<https://community.chocolatey.org/packages/jack>).

-   macOS users may install JACK from the [official website](<http://jackaudio.org/downloads/>) or

[Homebrew](<https://formulae.brew.sh/formula/jack>).

[:heart: Sponsor](<https://github.com/sponsors/wmedrano>)

Refer to the [documentation](<https://docs.rs/jack/>) for details about the API, building, and packaging.
Also take a look at the `examples` directory for usage.


<a id="Testing-7y451e81ktj0"></a>

# Testing

Testing requires setting up a dummy server and running the tests using a single
thread.

```sh
# Set up a dummy server for tests. The script is included in this repository.
./dummy_jack_server.sh &
# Run tests with limited concurrency. Optionally, `cargo nextest run` is set up
# to run single threaded by default.
RUST_TEST_THREADS=1 cargo test

```

**Note**: We use a single thread for tests since too many client instantiations
in short periods of time cause some unit tests to interact negatively with each
other. Additionally the JACK server may become flaky.


<a id="TestingPossibleIssues-8u551e81ktj0"></a>

## Possible Issues

If the tests are failing, a possible gotcha may be timing issues.

1.  Increase the value used by `sleep_on_test` in `client/common.rs`.

Another case is that libjack may be broken on your setup. Try using libjack2 or
pipewire-jack.
