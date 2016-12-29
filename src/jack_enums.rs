use jack_flags::client_status::ClientStatus;

/// The Error type that can occur within JACK.
#[derive(Clone, Debug)]
pub enum JackErr {
    CallbackRegistrationError,
    CallbackDeregistrationError,
    ClientActivationError,
    ClientDeactivationError,
    ClientError(ClientStatus),
    FreewheelError,
    InvalidDeactivation,
    PortAliasError,
    PortConnectionError,
    PortDisconnectionError,
    PortNamingError,
    PortMonitorError,
    PortNotFound,
    PortRegistrationError(String),
    SetBufferSizeError,
    TimeError,
    UnknownError,
    NotEnoughSpace,
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
    pub fn to_ffi(self) -> i32 {
        match self {
            JackControl::Continue => 0,
            JackControl::Quit => -1,
        }
    }
}
