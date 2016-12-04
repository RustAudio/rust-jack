use jack_sys as j;

bitflags! {
    /// Status flags for JACK clients.
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
    /// * `UNKNOWN_ERROR` - Not part of JACK and shouldn't occur ever.
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
