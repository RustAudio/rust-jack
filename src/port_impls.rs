use std::slice;
use jack_flags::port_flags::{IS_INPUT, IS_OUTPUT, PortFlags};
use port::PortData;

/// `AudioIn` implements the `PortData` trait which, defines an
/// endpoint for JACK. In this case, it is a readable 32 bit floating
/// point buffer for audio.
///
/// `AudioIn::buffer()` is used to gain access the buffer.
#[derive(Debug)]
pub struct AudioIn<'a> {
    buff: &'a [f32],
}

/// `AudioOut` implements the `PortData` trait, which defines an
/// endpoint for JACK. In this case, it is a mutable 32 bit floating
/// point buffer for audio.
///
/// `AudioOut::buffer()` is used to gain access the buffer.
#[derive(Debug)]
pub struct AudioOut<'a> {
    buff: &'a mut [f32],
}

unsafe impl<'a> PortData for AudioOut<'a> {
    /// Create an AudioOut instance from a buffer pointer and frame
    /// count. This is mostly used by `Port<AudioOut>` within a
    /// `process` scope.
    ///
    /// # Arguments
    ///
    /// * `ptr` - buffer pointer to underlying data.
    ///
    /// * `nframes` - the size of the buffer.
    unsafe fn from_ptr(ptr: *mut ::libc::c_void, nframes: u32) -> Self {
        let len = nframes as usize;
        let buff = slice::from_raw_parts_mut(ptr as *mut f32, len);
        AudioOut { buff: buff }
    }

    fn jack_port_type() -> &'static str {
        "32 bit float mono audio"
    }

    fn jack_flags() -> PortFlags {
        IS_OUTPUT
    }

    fn jack_buffer_size() -> u64 {
        // Not needed for built in types according to JACK api
        0
    }
}

impl<'a> AudioOut<'a> {
    /// Retrieve the underlying buffer for reading.
    pub fn buffer(&mut self) -> &mut [f32] {
        return self.buff;
    }
}

unsafe impl<'a> PortData for AudioIn<'a> {
    /// Create an AudioIn instance from a buffer pointer and frame
    /// count. This is mostly used by `Port<AudioIn>` within a
    /// `process` scope.
    ///
    /// # Arguments
    ///
    /// * `ptr` - buffer pointer to underlying data.
    ///
    /// * `nframes` - the size of the buffer.
    unsafe fn from_ptr(ptr: *mut ::libc::c_void, nframes: u32) -> Self {
        let len = nframes as usize;
        let buff = slice::from_raw_parts(ptr as *const f32, len);
        AudioIn { buff: buff }
    }

    fn jack_port_type() -> &'static str {
        "32 bit float mono audio"
    }

    fn jack_flags() -> PortFlags {
        IS_INPUT
    }

    fn jack_buffer_size() -> u64 {
        // Not needed for built in types according to JACK api
        0
    }
}

impl<'a> AudioIn<'a> {
    /// Retrieve the underlying buffer for writing purposes.
    pub fn buffer(&self) -> &[f32] {
        self.buff
    }
}
