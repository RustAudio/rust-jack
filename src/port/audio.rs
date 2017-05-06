use std::ops::{Deref, DerefMut};
use std::slice;

use jack_sys as j;
use libc;

use client::ProcessScope;
use port::port_flags::{IS_INPUT, IS_OUTPUT, PortFlags};
use port::{Port, PortSpec};

/// `AudioInSpec` implements the `PortSpec` trait which, defines an
/// endpoint for JACK. In this case, it is a readable 32 bit floating
/// point buffer for audio.
///
/// `AudioInSpec::buffer()` is used to gain access the buffer.
///
/// # Example
/// ```
/// let client = jack::client::Client::new("rusty_client", jack::client::client_options::NO_START_SERVER).unwrap().0;
/// let spec = jack::port::AudioInSpec::default();
/// let audio_in_port = client.register_port("in", spec).unwrap();
/// ```
#[derive(Debug, Default)]
pub struct AudioInSpec;

/// `AudioOutSpec` implements the `PortSpec` trait, which defines an
/// endpoint for JACK. In this case, it is a mutable 32 bit floating
/// point buffer for audio.
///
/// `AudioOutSpec::buffer()` is used to gain access the buffer.
///
/// # Example
/// ```
/// let client = jack::client::Client::new("rusty_client", jack::client::client_options::NO_START_SERVER).unwrap().0;
/// let spec = jack::port::AudioInSpec::default();
/// let audio_out_port = client.register_port("out", spec).unwrap();
/// ```
#[derive(Debug, Default)]
pub struct AudioOutSpec;


unsafe impl<'a> PortSpec for AudioOutSpec {
    fn jack_port_type(&self) -> &'static str {
        j::FLOAT_MONO_AUDIO
    }

    fn jack_flags(&self) -> PortFlags {
        IS_OUTPUT
    }

    fn jack_buffer_size(&self) -> libc::c_ulong {
        // Not needed for built in types according to JACK api
        0
    }
}

unsafe impl PortSpec for AudioInSpec {
    /// Create an AudioInSpec instance from a buffer pointer and frame
    /// count. This is mostly used by `Port<AudioInSpec>` within a
    /// `process` scope.
    ///
    /// # Arguments
    ///
    /// * `ptr` - buffer pointer to underlying data.
    ///
    /// * `nframes` - the size of the buffer.

    fn jack_port_type(&self) -> &'static str {
        j::FLOAT_MONO_AUDIO
    }

    fn jack_flags(&self) -> PortFlags {
        IS_INPUT
    }

    fn jack_buffer_size(&self) -> libc::c_ulong {
        // Not needed for built in types according to JACK api
        0
    }
}

/// Safetly and thinly wrap a `Port<AudioOutSpec>`. Derefs into a `&mut[f32]`.
///
/// # Example
/// ```
/// let client = jack::client::Client::new("c", jack::client::client_options::NO_START_SERVER).unwrap().0;
/// let mut out_port = client.register_port("p", jack::port::AudioOutSpec::default()).unwrap();
/// let _process = move |_: &jack::client::Client, ps: &jack::client::ProcessScope| {
///     let mut out_p = jack::port::AudioOutPort::new(&mut out_port, ps);
///     {
///         let out_b: &mut [f32] = &mut out_p; // can deref into &mut [f32]
///     }
///     out_p[0] = 0.0;
/// };
/// ```
pub struct AudioOutPort<'a> {
    _port: &'a mut Port<AudioOutSpec>,
    buffer: &'a mut [f32],
}

impl<'a> AudioOutPort<'a> {
    /// Wrap a `Port<AudioOutSpec>` within a process scope of a client
    /// that registered the port. Panics if the port does not belong
    /// to the client that created the process.
    pub fn new(port: &'a mut Port<AudioOutSpec>, ps: &'a ProcessScope) -> Self {
        assert_eq!(port.client_ptr(), ps.client_ptr());
        let buff = unsafe {
            slice::from_raw_parts_mut(port.buffer(ps.n_frames()) as *mut f32,
                                      ps.n_frames() as usize)
        };
        AudioOutPort {
            _port: port,
            buffer: buff,
        }
    }
}

impl<'a> Deref for AudioOutPort<'a> {
    type Target = [f32];

    fn deref(&self) -> &[f32] {
        self.buffer
    }
}

impl<'a> DerefMut for AudioOutPort<'a> {
    fn deref_mut(&mut self) -> &mut [f32] {
        self.buffer
    }
}


/// Safetly and thinly wrap a `Port<AudioInSpec>`. Derefs into a `&[f32]`.
///
/// # Example
/// ```
/// let client = jack::client::Client::new("c", jack::client::client_options::NO_START_SERVER).unwrap().0;
/// let in_port = client.register_port("p", jack::port::AudioInSpec::default()).unwrap();
/// let process = move |_: &jack::client::Client, ps: &jack::client::ProcessScope| {
///     let in_p = jack::port::AudioInPort::new(&in_port, ps);
///     {
///         let _in_b: &[f32] = &in_p; // can deref into &[f32]
///     }
///     let _x = in_p[0];
/// };
/// ```
pub struct AudioInPort<'a> {
    _port: &'a Port<AudioInSpec>,
    buffer: &'a [f32],
}

impl<'a> AudioInPort<'a> {
    /// Wrap a `Port<AudioInSpec>` within a process scope of a client
    /// that registered the port. Panics if the port does not belong
    /// to the client that created the process.
    pub fn new(port: &'a Port<AudioInSpec>, ps: &'a ProcessScope) -> Self {
        assert_eq!(port.client_ptr(), ps.client_ptr());
        let buff = unsafe {
            slice::from_raw_parts(port.buffer(ps.n_frames()) as *const f32,
                                  ps.n_frames() as usize)
        };
        AudioInPort {
            _port: port,
            buffer: buff,
        }
    }
}

impl<'a> Deref for AudioInPort<'a> {
    type Target = [f32];

    fn deref(&self) -> &[f32] {
        self.buffer
    }
}
