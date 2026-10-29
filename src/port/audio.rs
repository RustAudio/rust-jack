use jack_sys as j;
use std::slice;

use crate::{Port, PortFlags, PortSpec, ProcessScope};

/// [`AudioIn`] implements the [`PortSpec`] trait which, defines an
/// endpoint for JACK. In this case, it is a readable 32 bit floating
/// point buffer for audio.
///
/// [`Port::as_slice()`] is used to gain access the buffer.
///
/// # Example
/// ```
/// let client = jack::Client::new("rusty_client", jack::ClientOptions::default())
///     .unwrap()
///     .0;
/// let spec = jack::AudioIn::default();
/// let audio_in_port = client.register_port("in", spec).unwrap();
/// ```
#[derive(Copy, Clone, Debug, Default)]
pub struct AudioIn {
    _internal: (),
}

/// [`AudioOut`] implements the [`PortSpec`] trait, which defines an
/// endpoint for JACK. In this case, it is a mutable 32 bit floating
/// point buffer for audio.
///
/// [`Port::as_mut_slice()`] is used to gain access the buffer.
///
/// # Example
/// ```
/// let client = jack::Client::new("rusty_client", jack::ClientOptions::default())
///     .unwrap()
///     .0;
/// let spec = jack::AudioOut::default();
/// let audio_out_port = client.register_port("out", spec).unwrap();
/// ```
#[derive(Copy, Clone, Debug, Default)]
pub struct AudioOut {
    _internal: (),
}

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
    ///
    /// # Aliasing
    ///
    /// A JACK server may hand out the same memory for an input port and the output port it is
    /// connected to. If a client connects one of its own `AudioOut` ports to one of its own
    /// `AudioIn` ports, the slices from [`Port<AudioIn>::as_slice`] and
    /// [`Port<AudioOut>::as_mut_slice`] could then overlap, which breaks Rust's aliasing rules. Do
    /// not connect a client's output ports to its own input ports while holding both slices.
    ///
    /// This depends on the server, and more research is needed to say which ones are affected.
    /// From reading the source, JACK2 passes an output buffer straight through to an input port
    /// with one connection, but copies it when both ports belong to the same client, so it does not
    /// alias in this case. PipeWire's JACK layer returns the peer's buffer for a single connection
    /// without a same-client check, so it may alias, but this has not been verified at runtime.
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
    ///
    /// # Aliasing
    ///
    /// A JACK server may hand out the same memory for an input port and the output port it is
    /// connected to. If a client connects one of its own `AudioOut` ports to one of its own
    /// `AudioIn` ports, the slices from [`Port<AudioIn>::as_slice`] and
    /// [`Port<AudioOut>::as_mut_slice`] could then overlap, which breaks Rust's aliasing rules. Do
    /// not connect a client's output ports to its own input ports while holding both slices.
    ///
    /// This depends on the server, and more research is needed to say which ones are affected.
    /// From reading the source, JACK2 passes an output buffer straight through to an input port
    /// with one connection, but copies it when both ports belong to the same client, so it does not
    /// alias in this case. PipeWire's JACK layer returns the peer's buffer for a single connection
    /// without a same-client check, so it may alias, but this has not been verified at runtime.
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
