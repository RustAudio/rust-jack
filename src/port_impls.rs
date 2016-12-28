use std::slice;
use std::ops::{Deref, DerefMut};

use jack_flags::port_flags::{IS_INPUT, IS_OUTPUT, PortFlags};
use port::{Port, PortSpec};
use callbacks::ProcessScope;

/// `AudioIn` implements the `PortSpec` trait which, defines an
/// endpoint for JACK. In this case, it is a readable 32 bit floating
/// point buffer for audio.
///
/// `AudioIn::buffer()` is used to gain access the buffer.
#[derive(Debug)]
pub struct AudioIn;

/// `AudioOut` implements the `PortSpec` trait, which defines an
/// endpoint for JACK. In this case, it is a mutable 32 bit floating
/// point buffer for audio.
///
/// `AudioOut::buffer()` is used to gain access the buffer.
#[derive(Debug)]
pub struct AudioOut;


unsafe impl<'a> PortSpec for AudioOut {
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

unsafe impl PortSpec for AudioIn {
    /// Create an AudioIn instance from a buffer pointer and frame
    /// count. This is mostly used by `Port<AudioIn>` within a
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

pub struct AudioOutPort<'a> {
    _port: &'a mut Port<AudioOut>,
    buffer: &'a mut [f32],
}

impl<'a> AudioOutPort<'a> {
    pub fn new(port: &'a mut Port<AudioOut>, ps: &'a ProcessScope) -> Self {
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


pub struct AudioInPort<'a> {
    _port: &'a Port<AudioIn>,
    buffer: &'a [f32],
}

impl<'a> AudioInPort<'a> {
    pub fn new(port: &'a Port<AudioIn>, ps: &'a ProcessScope) -> Self {
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
