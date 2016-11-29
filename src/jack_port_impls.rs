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
pub struct Input;

#[derive(Debug)]
pub struct Audio<T> {
    pd: PhantomData<T>,
}

pub type AudioInputSpec = Audio<Input>;
pub type AudioOutputSpec = Audio<Output>;

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

unsafe impl PortSpec for Audio<Input> {
    fn port_type() -> &'static str {
        "32 bit mono audio"
    }

    fn flags() -> PortFlags {
        flags::IS_INPUT
    }

    fn buffer_size() -> u64 {
        // Not needed for built in types according to jack api
        0
    }
}

pub struct AudioOutData<'a> {
    _port: &'a Port<Audio<Output>>,
    pub buffer: &'a mut [f32],
}

pub struct AudioInData<'a> {
    _port: &'a Port<Audio<Input>>,
    pub buffer: &'a [f32],
}

impl<'a> AudioOutData<'a> {
    pub fn get(p: &'a Port<Audio<Output>>, ps: &ProcessScope) -> Self {
        let buff: &mut [f32] = unsafe {
            slice::from_raw_parts_mut(p.buffer_ptr(ps.n_frames()) as *mut f32,
                                      ps.n_frames() as usize)
        };
        AudioOutData {
            _port: &p,
            buffer: buff,
        }
    }
}

impl<'a> AudioInData<'a> {
    pub fn get(p: &'a Port<Audio<Input>>, ps: &ProcessScope) -> Self {
        let buff: &[f32] = unsafe {
            slice::from_raw_parts(p.buffer_ptr(ps.n_frames()) as *mut f32,
                                  ps.n_frames() as usize)
        };
        AudioInData {
            _port: &p,
            buffer: buff,
        }
    }
}

pub fn please_dont_work(p: Port<Audio<Output>>, ps: &ProcessScope) {
    let x = AudioOutData::get(&p, ps);
    let y = AudioOutData::get(&p, ps);
    x.buffer[0] = 0.0;
    y.buffer[0] = 0.1;
}

pub fn please_work(p: Port<AudioOutputSpec>, ps: &ProcessScope) {
    let x = AudioOutData::get(&p, ps);
    x.buffer[0] = 0.0;
}
