use jack_sys as j;
use libc;
use std::slice;

use client::ProcessScope;
use port::{Port, PortSpec};
use port::port_flags::PortFlags;

/// `AudioIn` implements the `PortSpec` trait which, defines an
/// endpoint for JACK. In this case, it is a readable 32 bit floating
/// point buffer for audio.
///
/// `AudioIn::buffer()` is used to gain access the buffer.
///
/// # Example
/// ```
/// let client = jack::Client::new("rusty_client", jack::ClientOptions::NO_START_SERVER)
///     .unwrap()
///     .0;
/// let spec = jack::AudioIn::default();
/// let audio_in_port = client.register_port("in", spec).unwrap();
/// ```
#[derive(Copy, Clone, Debug, Default)]
pub struct AudioIn;

/// `AudioOut` implements the `PortSpec` trait, which defines an
/// endpoint for JACK. In this case, it is a mutable 32 bit floating
/// point buffer for audio.
///
/// `AudioOut::buffer()` is used to gain access the buffer.
///
/// # Example
/// ```
/// let client = jack::Client::new("rusty_client", jack::ClientOptions::NO_START_SERVER)
///     .unwrap()
///     .0;
/// let spec = jack::AudioIn::default();
/// let audio_out_port = client.register_port("out", spec).unwrap();
/// ```
#[derive(Copy, Clone, Debug, Default)]
pub struct AudioOut;

unsafe impl<'a> PortSpec for AudioOut {
    fn jack_port_type(&self) -> &'static str {
        j::FLOAT_MONO_AUDIO
    }

    fn jack_flags(&self) -> PortFlags {
        PortFlags::IS_OUTPUT
    }

    fn jack_buffer_size(&self) -> libc::c_ulong {
        // Not needed for built in types according to JACK api
        0
    }
}

unsafe impl PortSpec for AudioIn {
    fn jack_port_type(&self) -> &'static str {
        j::FLOAT_MONO_AUDIO
    }

    fn jack_flags(&self) -> PortFlags {
        PortFlags::IS_INPUT
    }

    fn jack_buffer_size(&self) -> libc::c_ulong {
        // Not needed for built in types according to JACK api
        0
    }
}

impl Port<AudioIn> {
    /// Read the received audio data.
    pub fn as_slice<'a>(&'a self, ps: &'a ProcessScope) -> &'a [f32] {
        assert_eq!(self.client_ptr(), ps.client_ptr());
        let buff = unsafe {
            slice::from_raw_parts(
                self.buffer(ps.n_frames()) as *const f32,
                ps.n_frames() as usize,
            )
        };
        buff
    }
}

impl Port<AudioOut> {
    /// Get a slice to write audio data to.
    pub fn as_mut_slice<'a>(&'a mut self, ps: &'a ProcessScope) -> &'a mut [f32] {
        assert_eq!(self.client_ptr(), ps.client_ptr());
        let buff = unsafe {
            slice::from_raw_parts_mut(
                self.buffer(ps.n_frames()) as *mut f32,
                ps.n_frames() as usize,
            )
        };
        buff
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use client::Client;
    use client::ClosureProcessHandler;
    use jack_enums::Control;
    use std::sync::mpsc::channel;

    fn open_test_client(name: &str) -> Client {
        Client::new(name, ClientOptions::NO_START_SERVER).unwrap().0
    }

    #[test]
    fn port_audio_can_read_write() {
        let c = open_test_client("port_audio_crw");
        let in_a = c.register_port("ia", AudioIn::default()).unwrap();
        let in_b = c.register_port("ib", AudioIn::default()).unwrap();
        let mut out_a = c.register_port("oa", AudioOut::default()).unwrap();
        let mut out_b = c.register_port("ob", AudioOut::default()).unwrap();
        let (signal_succeed, did_succeed) = channel();
        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
            let exp_a = 0.31244352;
            let exp_b = -0.61212;
            let in_a = in_a.as_slice(ps);
            let in_b = in_b.as_slice(ps);
            let out_a = out_a.as_mut_slice(ps);
            let out_b = out_b.as_mut_slice(ps);
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
        let ac = c.activate_async((), ClosureProcessHandler::new(process_callback))
            .unwrap();
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
