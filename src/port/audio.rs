use std::ops::{Deref, DerefMut};
use std::slice;

use jack_sys as j;
use libc;

use client::ProcessScope;
use port::{Port, PortSpec};
use port::port_flags::{PortFlags, IS_INPUT, IS_OUTPUT};

/// `AudioInSpec` implements the `PortSpec` trait which, defines an
/// endpoint for JACK. In this case, it is a readable 32 bit floating
/// point buffer for audio.
///
/// `AudioInSpec::buffer()` is used to gain access the buffer.
///
/// # Example
/// ```
/// let client = jack::client::Client::new(
///     "rusty_client",
///     jack::client::client_options::NO_START_SERVER,
/// ).unwrap()
///     .0;
/// let spec = jack::port::AudioInSpec::default();
/// let audio_in_port = client.register_port("in", spec).unwrap();
/// ```
#[derive(Copy, Clone, Debug, Default)]
pub struct AudioInSpec;

/// `AudioOutSpec` implements the `PortSpec` trait, which defines an
/// endpoint for JACK. In this case, it is a mutable 32 bit floating
/// point buffer for audio.
///
/// `AudioOutSpec::buffer()` is used to gain access the buffer.
///
/// # Example
/// ```
/// let client = jack::client::Client::new(
///     "rusty_client",
///     jack::client::client_options::NO_START_SERVER,
/// ).unwrap()
///     .0;
/// let spec = jack::port::AudioInSpec::default();
/// let audio_out_port = client.register_port("out", spec).unwrap();
/// ```
#[derive(Copy, Clone, Debug, Default)]
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

/// Safely and thinly wrap a `Port<AudioOutSpec>`. Derefs into a `&mut[f32]`.
///
/// # Example
/// ```
/// let client = jack::client::Client::new("c", jack::client::client_options::NO_START_SERVER)
///     .unwrap()
///     .0;
/// let mut out_port = client
///     .register_port("p", jack::port::AudioOutSpec::default())
///     .unwrap();
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
            slice::from_raw_parts_mut(
                port.buffer(ps.n_frames()) as *mut f32,
                ps.n_frames() as usize,
            )
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

/// Safely and thinly wrap a `Port<AudioInSpec>`. Derefs into a `&[f32]`.
///
/// # Example
/// ```
/// let client = jack::client::Client::new("c", jack::client::client_options::NO_START_SERVER)
///     .unwrap()
///     .0;
/// let in_port = client
///     .register_port("p", jack::port::AudioInSpec::default())
///     .unwrap();
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
            slice::from_raw_parts(
                port.buffer(ps.n_frames()) as *const f32,
                ps.n_frames() as usize,
            )
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

#[cfg(test)]
mod test {
    use super::*;
    use client::AsyncClient;
    use client::Client;
    use client::ClosureProcessHandler;
    use client::client_options;
    use jack_enums::Control;
    use std::sync::mpsc::channel;

    fn open_test_client(name: &str) -> Client {
        Client::new(name, client_options::NO_START_SERVER)
            .unwrap()
            .0
    }

    #[test]
    fn port_audio_can_read_write() {
        let c = open_test_client("port_audio_crw");
        let in_a = c.register_port("ia", AudioInSpec::default()).unwrap();
        let in_b = c.register_port("ib", AudioInSpec::default()).unwrap();
        let mut out_a = c.register_port("oa", AudioOutSpec::default()).unwrap();
        let mut out_b = c.register_port("ob", AudioOutSpec::default()).unwrap();
        let (signal_succeed, did_succeed) = channel();
        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
            let exp_a = 0.31244352;
            let exp_b = -0.61212;
            let in_a = AudioInPort::new(&in_a, ps);
            let in_b = AudioInPort::new(&in_b, ps);
            let mut out_a = AudioOutPort::new(&mut out_a, ps);
            let mut out_b = AudioOutPort::new(&mut out_b, ps);
            for v in out_a.iter_mut() {
                *v = exp_a;
            }
            for v in out_b.iter_mut() {
                *v = exp_b;
            }
            if in_a.iter().all(|v| *v == exp_a) && in_b.iter().all(|v| *v == exp_b) {
                let s = signal_succeed.clone();
                let _ = s.send(true);
            }
            Control::Continue
        };
        let ac = AsyncClient::new(c, (), ClosureProcessHandler::new(process_callback)).unwrap();
        ac.as_client()
            .connect_ports_by_name("port_audio_crw:oa", "port_audio_crw:ia")
            .unwrap();
        ac.as_client()
            .connect_ports_by_name("port_audio_crw:ob", "port_audio_crw:ib")
            .unwrap();
        assert!(
            did_succeed.iter().any(|b| b),
            "input port does not have expected data"
        );
        ac.deactivate().unwrap();
    }
}
