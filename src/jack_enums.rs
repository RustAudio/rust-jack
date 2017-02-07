// The MIT License (MIT)
//
// Copyright (c) 2017 Will Medrano (will.s.medrano@gmail.com)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use libc;

use client::client_status::ClientStatus;

/// An error that can occur in JACK.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JackErr {
    CallbackDeregistrationError,
    CallbackRegistrationError,
    ClientActivationError,
    ClientDeactivationError,
    ClientError(ClientStatus),
    FreewheelError,
    InvalidDeactivation,
    NotEnoughSpace,
    PortAliasError,
    PortAlreadyConnected(String, String),
    PortConnectionError(String, String),
    PortDisconnectionError,
    PortMonitorError,
    PortNamingError,
    PortRegistrationError(String),
    SetBufferSizeError,
    TimeError,
    UnknownError,
}

/// Used by `JackHandler::latency()`.
#[derive(Clone, Copy, Debug)]
pub enum LatencyType {
    Capture,
    Playback,
}

/// Specify an option, either to continue processing, or to stop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JackControl {
    /// Continue processing.
    Continue,

    /// Stop processing.
    Quit,
}

impl JackControl {
    pub fn to_ffi(self) -> libc::c_int {
        match self {
            JackControl::Continue => 0,
            JackControl::Quit => -1,
        }
    }
}

impl Default for JackControl {
    fn default() -> Self {
        JackControl::Continue
    }
}
