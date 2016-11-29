use std::marker::PhantomData;
use std::slice;
use flags;
use flags::PortFlags;
use jack_port::Port;
use jack_port::PortSpec;
use callbacks::ProcessScope;

#[derive(Debug)]
pub struct Output;

#[derive(Debug)]
pub struct Audio<T> {
    pd: PhantomData<T>,
}

unsafe impl PortSpec for Audio<Output> {
    fn port_type() -> &'static str {
        "32 bit mono audio"
    }

    fn flags() -> PortFlags {
        flags::IS_OUTPUT
    }

    fn buffer_size() -> u64 {
        // Not needed for built in types according to jack api
        0
    }
}

pub struct AudioOutData<'a> {
    _port: &'a mut Port<Audio<Output>>,
    pub buffer: &'a mut [f32],
}

impl<'a> AudioOutData<'a> {
    pub fn get(p: &'a mut Port<Audio<Output>>, ps: &ProcessScope) -> Self {
        let buff: &mut [f32] = unsafe {
            slice::from_raw_parts_mut(p.buffer_ptr(ps.n_frames()) as *mut f32,
                                      ps.n_frames() as usize)
        };
        AudioOutData {
            _port: &mut p,
            buffer: buff,
        }
    }

    pub fn slice_mut(&mut self) -> &mut [f32] {
        self.buffer
    }
}

pub type AudioOutputSpec = Audio<Output>;

pub fn please_work(mut x: Port<Audio<Output>>, ps: &ProcessScope) {
    let y = AudioOutData::get(&mut x, ps);
    let _a = y.buffer[0];
}
