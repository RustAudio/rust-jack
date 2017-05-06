use jack_sys as j;

bitflags! {
    /// Option flags for opening a JACK client.
    pub flags ClientOptions: j::Enum_JackOptions {
        /// Do not automatically start the JACK server when it is not already running. This option
        /// is always selected if `$JACK_NO_START_SERVER` is defined in the calling process
        /// environment.
        const NO_START_SERVER = j::JackNoStartServer,

        /// Use the exact client name requested. Otherwise, JACK
        /// automatically generates a unique one if needed.
        const USE_EXACT_NAME  = j::JackUseExactName,

        /// Open with optional `server_name` parameter.
        ///
        /// TODO: implement
        const SERVER_NAME     = j::JackServerName,

        /// Load internal client from optional `load_name`, otherwise use the `client_name`.
        ///
        /// TODO(wmedrano): Implement `load_name` functionality
        const LOAD_NAME       = j::JackLoadName,

        /// Pass optional `load_init` to `jack_initialize()` entry
        /// point of an internal client.
        ///
        /// TODO: implement
        const LOAD_INIT       = j::JackLoadInit,

        /// Pass a SessionID token. This allows the session manager to identify the client again.
        const SESSION_ID      = j::JackSessionID,
    }
}
