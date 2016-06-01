use std::{ffi, slice};
use jack_sys as j;
use flags::*;
use utils;

/// Converts a jack client handle and jack port handle in a `Port`. If either
/// `client` or `port` is `null`, then `None` is returned.
pub unsafe fn ptrs_to_port(client: *mut j::jack_client_t,
                           port: *mut j::jack_port_t)
                           -> Option<Port> {
    if client.is_null() || port.is_null() {
        None
    } else {
        Some(Port { client: client, port: port })
    }
}

pub unsafe fn port_pointer(port: &Port) -> *mut j::jack_port_t {
    port.port
}

/// An endpoint to interact with Jack data streams, for audio, midi, etc...
#[derive(Debug, Clone, Copy)]
pub struct Port {
    client: *mut j::jack_client_t,
    port: *mut j::jack_port_t,
}

impl Port {
    /// The maximum length of a full Jack port name. Unlike the "C" Jack API,
    /// this does not count the `NULL` character and corresponds to a string's
    /// `.len()`.
    ///
    /// The port's full name contains the owning client name concatenated with a
    /// colon (:) followed by its short name.
    ///
    /// This value is constant
    pub fn name_size() -> usize {
        let s = unsafe { j::jack_port_name_size() - 1 };
        s as usize
    }

    /// The maximum length of a port type. Unlike the "C" Jack API, this does
    /// not count the `NULL` character and corresponds to a string's `.len()`.
    ///
    /// This value is constant.
    pub fn type_size() -> usize {
        let s = unsafe { j::jack_port_type_size() - 1 };
        s as usize
    }

    /// Remove the port from the client, disconnecting any existing connections.
    pub fn unregister(self) -> Result<(), ()> {
        let res = unsafe { j::jack_port_unregister(self.client, self.port) };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    /// Returns the full name of the port, including the "client_name:" prefix.
    pub fn name<'a>(&'a self) -> &'a str {
        unsafe { ffi::CStr::from_ptr(j::jack_port_name(self.port)).to_str().unwrap() }
    }

    /// Returns the short name of the port, it excludes the "client_name:" prefix.
    pub fn short_name<'a>(&'a self) -> &'a str {
        unsafe { ffi::CStr::from_ptr(j::jack_port_short_name(self.port)).to_str().unwrap() }
    }

    /// Returns the uuid of the port as a u64.
    pub fn uuid(&self) -> u64 {
        unsafe { j::jack_port_uuid(self.port) }
    }

    /// The flags for the port. These are set when the port is registered with
    /// its client.
    pub fn flags(&self) -> PortFlags {
        let bits = unsafe { j::jack_port_flags(self.port) };
        PortFlags::from_bits(bits as u32).unwrap()
    }

    /// The port type. Jack's built in types include "32 bit float mono audio"
    /// and "8 bit raw midi". Custom types may also be used.
    pub fn port_type<'a>(&self) -> &'a str {
        unsafe { ffi::CStr::from_ptr(j::jack_port_type(self.port)).to_str().unwrap() }
    }

    /// Number of ports connected to/from
    pub fn connected_count(&self) -> usize {
        let n = unsafe { j::jack_port_connected(self.port) };
        n as usize
    }

    /// Returns `true` if the port is directly connected to a port with the name
    /// `port_name`.
    pub fn is_connected_to(&self, port_name: &str) -> bool {
        let res = unsafe {
            let port_name = ffi::CString::new(port_name).unwrap();
            j::jack_port_connected_to(self.port, port_name.as_ptr())
        };
        match res {
            0 => false,
            _ => true,
        }
    }

    /// Full port names to which `self` is connected to. This combines Jack's
    /// `jack_port_get_all_connections()` and `jack_port_get_connections()`. If
    /// the `client` from which `port` was spawned from is the owner, then it
    /// may be used in the graph reordered callback or else it should not be
    /// used.
    ///
    /// # Unsafe
    ///
    /// * Can't be used in the callback for graph reordering under certain
    /// conditions.
    pub unsafe fn connections(&self) -> Vec<String> {
        let connections_ptr = {
            let ptr = if j::jack_port_is_mine(self.client, self.port) != 0 {
                j::jack_port_get_connections(self.port)
            } else {
                j::jack_port_get_all_connections(self.client, self.port)
            };
            utils::collect_strs(ptr)
        };
        connections_ptr
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
        match unsafe { j::jack_port_monitoring_input(self.port) } {
            0 => false,
            _ => true,
        }
    }

    /// Set's the short name of the port. If the full name is longer than
    /// `Port::name_size()`, then it will be truncated.
    pub fn set_name(&self, short_name: &str) -> Result<(), ()> {
        let short_name = ffi::CString::new(short_name).unwrap();
        let res = unsafe { j::jack_port_set_name(self.port, short_name.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(()),
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
    pub fn set_alias(&self, alias: &str) -> Result<(), ()> {
        let alias = ffi::CString::new(alias).unwrap();
        let res = unsafe { j::jack_port_set_alias(self.port, alias.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    /// Remove `alias` as an alias for port. May be called at any time.
    ///
    /// After a successful call, `alias` can no longer be used as an alternate
    /// name for `self`.
    pub fn unset_alias(&self, alias: &str) -> Result<(), ()> {
        let alias = ffi::CString::new(alias).unwrap();
        let res = unsafe { j::jack_port_unset_alias(self.port, alias.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    /// Turn input monitoring for the port on or off.
    ///
    /// This only works if the port has the `CAN_MONITOR` flag set.
    pub fn request_monitor(&self, enable_monitor: bool) -> Result<(), ()> {
        let res = unsafe {
            let onoff = match enable_monitor {
                true => 1,
                false => 0,
            };
            j::jack_port_request_monitor(self.port, onoff)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    /// If the `CAN_MONITOR` flag is set for the port, then input monitoring is
    /// turned on if it was off, and turns it off if only one request has been
    /// made to turn it on. Otherwise it does nothing.
    pub fn ensure_monitor(&self, enable_monitor: bool) -> Result<(), ()> {
        let res = unsafe {
            let onoff = match enable_monitor {
                true => 1,
                false => 0,
            };
            j::jack_port_ensure_monitor(self.port, onoff)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    /// Perform the same function as `Client::disconnect_ports()`, but with a
    /// port handle instead.
    ///
    /// Avoids the name lookup inherent in the name-based version.
    ///
    /// Clients connecting their own ports are likely to use this function,
    /// while generic connection clients (e.g. patchbays) would use
    /// `Client::disconnect_ports()`.
    pub fn disconnect(&self) -> Result<(), ()> {
        match unsafe { j::jack_port_disconnect(self.client, self.port) } {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    /// Returns a pointer to the memory area associated with the specified
    /// port. For an output port, it will be a memory area that can be written
    /// to; for an input port, it will be an area containing the data from the
    /// port's connection(s), or zero-filled. If there are multiple inbound
    /// connections, the data will be mixed appropriately.
    ///
    /// Do not cache the returned address across `process()` calls. Port buffers
    /// have to be retrieved in each callback for proper functioning.
    pub unsafe fn buffer(&self, n_frames: u32) -> *mut ::libc::c_void {
        j::jack_port_get_buffer(self.port, n_frames)
    }

    /// Interprets the buffer as a slice of type `T` with length `n_frames`.
    pub unsafe fn as_slice<T>(&self, n_frames: u32) -> &[T] {
        let buffer = self.buffer(n_frames) as *const T;
        slice::from_raw_parts(buffer, n_frames as usize)
    }

    /// Interprets the buffer as a mutable slice of type `T` with length
    /// `n_frames`.
    pub unsafe fn as_slice_mut<T>(&self, n_frames: u32) -> &mut [T] {
        let buffer = self.buffer(n_frames) as *mut T;
        slice::from_raw_parts_mut(buffer, n_frames as usize)
    }
}
