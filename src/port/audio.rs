use jack_sys as j;
use std::slice;

use crate::{Port, PortFlags, PortSpec, ProcessScope};

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

unsafe impl PortSpec for AudioOut {
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
        unsafe {
            slice::from_raw_parts(
                self.buffer(ps.n_frames()) as *const f32,
                ps.n_frames() as usize,
            )
        }
    }
}

impl Port<AudioOut> {
    /// Get a slice to write audio data to.
    pub fn as_mut_slice<'a>(&'a mut self, ps: &'a ProcessScope) -> &'a mut [f32] {
        assert_eq!(self.client_ptr(), ps.client_ptr());
        unsafe {
            slice::from_raw_parts_mut(
                self.buffer(ps.n_frames()) as *mut f32,
                ps.n_frames() as usize,
            )
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Client, ClientOptions, ClosureProcessHandler, Control};

    fn open_test_client(name: &str) -> Client {
        Client::new(name, ClientOptions::NO_START_SERVER).unwrap().0
    }

    #[test]
    fn port_audio_can_read_write() {
        let c = open_test_client("port_audio_crw");
        let in_a = c.register_port("ia", AudioIn).unwrap();
        let in_b = c.register_port("ib", AudioIn).unwrap();
        let mut out_a = c.register_port("oa", AudioOut).unwrap();
        let mut out_b = c.register_port("ob", AudioOut).unwrap();
        let (success_sender, success_receiver) = std::sync::mpsc::sync_channel(1);
        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
            let exp_a = 0.312_443;
            let exp_b = -0.612_120;
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
            if in_a.iter().all(|v| (*v - exp_a).abs() < 1E-5)
                && in_b.iter().all(|v| (*v - exp_b).abs() < 1E-5)
            {
                let _ = success_sender.send(true);
            }
            Control::Continue
        };
        let ac = c
            .activate_async((), ClosureProcessHandler::new(process_callback))
            .unwrap();
        ac.as_client()
            .connect_ports_by_name("port_audio_crw:oa", "port_audio_crw:ia")
            .unwrap();
        ac.as_client()
            .connect_ports_by_name("port_audio_crw:ob", "port_audio_crw:ib")
            .unwrap();
        assert_eq!(
            success_receiver
                .recv_timeout(std::time::Duration::from_secs(2))
                .unwrap(),
            true
        );
        ac.deactivate().unwrap();
    }
}
