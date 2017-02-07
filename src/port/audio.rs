// The MIT License (MIT)
//
// Copyright (c) 2017 Will Medrano (will.s.medrano@gmail.com)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

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

/// Safetly wrap a `Port<AudioOutSpec>`. Derefs into a `&mut[f32]`.
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


/// Safetly wrap a `Port<AudioInSpec>`. Derefs into a `&[f32]`.
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
