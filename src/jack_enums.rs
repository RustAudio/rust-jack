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
    PortConnectionError {
        source: String,
        destination: String,
        code_or_message: CodeOrMessage,
    },
    PortDisconnectionError,
    PortMonitorError,
    PortNamingError,
    PortRegistrationError(String),
    SetBufferSizeError,
    TimeError,
    WeakFunctionNotFound(&'static str),
    ClientIsNoLongerAlive,
    ClientPanicked,
    RingbufferCreateFailed,
    UnknownError {
        error_code: libc::c_int,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::LibraryError(err) => write!(f, "library error {err}"),
            Error::CallbackDeregistrationError => write!(f, "callback deregistration error"),
            Error::CallbackRegistrationError => write!(f, "callback registration error"),
            Error::ClientActivationError => write!(f, "client activation error"),
            Error::ClientDeactivationError => write!(f, "client deactivation error"),
            Error::ClientError(status) => write!(f, "client error, status is {status:?}"),
            Error::FreewheelError => write!(f, "freewheel error"),
            Error::InvalidDeactivation => write!(f, "invalid deactivation"),
            Error::NotEnoughSpace => write!(f, "not enough space"),
            Error::PortAliasError => write!(f, "port alias error"),
            Error::PortAlreadyConnected(a, b) => write!(f, "port {a} is already connected to {b}"),
            Error::PortConnectionError {
                source,
                destination,
                code_or_message: CodeOrMessage::Message(message),
            } => write!(
                f,
                "error connecting port {source} to port {destination}: {message}"
            ),
            Error::PortConnectionError {
                source,
                destination,
                code_or_message: CodeOrMessage::Code(code),
            } => write!(
                f,
                "error (code={code}) connecting port {source} to port {destination}, perhaps the source or destination port is not part of an active client"
            ),
            Error::PortDisconnectionError => write!(f, "port disconnection error"),
            Error::PortMonitorError => write!(f, "port monitoring error"),
            Error::PortNamingError => write!(f, "port naming error"),
            Error::PortRegistrationError(p) => write!(f, "failed to register port {p}"),
            Error::SetBufferSizeError => write!(
                f,
                "set buffer size error, setting buffer size is likely not supported"
            ),
            Error::TimeError => write!(f, "time error"),
            Error::WeakFunctionNotFound(func) => write!(f, "weak function {func} not found"),
            Error::ClientIsNoLongerAlive => write!(f, "client is no longer alive"),
            Error::ClientPanicked => write!(f, "client notifcation or processor panicked"),
            Error::RingbufferCreateFailed => write!(f, "ringbuffer creation failed"),
            Error::UnknownError { error_code } => write!(f, "unkown error with code {error_code}"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeOrMessage {
    Code(libc::c_int),
    Message(&'static str),
}

impl std::fmt::Display for CodeOrMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeOrMessage::Code(c) => write!(f, "code(code{c})"),
            CodeOrMessage::Message(msg) => write!(f, "{msg}"),
        }
    }
}

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
