use libc;

use jack_flags::client_status::ClientStatus;

/// The Error type that can occur within JACK.
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

/// Specify an option.
#[derive(Clone, Copy, Debug)]
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
