use jack_sys as j;

bitflags! {
    /// Option flags for opening a Jack client.
    ///
    /// * `NULL_OPTION` - Equivalent to `ClientOptions::empty()`
    ///
    /// * `NO_START_SERVER`: Do not automatically start the Jack server when it
    /// is not already running. This option is always selected if
    /// `$JACK_NO_START_SERVER` is defined in the calling process
    /// environment.
    ///
    /// * `USE_EXACT_NAME`: Use the exact client name requested. Otherwise,
    /// Jack automatically generates a unique one if needed.
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

bitflags! {
    /// Status flags for Jack clients.
    ///
    /// * `FAILURE` - Overall operation failed.
    ///
    /// * `INVALID_OPTION` - The operation contained an invalid or unsupported
    /// option.
    ///
    /// * `NAME_NOT_UNIQUE` - The desired client name was not unique. With the
    /// `USE_EXACT_NAME` option this situation is fatal. Otherwise, the name was
    /// modified by appending a dash and a two-digit number in the range "-01"
    /// to "-99". `Client::name()` will return the exact string that was
    /// used. If the specified client_name plus these extra characters would be
    /// too long, the open fails instead.
    ///
    /// * `SERVER_STARTED` - The JACK server was started as a result of this
    /// operation. Otherwise, it was running already. In either case the caller
    /// is now connected to jackd, so there is no race condition. When the
    /// server shuts down, the client will find out.
    ///
    /// * `SERVER_FAILED` - Unable to connect to the JACK server.
    ///
    /// * `SERVER_ERROR` - Communication error with the JACK server.
    ///
    /// * `NO_SUCH_CLIENT` - Requested client does not exist.
    ///
    /// * `LOAD_FAILURE` - Unable to load internal client
    ///
    /// * `INIT_FAILURE` - Unable to initialize client
    ///
    /// * `SHM_FAILURE` - Unable to access shared memory
    ///
    /// * `VERSION_ERROR` - Client's protocol version does not match
    ///
    /// * `BACKEND_ERROR` - No documentation found. TODO: dig deeper
    ///
    /// * `CLIENT_ZOZMBIE` - No documentation found. TODO: dig deeper
    ///
    /// * `UNKNOWN_ERROR` - Not part of jack and shouldn't occur ever.
    /// File an issue if you can get it to appear.
    pub flags ClientStatus: u32 {
        const FAILURE         = j::JackFailure,
        const INVALID_OPTION  = j::JackInvalidOption,
        const NAME_NOT_UNIQUE = j::JackNameNotUnique,
        const SERVER_STARTED  = j::JackServerStarted,
        const SERVER_FAILED   = j::JackServerFailed,
        const SERVER_ERROR    = j::JackServerError,
        const NO_SUCH_CLIENT  = j::JackNoSuchClient,
        const LOAD_FAILURE    = j::JackLoadFailure,
        const INIT_FAILURE    = j::JackInitFailure,
        const SHM_FAILURE     = j::JackShmFailure,
        const VERSION_ERROR   = j::JackVersionError,
        const BACKEND_ERROR   = j::JackBackendError,
        const CLIENT_ZOMBIE   = j::JackClientZombie,
        const UNKNOWN_ERROR   = 0x2000, // TODO: don't use this
    }
}

bitflags! {
    /// Flags for specifying port options.
    ///
    /// * `IS_INPUT` - The port can receive data.
    ///
    /// * `IS_OUTPUT` - Data can be read from the port.
    ///
    /// * `IS_PHYSICAL` - Port corresponds to some kind of physical I/O
    /// connector.
    ///
    /// * `CAN_MONITOR` - A call to `jack_port_request_monitor()` makes
    /// sense. TODO: implement. Precisely what this means it dependent on the
    /// client. A typical result of it being called with `true` as the second
    /// argument is that data that would be available from an output port (with
    /// `IS_PHYSICAL` set) is sent to a physical output connector as well, so
    /// that it can be heard/seen/whatever.
    ///
    /// * `IS_TERMINAL` - For an input port, the data received by the port will
    /// not be passed on or made available at any other port. For output, the
    /// data available at the port does not originate from any other port. Audio
    /// synthesizers, I/O hardware interface clients, HDR systems are examples
    /// of clients that would set this flag for their ports.
    pub flags PortFlags: u32 {
        const NO_PORT_FLAGS = 0,
        const IS_INPUT    = j::JackPortIsInput,
        const IS_OUTPUT   = j::JackPortIsOutput,
        const IS_PHYSICAL = j::JackPortIsPhysical,
        const CAN_MONITOR = j::JackPortCanMonitor,
        const IS_TERMINAL = j::JackPortIsTerminal,
    }
}
