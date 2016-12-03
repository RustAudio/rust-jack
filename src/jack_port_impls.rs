use std::slice;
use flags;
use flags::PortFlags;
use jack_port::PortData;

#[derive(Debug, Default)]
pub struct AudioIn<'a> {
    buff: &'a [f32],
}

#[derive(Debug, Default)]
pub struct AudioOut<'a> {
    buff: &'a mut [f32],
}

unsafe impl<'a> PortData for AudioOut<'a> {
    unsafe fn from_ptr(ptr: *mut ::libc::c_void, nframes: u32) -> Self {
        let len = nframes as usize;
        let buff = slice::from_raw_parts_mut(ptr as *mut f32, len);
        AudioOut { buff: buff }
    }

    fn jack_port_type() -> &'static str {
        "32 bit mono audio"
    }

    fn jack_flags() -> PortFlags {
        flags::IS_OUTPUT
    }

    fn jack_buffer_size() -> u64 {
        // Not needed for built in types according to jack api
        0
    }
}

impl<'a> AudioOut<'a> {
    pub fn buffer(&mut self) -> &mut [f32] {
        return self.buff;
    }
}

unsafe impl<'a> PortData for AudioIn<'a> {
    unsafe fn from_ptr(ptr: *mut ::libc::c_void, nframes: u32) -> Self {
        let len = nframes as usize;
        let buff = slice::from_raw_parts(ptr as *const f32, len);
        AudioIn { buff: buff }
    }
    fn jack_port_type() -> &'static str {
        "32 bit mono audio"
    }

    fn jack_flags() -> PortFlags {
        flags::IS_INPUT
    }

    fn jack_buffer_size() -> u64 {
        // Not needed for built in types according to jack api
        0
    }
}

impl<'a> AudioIn<'a> {
    pub fn buffer(&self) -> &[f32] {
        self.buff
    }
}
