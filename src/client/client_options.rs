use bitflags::bitflags;
use jack_sys as j;

bitflags! {
    /// Option flags for opening a JACK client.
    ///
    /// JACK's `JackServerName`, `JackLoadName` and `JackLoadInit` options are not provided. They
    /// make `jack_client_open` read extra arguments that `Client::new` does not pass, and
    /// `Client::new` ignores them if they are set with `ClientOptions::from_bits_retain`.
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
    pub struct ClientOptions: j::Enum_JackOptions {
        /// Do not automatically start the JACK server when it is not already running. This option
        /// is always selected if `$JACK_NO_START_SERVER` is defined in the calling process
        /// environment.
        const NO_START_SERVER = j::JackNoStartServer;

        /// Use the exact client name requested. Otherwise, JACK
        /// automatically generates a unique one if needed.
        const USE_EXACT_NAME  = j::JackUseExactName;

        /// Pass a SessionID token. This allows the session manager to identify the client again.
        const SESSION_ID      = j::JackSessionID;
    }
}

impl Default for ClientOptions {
    fn default() -> Self {
        ClientOptions::NO_START_SERVER
    }
}
