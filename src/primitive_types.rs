use jack_sys as j;

/// Type used to represent sample frame counts.
pub type Frames = j::jack_nframes_t;

/// Type used to represent the value of free running monotonic clock with units
/// of microseconds.
pub type Time = j::jack_time_t;

/// Ports have unique ids. A port registration callback is the only place you
/// ever need to know
/// their value.
pub type PortId = j::jack_port_id_t;
