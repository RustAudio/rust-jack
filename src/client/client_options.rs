use bitflags::bitflags;
use jack_sys as j;

bitflags! {
    /// Option flags for opening a JACK client.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct ClientOptions: j::Enum_JackOptions {
        /// Do not automatically start the JACK server when it is not already running. This option
        /// is always selected if `$JACK_NO_START_SERVER` is defined in the calling process
        /// environment.
        const NO_START_SERVER = j::JackNoStartServer;

        /// Use the exact client name requested. Otherwise, JACK
        /// automatically generates a unique one if needed.
        const USE_EXACT_NAME  = j::JackUseExactName;

        /// Connect to a named JACK server. Set automatically by `Client::new_with_server_name`;
        /// `Client::new` rejects it because it has no server name to pass.
        const SERVER_NAME     = j::JackServerName;

        /// Load internal client from optional `load_name`, otherwise use the `client_name`.
        ///
        /// Not supported when opening a client; `Client::new` returns an error if it is set.
        const LOAD_NAME       = j::JackLoadName;

        /// Pass optional `load_init` to `jack_initialize()` entry point of an internal client.
        ///
        /// Not supported when opening a client; `Client::new` returns an error if it is set.
        const LOAD_INIT       = j::JackLoadInit;

        /// Pass a SessionID token. This allows the session manager to identify the client again.
        ///
        /// Not supported yet; `Client::new` returns an error if it is set.
        const SESSION_ID      = j::JackSessionID;
    }
}

impl ClientOptions {
    /// Options that make `jack_client_open` read an extra variadic argument. Passing one without
    /// its argument is undefined behavior, so `Client` rejects any it cannot supply.
    pub(crate) const VARARG_OPTIONS: ClientOptions = ClientOptions::SERVER_NAME
        .union(ClientOptions::LOAD_NAME)
        .union(ClientOptions::LOAD_INIT)
        .union(ClientOptions::SESSION_ID);
}

impl Default for ClientOptions {
    fn default() -> Self {
        ClientOptions::NO_START_SERVER
    }
}
