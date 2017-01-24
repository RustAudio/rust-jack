use jack_sys as j;

/// Type used to represent sample frame counts.
pub type JackFrames = j::jack_nframes_t;

/// Type used to represent the value of free running monotonic clock with units of microseconds.
pub type JackTime = j::jack_time_t;

/// Ports have unique ids. A port registration callback is the only place you ever need to know
/// their value.
pub type JackPortId = j::jack_port_id_t;

/// `ProcessScope` provides information on the client and frame
/// timings within a `process` callback.
#[derive(Debug)]
pub struct ProcessScope {
    // To be used _only_ for runtime verification that the client who wrote
    // that the only ports being used are ones created by the client whose
    // handler is being run.
    client_ptr: *mut j::jack_client_t,

    // Used to allow safe access to IO port buffers
    n_frames: JackFrames,
}

impl ProcessScope {
    #[inline(always)]
    pub fn n_frames(&self) -> JackFrames {
        self.n_frames
    }

    #[inline(always)]
    pub fn client_ptr(&self) -> *mut j::jack_client_t {
        self.client_ptr
    }

    /// Create a `ProcessScope` for the client with the given pointer
    /// and the specified amount of frames.
    pub unsafe fn from_raw(n_frames: JackFrames, client_ptr: *mut j::jack_client_t) -> Self {
        ProcessScope {
            n_frames: n_frames,
            client_ptr: client_ptr,
        }
    }
}
