use libc;

use ClientStatus;

/// An error that can occur in JACK.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
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
    WeakFunctionNotFound,
    UnknownError,
}

/// Used by `NotificationHandler::latency()`.
#[derive(Clone, Copy, Debug)]
pub enum LatencyType {
    Capture,
    Playback,
}

/// Specify an option, either to continue processing, or to stop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Control {
    /// Continue processing.
    Continue,

    /// Stop processing.
    Quit,
}

impl Control {
    pub fn to_ffi(self) -> libc::c_int {
        match self {
            Control::Continue => 0,
            Control::Quit => -1,
        }
    }
}

impl Default for Control {
    fn default() -> Self {
        Control::Continue
    }
}
