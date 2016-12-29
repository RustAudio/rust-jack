use std::slice;
use std::ops::{Deref, DerefMut};

use jack_flags::port_flags::{IS_INPUT, IS_OUTPUT, PortFlags};
use port::{Port, PortSpec};
use callbacks::ProcessScope;

/// `AudioInSpec` implements the `PortSpec` trait which, defines an
/// endpoint for JACK. In this case, it is a readable 32 bit floating
/// point buffer for audio.
///
/// `AudioInSpec::buffer()` is used to gain access the buffer.
#[derive(Debug, Default)]
pub struct AudioInSpec;

/// `AudioOutSpec` implements the `PortSpec` trait, which defines an
/// endpoint for JACK. In this case, it is a mutable 32 bit floating
/// point buffer for audio.
///
/// `AudioOutSpec::buffer()` is used to gain access the buffer.
#[derive(Debug, Default)]
pub struct AudioOutSpec;


unsafe impl<'a> PortSpec for AudioOutSpec {
    fn jack_port_type(&self) -> &'static str {
        "32 bit float mono audio"
    }

    fn jack_flags(&self) -> PortFlags {
        IS_OUTPUT
    }

    fn jack_buffer_size(&self) -> u64 {
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
        "32 bit float mono audio"
    }

    fn jack_flags(&self) -> PortFlags {
        IS_INPUT
    }

    fn jack_buffer_size(&self) -> u64 {
        // Not needed for built in types according to JACK api
        0
    }
}

/// Safetly wrap a `Port<AudioOutPort>`. Can deref into a `&mut[f32]`.
pub struct AudioOutPort<'a> {
    _port: &'a mut Port<AudioOutSpec>,
    buffer: &'a mut [f32],
}

impl<'a> AudioOutPort<'a> {
    /// Wrap a `Port<AudioOutSpec>` within a process scope of a client
    /// that registered the port. Panics if the port does not belong
    /// to the client that created the process.
    ///
    /// # Examples
    /// ```rust
    /// let out_port = client.register_port(...);
    /// let process_callback = move |ps: &jack::ProcessScope| {
    ///     let mut wrapped_port = jack::AudioOutPort::new(&mut out_port, ps);
    ///     // wrapped_port can deref into &mut[f32]
    ///     wrapped_port[0] = 1.0;
    ///     wrapped_port[1] = -1.0;
    ///     wrapped_port[2] = 0.0;
    /// };
    /// ....activate(process_callback).unwrap();
    /// ```
    pub fn new(port: &'a mut Port<AudioOutSpec>, ps: &'a ProcessScope) -> Self {
        unsafe { assert_eq!(port.client_ptr(), ps.client_ptr()) };
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


/// Safetly wrap a `Port<AudioInPort>`. Derefs into a `&[f32]`.
pub struct AudioInPort<'a> {
    _port: &'a Port<AudioInSpec>,
    buffer: &'a [f32],
}

impl<'a> AudioInPort<'a> {
    /// Wrap a `Port<AudioInSpec>` within a process scope of a client
    /// that registered the port. Panics if the port does not belong
    /// to the client that created the process.
    ///
    /// # Examples
    /// ```rust
    /// let p = client.register_port(...);
    /// let process_callback = move |ps: &jack::ProcessScope| {
    ///     let wrapped_port = jack::AudioInPort::new(&p, ps);
    ///     use std::f32;
    ///     let peak = wrapped_port
    ///         .iter()
    ///         .map(|x| x.abs())
    ///         .fold(0.0 as f32, |a, b| a.max(b));
    ///     ...
    /// };
    /// ....activate(process_callback).unwrap();
    /// ```
    pub fn new(port: &'a Port<AudioInSpec>, ps: &'a ProcessScope) -> Self {
        unsafe { assert_eq!(port.client_ptr(), ps.client_ptr()) };
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
