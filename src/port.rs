use std::{ffi, slice};
use jack_sys as j;
use flags::*;

/// Register a port when given a jack client pointer. For use within the API.
pub unsafe fn port_register(client: *mut j::jack_client_t,
                            port_name: &str,
                            port_type: &str,
                            port_flags: PortFlags,
                            buffer_size: usize)
                            -> Result<Port, ()> {
    assert!(!client.is_null());
    let port = {
        let port_name = ffi::CString::new(port_name).unwrap();
        let port_type = ffi::CString::new(port_type).unwrap();
        let port_flags = port_flags.bits() as u64;
        let buffer_size = buffer_size as u64;
        j::jack_port_register(client,
                              port_name.as_ptr(),
                              port_type.as_ptr(),
                              port_flags,
                              buffer_size)
    };
    if port.is_null() {
        Err(())
    } else {
        Ok(Port {
            client: client,
            port: port,
        })
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
    pub fn name_size() -> usize {
        let s = unsafe { j::jack_port_name_size() - 1 };
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
            _ => true
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
    pub unsafe fn as_slice_mut<T>(&self, n_frames: u32) -> &mut[T] {
        let buffer = self.buffer(n_frames) as *mut T;
        slice::from_raw_parts_mut(buffer, n_frames as usize)
    }
}
