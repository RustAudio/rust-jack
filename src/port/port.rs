use std::marker::Sized;
use std::{ffi, iter};

use libc;

use jack_enums::JackErr;
use port::port_flags::PortFlags;
use jack_sys as j;
use primitive_types as pt;

lazy_static! {
    /// The maximum string length for port names.
    pub static ref PORT_NAME_SIZE: usize = unsafe { j::jack_port_name_size() - 1 } as usize;

    /// The maximum string length for jack type names.
    pub static ref PORT_TYPE_SIZE: usize = unsafe { j::jack_port_type_size() - 1 } as usize;
}

/// Defines the configuration for a certain port to JACK, ie 32 bit floating audio input, 8 bit raw
/// midi output, etc...
pub unsafe trait PortSpec: Default + Sized {
    /// String used by JACK upon port creation to identify the port
    /// type.
    fn jack_port_type(&self) -> &'static str;

    /// Flags used by jack upon port creation.
    fn jack_flags(&self) -> PortFlags;

    /// Size used by jack upon port creation.
    fn jack_buffer_size(&self) -> libc::c_ulong;
}

/// An endpoint to interact with JACK data streams, for audio, midi,
/// etc...
///
/// Most JACK functionality is exposed, including the raw pointers, but it should be possible to
/// create a client without the need for calling `unsafe` `Port` methods.
#[derive(Debug)]
pub struct Port<PS: PortSpec> {
    spec: PS,
    client_ptr: *mut j::jack_client_t,
    port_ptr: *mut j::jack_port_t,
}

unsafe impl<PS: PortSpec> Send for Port<PS> {}
unsafe impl<PS: PortSpec> Sync for Port<PS> {}

impl<PS: PortSpec> Port<PS> {
    /// Returns the spec that was used to create this port.
    pub fn spec(&self) -> &PS {
        &self.spec
    }

    /// Return a copy of port as an unowned port that can still be used for querying information.
    pub fn clone_unowned(&self) -> Port<Unowned> {
        Port {
            spec: Unowned,
            client_ptr: self.client_ptr(),
            port_ptr: self.as_ptr(),
        }
    }

    /// Returns the full name of the port, including the "client_name:" prefix.
    pub fn name<'a>(&'a self) -> &'a str {
        unsafe { ffi::CStr::from_ptr(j::jack_port_name(self.as_ptr())).to_str().unwrap() }
    }

    /// Returns the short name of the port, it excludes the "client_name:" prefix.
    pub fn short_name<'a>(&'a self) -> &'a str {
        unsafe { ffi::CStr::from_ptr(j::jack_port_short_name(self.as_ptr())).to_str().unwrap() }
    }

    /// The flags for the port. These are set when the port is registered with
    /// its client.
    pub fn flags(&self) -> PortFlags {
        let bits = unsafe { j::jack_port_flags(self.as_ptr()) };
        PortFlags::from_bits(bits as j::Enum_JackPortFlags).unwrap()
    }

    /// The port type. JACK's built in types include `"32 bit float mono audio`" and `"8 bit raw
    /// midi"`. Custom types may also be used.
    pub fn port_type<'a>(&self) -> &'a str {
        unsafe { ffi::CStr::from_ptr(j::jack_port_type(self.as_ptr())).to_str().unwrap() }
    }

    /// Number of ports connected to/from `&self`.
    pub fn connected_count(&self) -> usize {
        let n = unsafe { j::jack_port_connected(self.as_ptr()) };
        n as usize
    }

    /// Returns `true` if the port is directly connected to a port with the name `port_name`.
    pub fn is_connected_to(&self, port_name: &str) -> bool {
        let res = unsafe {
            let port_name = ffi::CString::new(port_name).unwrap();
            j::jack_port_connected_to(self.as_ptr(), port_name.as_ptr())
        };
        match res {
            0 => false,
            _ => true,
        }
    }

    /// Remove connections to/from port `self`.
    pub fn disconnect(&self) -> Result<(), JackErr> {
        let res = unsafe { j::jack_port_disconnect(self.client_ptr(), self.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortDisconnectionError),
        }
    }

    /// Get the alias names for `self`.
    ///
    /// Will return a vector of strings of up to 2 elements.
    pub fn aliases(&self) -> Vec<String> {
        let mut a: Vec<i8> = iter::repeat(0).take(*PORT_NAME_SIZE + 1).collect();
        let mut b = a.clone();
        unsafe {
            let mut ptrs: [*mut i8; 2] = [a.as_mut_ptr(), b.as_mut_ptr()];
            j::jack_port_get_aliases(self.as_ptr(), ptrs.as_mut_ptr());
        };
        [a, b]
            .iter()
            .map(|p| p.as_ptr())
            .map(|p| unsafe { ffi::CStr::from_ptr(p).to_string_lossy().into_owned() })
            .filter(|s| s.len() > 0)
            .collect()
    }

    /// Returns `true` if monitoring has been requested for `self`.
    pub fn is_monitoring_input(&self) -> bool {
        match unsafe { j::jack_port_monitoring_input(self.as_ptr()) } {
            0 => false,
            _ => true,
        }
    }

    /// Turn input monitoring for the port on or off.
    ///
    /// This only works if the port has the `CAN_MONITOR` flag set.
    pub fn request_monitor(&self, enable_monitor: bool) -> Result<(), JackErr> {
        let onoff = match enable_monitor {
            true => 1,
            false => 0,
        };
        let res = unsafe { j::jack_port_request_monitor(self.as_ptr(), onoff) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortMonitorError),
        }
    }

    /// If the `CAN_MONITOR` flag is set for the port, then input monitoring is turned on if it was
    /// off, and turns it off if only one request has been made to turn it on. Otherwise it does
    /// nothing.
    pub fn ensure_monitor(&self, enable_monitor: bool) -> Result<(), JackErr> {
        let onoff = match enable_monitor {
            true => 1,
            false => 0,
        };
        let res = unsafe { j::jack_port_ensure_monitor(self.as_ptr(), onoff) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortMonitorError),
        }
    }

    /// Set's the short name of the port. If the full name is longer than `PORT_NAME_SIZE`, then it
    /// will be truncated.
    pub fn set_name(&mut self, short_name: &str) -> Result<(), JackErr> {
        let short_name = ffi::CString::new(short_name).unwrap();
        let res = unsafe { j::jack_port_set_name(self.as_ptr(), short_name.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortNamingError),
        }
    }

    /// Sets `alias` as an alias for `self`.
    ///
    /// May be called at any time. If the alias is longer than `PORT_NAME_SIZE`, it will be
    /// truncated.
    ///
    /// After a successful call, and until JACK exists, or the alias is unset, `alias` may be used
    /// as an alternate name for the port.
    ///
    /// Ports can have up to two aliases - if both are already set, this function will return an
    /// error.
    pub fn set_alias(&mut self, alias: &str) -> Result<(), JackErr> {
        let alias = ffi::CString::new(alias).unwrap();
        let res = unsafe { j::jack_port_set_alias(self.as_ptr(), alias.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortAliasError),
        }
    }

    /// Remove `alias` as an alias for port. May be called at any time.
    ///
    /// After a successful call, `alias` can no longer be used as an alternate name for `self`.
    pub fn unset_alias(&mut self, alias: &str) -> Result<(), JackErr> {
        let alias = ffi::CString::new(alias).unwrap();
        let res = unsafe { j::jack_port_unset_alias(self.as_ptr(), alias.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortAliasError),
        }
    }

    /// Remove the port from the client, disconnecting any existing connections.  The port must have
    /// been created with the provided client.
    pub fn unregister(self) -> Result<(), JackErr> {
        let res = unsafe { j::jack_port_unregister(self.client_ptr, self.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortDisconnectionError),
        }
    }

    /// Create a Port from raw JACK pointers.
    ///
    /// This is mostly for use within the jack crate itself.
    pub unsafe fn from_raw(spec: PS,
                           client_ptr: *mut j::jack_client_t,
                           port_ptr: *mut j::jack_port_t)
                           -> Self {
        Port {
            spec: spec,
            port_ptr: port_ptr,
            client_ptr: client_ptr,
        }
    }

    /// Obtain the client pointer that spawned this port.
    ///
    /// This is mostly for use within the jack crate itself.
    #[inline(always)]
    pub fn client_ptr(&self) -> *mut j::jack_client_t {
        self.client_ptr
    }

    /// Obtain the ffi port pointer.
    ///
    /// This is mostly for use within the jack crate itself.
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut j::jack_port_t {
        self.port_ptr
    }

    /// Obtain the buffer that the Port is holding. For standard audio and midi ports, consider
    /// using the `AudioInPort`, `AudioOutPort`, `MidiInPort`, or `MidiOutPort` adapter. For more
    /// custom data, consider implementing your own adapter that safely uses the `Port::buffer`
    /// method.
    #[inline(always)]
    pub unsafe fn buffer(&self, n_frames: pt::JackFrames) -> *mut libc::c_void {
        j::jack_port_get_buffer(self.port_ptr, n_frames)
    }
}

/// `PortSpec` for a port that holds no readable or write-able stream data from JACK, though it can
/// be used for obtaining information about external ports.
#[derive(Debug, Default)]
pub struct Unowned;

/// `Port<UnownedSpec>` - Port that holds no data from Jack, though it can still be used to query
/// information.
pub type UnownedPort = Port<Unowned>;

unsafe impl PortSpec for Unowned {
    /// Panics on call since the `Unowned` spec can't be used to create ports.
    fn jack_port_type(&self) -> &'static str {
        unreachable!()
    }

    /// Panics on call since the `Unowned` spec can't be used to create ports.
    fn jack_flags(&self) -> PortFlags {
        unreachable!()
    }

    /// Panics on call since the `Unowned` spec can't be used to create ports.
    fn jack_buffer_size(&self) -> libc::c_ulong {
        unreachable!()
    }
}
