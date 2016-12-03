use std::marker::Sized;
use std::ffi;
use jack_flags::PortFlags;
use jack_sys as j;
use jack_enums::JackErr;
use callbacks::ProcessScope;

lazy_static! {
    pub static ref PORT_NAME_SIZE: usize = unsafe { j::jack_port_name_size() - 1 } as usize;
    pub static ref PORT_TYPE_SIZE: usize = unsafe { j::jack_port_type_size() - 1 } as usize;
}

pub unsafe trait PortData: Sized {
    unsafe fn from_ptr(ptr: *mut ::libc::c_void, nframes: u32) -> Self;
    fn jack_port_type() -> &'static str;
    fn jack_flags() -> PortFlags;
    fn jack_buffer_size() -> u64;
}

#[derive(Debug)]
pub struct Port<PD: PortData> {
    port_data: Option<PD>,
    client_ptr: *mut j::jack_client_t,
    port_ptr: *mut j::jack_port_t,
}

unsafe impl<PD: PortData> Send for Port<PD> {}

impl<PD: PortData> Port<PD> {
    /// Returns the data
    pub fn data(&mut self, ps: &ProcessScope) -> &mut PD {
        assert!(self.client_ptr == ps.client_ptr(),
                "Port data may only be obtained for within the process of the client that \
                 created it.");
        let n = ps.n_frames();
        let ptr = unsafe { j::jack_port_get_buffer(self.port_ptr(), n) };
        self.port_data = Some(unsafe { PD::from_ptr(ptr, n) });
        self.port_data.as_mut().unwrap()
    }

    /// Returns the full name of the port, including the "client_name:" prefix.
    pub fn name<'a>(&'a self) -> &'a str {
        unsafe { ffi::CStr::from_ptr(j::jack_port_name(self.port_ptr)).to_str().unwrap() }
    }

    /// Returns the short name of the port, it excludes the "client_name:" prefix.
    pub fn short_name<'a>(&'a self) -> &'a str {
        unsafe { ffi::CStr::from_ptr(j::jack_port_short_name(self.port_ptr)).to_str().unwrap() }
    }

    /// Returns the uuid of the port as a u64.
    pub fn uuid(&self) -> u64 {
        unsafe { j::jack_port_uuid(self.port_ptr) }
    }

    /// The flags for the port. These are set when the port is registered with
    /// its client.
    pub fn flags(&self) -> PortFlags {
        let bits = unsafe { j::jack_port_flags(self.port_ptr) };
        PortFlags::from_bits(bits as u32).unwrap()
    }

    /// The port type. Jack's built in types include "32 bit float mono audio"
    /// and "8 bit raw midi". Custom types may also be used.
    pub fn port_type<'a>(&self) -> &'a str {
        unsafe { ffi::CStr::from_ptr(j::jack_port_type(self.port_ptr)).to_str().unwrap() }
    }

    /// Number of ports connected to/from
    pub fn connected_count(&self) -> usize {
        let n = unsafe { j::jack_port_connected(self.port_ptr) };
        n as usize
    }

    /// Returns `true` if the port is directly connected to a port with the name
    /// `port_name`.
    pub fn is_connected_to(&self, port_name: &str) -> bool {
        let res = unsafe {
            let port_name = ffi::CString::new(port_name).unwrap();
            j::jack_port_connected_to(self.port_ptr, port_name.as_ptr())
        };
        match res {
            0 => false,
            _ => true,
        }
    }

    /// Get the alias names for `self`.
    ///
    /// Will return a vector of strings of up to 2 elements.
    ///
    /// # TODO: Implement
    pub fn aliases(&self) -> Vec<String> {
        unimplemented!();
    }

    /// Returns `true` if monitoring has been requested for `self`.
    pub fn is_monitoring_input(&self) -> bool {
        match unsafe { j::jack_port_monitoring_input(self.port_ptr) } {
            0 => false,
            _ => true,
        }
    }

    /// If the `CAN_MONITOR` flag is set for the port, then input monitoring is
    /// turned on if it was off, and turns it off if only one request has been
    /// made to turn it on. Otherwise it does nothing.
    pub fn ensure_monitor(&self, enable_monitor: bool) -> Result<(), JackErr> {
        let onoff = match enable_monitor {
            true => 1,
            false => 0,
        };
        let res = unsafe { j::jack_port_ensure_monitor(self.port_ptr, onoff) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortMonitorError),
        }
    }

    /// Set's the short name of the port. If the full name is longer than
    /// `Port::name_size()`, then it will be truncated.
    pub fn set_name(&mut self, short_name: &str) -> Result<(), JackErr> {
        let short_name = ffi::CString::new(short_name).unwrap();
        let res = unsafe { j::jack_port_set_name(self.port_ptr, short_name.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortNamingError),
        }
    }

    /// Sets `alias` as an alias for `self`.
    ///
    /// May be called at any time. If the alias is longer than
    /// `Client::name_size()`, it will be truncated.
    ///
    /// After a successful call, and until Jack exists, or the alias is unset,
    /// `alias` may be used as an alternate name for the port.
    ///
    /// Ports can have up to two aliases - if both are already set, this
    /// function will return an error.
    pub fn set_alias(&mut self, alias: &str) -> Result<(), JackErr> {
        let alias = ffi::CString::new(alias).unwrap();
        let res = unsafe { j::jack_port_set_alias(self.port_ptr, alias.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortAliasError),
        }
    }

    /// Remove `alias` as an alias for port. May be called at any time.
    ///
    /// After a successful call, `alias` can no longer be used as an alternate
    /// name for `self`.
    pub fn unset_alias(&mut self, alias: &str) -> Result<(), JackErr> {
        let alias = ffi::CString::new(alias).unwrap();
        let res = unsafe { j::jack_port_unset_alias(self.port_ptr, alias.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortAliasError),
        }
    }

    /// Turn input monitoring for the port on or off.
    ///
    /// This only works if the port has the `CAN_MONITOR` flag set.
    pub fn request_monitor(&mut self, enable_monitor: bool) -> Result<(), JackErr> {
        let onoff = match enable_monitor {
            true => 1,
            false => 0,
        };
        let res = unsafe { j::jack_port_request_monitor(self.port_ptr, onoff) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortMonitorError),
        }
    }

    /// Remvoe the port from the client, disconnecting any existing connections.
    /// THe port must have been created with the provided client.
    pub fn unregister(self) -> Result<(), JackErr> {
        let res = unsafe { j::jack_port_unregister(self.client_ptr, self.port_ptr) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortDisconnectionError),
        }
    }

    pub unsafe fn from_raw(client_ptr: *mut j::jack_client_t,
                           port_ptr: *mut j::jack_port_t)
                           -> Self {
        Port {
            port_data: None,
            port_ptr: port_ptr,
            client_ptr: client_ptr,
        }
    }

    pub unsafe fn client_ptr(&self) -> *mut j::jack_client_t {
        self.client_ptr
    }

    pub unsafe fn port_ptr(&self) -> *mut j::jack_port_t {
        self.port_ptr
    }
}

#[derive(Debug)]
pub struct Unowned;

unsafe impl PortData for Unowned {
    unsafe fn from_ptr(_ptr: *mut ::libc::c_void, _nframes: u32) -> Self {
        unimplemented!()
    }

    fn jack_port_type() -> &'static str {
        unreachable!()
    }

    fn jack_flags() -> PortFlags {
        unreachable!()
    }

    fn jack_buffer_size() -> u64 {
        unreachable!()
    }
}

pub type UnownedPort = Port<Unowned>;
