use jack_sys as j;

bitflags! {
    /// Option flags for opening a JACK client.
    ///
    /// * `NULL_OPTION` - Equivalent to `ClientOptions::empty()`
    ///
    /// * `NO_START_SERVER`: Do not automatically start the JACK server when it
    /// is not already running. This option is always selected if
    /// `$JACK_NO_START_SERVER` is defined in the calling process
    /// environment.
    ///
    /// * `USE_EXACT_NAME`: Use the exact client name requested. Otherwise,
    /// JACK automatically generates a unique one if needed.
    ///
    /// * `SERVER_NAME`: Open with optional `server_name` parameter. TODO:
    /// implement
    ///
    /// * `LOAD_NAME`: Load internal client from optional `load_name`,
    /// otherwise use the `client_name`. TODO implement
    ///
    /// * `LOAD_INIT`: Pass optional `load_init` to `jack_initialize()`
    /// entry point of an internal client. TODO: implement
    ///
    /// * `SESSION_ID`: Pass a SessionID token. This allows the session
    /// manager to identify the client again.
    pub flags ClientOptions: u32 {
        const NULL_OPTION     = j::JackNullOption,
        const NO_START_SERVER = j::JackNoStartServer,
        const USE_EXACT_NAME  = j::JackUseExactName,
        const SERVER_NAME     = j::JackServerName,
        const LOAD_NAME       = j::JackLoadName,
        const LOAD_INIT       = j::JackLoadInit,
        const SESSION_ID      = j::JackSessionID,
    }
}
