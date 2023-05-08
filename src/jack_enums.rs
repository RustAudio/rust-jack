use crate::ClientStatus;

/// An error that can occur in JACK.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    LibraryError(String),
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
    WeakFunctionNotFound(&'static str),
    ClientIsNoLongerAlive,
    RingbufferCreateFailed,
    UnknownError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

/// Specify an option, either to continue processing, or to stop.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Control {
    /// Continue processing.
    #[default]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
