use crate::ClientStatus;

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
    ClientIsNoLongerAlive,
    RingbufferCreateFailed,
    UnknownError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "JackError: {:?}", &self) // FIXME
    }
}

impl std::error::Error for Error {}

/// Used by `NotificationHandler::latency()`.
#[derive(Clone, Copy, Debug)]
pub enum LatencyType {
    Capture,
    Playback,
}

impl LatencyType {
    pub fn to_ffi(self) -> libc::c_uint {
        match self {
            LatencyType::Playback => jack_sys::JackPlaybackLatency,
            LatencyType::Capture => jack_sys::JackCaptureLatency,
        }
    }
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
