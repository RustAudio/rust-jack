use bitflags::bitflags;
use jack_sys as j;

bitflags! {
    /// Status flags for JACK clients.
    pub struct ClientStatus: j::Enum_JackStatus {
        /// Overall operation failed. File an issue if you can get it to appear.
        const FAILURE         = j::JackFailure;

        /// The operation contained an invalid or unsupported option.
        const INVALID_OPTION  = j::JackInvalidOption;

        /// The desired client name was not unique. With the `USE_EXACT_NAME` option this situation
        /// is fatal. Otherwise, the name was modified by appending a dash and a two-digit number in
        /// the range "-01" to "-99". `Client::name()` will return the exact string that was
        /// used. If the specified client_name plus these extra characters would be too long, the
        /// open fails instead.
        const NAME_NOT_UNIQUE = j::JackNameNotUnique;

        /// The JACK server was started as a result of this operation. Otherwise, it was running
        /// already. In either case the caller is now connected to jackd, so there is no race
        /// condition. When the server shuts down, the client will find out.
        const SERVER_STARTED  = j::JackServerStarted;

        /// Unable to connect to the JACK server.
        const SERVER_FAILED   = j::JackServerFailed;

        /// Communication error with the JACK server.
        const SERVER_ERROR    = j::JackServerError;

        /// Requested client does not exist.
        const NO_SUCH_CLIENT  = j::JackNoSuchClient;

        /// Unable to load internal client
        const LOAD_FAILURE    = j::JackLoadFailure;

        /// Unable to initialize client
        const INIT_FAILURE    = j::JackInitFailure;

        /// Unable to access shared memory
        const SHM_FAILURE     = j::JackShmFailure;

        /// Client's protocol version does not match
        const VERSION_ERROR   = j::JackVersionError;

        /// No documentation found. TODO: dig deeper
        const BACKEND_ERROR   = j::JackBackendError;

        /// No documentation found. TODO: dig deeper
        const CLIENT_ZOMBIE   = j::JackClientZombie;
    }
}
