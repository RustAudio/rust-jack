// info/error related stuff in logging.rs
mod test_logging;

// time related functionality in lib.rs
mod test_time;

// client.rs excluding functionality that involves ports or callbacks
mod test_client;

// client.rs for port functionality
mod test_client_port;

// client.rs and callback.rs for callback functionality
mod test_client_cback;

// port.rs
mod test_port;

// ringbuffer/mod.rs
mod test_ringbuffer;

// port_impls/audio.rs
mod test_port_audio;

// port_impls/midi.rs
mod test_port_midi;
