use std::{ffi, ptr};
use jack_sys as j;
use callbacks;
use port;
use flags::*;
use port::*;
use utils;
use callbacks::JackHandler;

/// A client to interact with a Jack server.
///
/// # Example
/// ```
/// // TODO: make example
/// ```
pub struct Client<T: JackHandler> {
    client: *mut j::jack_client_t,
    handler: *mut T,
    status: ClientStatus,
}

impl<T: JackHandler> Client<T> {
    /// The maximum length of the Jack client name string. Unlike the "C" Jack
    /// API, this does not take into account the final `NULL` character and
    /// instead corresponds directly to `.len()`. This value is constant.
    pub fn name_size() -> usize {
        let s = unsafe { j::jack_client_name_size() - 1 };
        s as usize
    }

    /// The buffer size of a port type
    ///
    /// This function may only be called in a buffer size callback.
    pub fn type_buffer_size(&self, port_type: &str) -> usize {
        let port_type = ffi::CString::new(port_type).unwrap();
        let n = unsafe {
            j::jack_port_type_get_buffer_size(self.client, port_type.as_ptr())
        };
        n
    }

    /// Opens a Jack client with the given name and options. If the client is
    /// successfully opened, then `Ok(client)` is returned. If there is a
    /// failure, then `Err(ClientStatus)` will be returned.
    ///
    /// Although the client may be successful in opening, there still may be
    /// some errors minor errors when attempting to opening. To access these,
    /// check `Client::status()`.
    pub fn open(client_name: &str, options: ClientOptions) -> Result<Self, ClientStatus> {
        let mut status_bits = 0;
        let client = unsafe {
            let client_name = ffi::CString::new(client_name).unwrap();
            j::jack_client_open(ffi::CString::new(client_name).unwrap().as_ptr(),
                                options.bits(),
                                &mut status_bits)
        };
        let status = ClientStatus::from_bits(status_bits).unwrap_or(UNKNOWN_ERROR);
        if client.is_null() {
            Err(status)
        } else {
            Ok(Client {
                client: client,
                handler: ptr::null_mut(),
                status: status,
            })
        }
    }

    /// Disconnects the client from the Jack server. This does not need to
    /// manually be called, as the client will automatically close when the /// client object is dropped.
    pub fn close(self) {}

    /// Get the status of the client.
    pub fn status(&self) -> ClientStatus {
        self.status
    }

    /// Get the name of the current client. This may differ from the name
    /// requested by `Client::open` as Jack will may rename a client if
    /// necessary (ie: name collision, name too long). If the name has changed,
    /// it should be indicated by `Client::status`.
    pub fn name<'a>(&'a self) -> &'a str {
        unsafe {
            let ptr = j::jack_get_client_name(self.client);
            let cstr = ffi::CStr::from_ptr(ptr);
            cstr.to_str().unwrap()
        }
    }

    /// Get the uuid of the current client.
    pub fn uuid<'a>(&'a self) -> &'a str {
        self.uuid_by_name(self.name()).unwrap()
    }

    /// Get the pthread ID of the thread running the Jack client side code.
    ///
    /// # TODO
    /// * Integrate a pthread library
    /// * Implement, do people need this though?
    pub fn thread_id<P>(&self) -> P {
        unimplemented!();
    }

    /// Get the name of the client with the UUID specified by `uuid`. If the
    /// client is found then `Some(name)` is returned, if not, then `None` is
    /// returned.
    pub fn name_by_uuid<'a>(&'a self, uuid: &str) -> Option<&'a str> {
        unsafe {
            let uuid = ffi::CString::new(uuid).unwrap();
            let name_ptr = j::jack_get_client_name_by_uuid(self.client, uuid.as_ptr());
            if name_ptr.is_null() {
                None
            } else {
                Some(ffi::CStr::from_ptr(name_ptr).to_str().unwrap())
            }
        }
    }

    /// Get the uuid of the client with the name specified by `name`. If the
    /// client is found then `Some(uuid)` is returned, if not, then `None` is
    /// returned.
    pub fn uuid_by_name<'a>(&'a self, name: &str) -> Option<&'a str> {
        unsafe {
            let name = ffi::CString::new(name).unwrap();
            let uuid_ptr = j::jack_get_client_name_by_uuid(self.client, name.as_ptr());
            if uuid_ptr.is_null() {
                None
            } else {
                Some(ffi::CStr::from_ptr(uuid_ptr).to_str().unwrap())
            }
        }
    }

    /// Returns a vector of ports that match the specified arguments
    ///
    /// `port_name_pattern` - A regular expression used to select ports by
    /// name. If `None` or zero lengthed, no selection based on name will be
    /// carried out.
    ///
    /// `type_name_pattern` - A regular expression used to select ports by
    /// type. If `None` or zero lengthed, no selection based on type will be
    /// carried out.
    ///
    /// `flags` - A value used to select ports by their flags. Use
    /// `PortFlags::empty()` for no flag selection.
    pub fn ports(&self,
                 port_name_pattern: Option<&str>,
                 type_name_pattern: Option<&str>,
                 flags: PortFlags) -> Vec<String> {
        let pnp = ffi::CString::new(port_name_pattern.unwrap_or(""))
            .unwrap();
        let tnp = ffi::CString::new(type_name_pattern.unwrap_or(""))
            .unwrap();
        let flags = flags.bits() as u64;
        unsafe {
            utils::collect_strs(j::jack_get_ports(self.client,
                                                  pnp.as_ptr(),
                                                  tnp.as_ptr(),
                                                  flags))
        }
    }

    /// Get a `Port` by its port name.
    pub fn port_by_name(&self, port_name: &str) -> Option<Port> {
        let port_name = ffi::CString::new(port_name).unwrap();
        unsafe {
            ptrs_to_port(self.client,
                         j::jack_port_by_name(self.client, port_name.as_ptr()))
        }
    }

    /// Get a `Port` by its port id.
    pub fn port_by_id(&self, port_id: u32) -> Option<Port> {
        unsafe {
            ptrs_to_port(self.client,
                         j::jack_port_by_id(self.client, port_id))
        }
    }

    /// Tell the Jack server that the program is ready to start processing
    /// audio. Jack will call the methods specified by the `JackHandler` trait, from `handler`.
    ///
    /// `handler` is consumed, but it is returned when `Client::deactivate` is called.
    pub fn activate(&mut self, handler: T) -> Result<(), ()> {
        let handler = unsafe { callbacks::register_callbacks(self.client, handler).unwrap() };
        if handler.is_null() {
            Err(())
        } else {
            let res = unsafe { j::jack_activate(self.client) };
            match res {
                0 => {
                    self.handler = handler;
                    Ok(())
                }
                _ => {
                    unsafe { Box::from_raw(handler) };
                    Err(())
                }
            }
        }
    }

    /// Tell the Jack server to remove this client from the process graph. Also,
    /// disconnect all ports belonging to it since inactive clients have no port
    /// connections.
    ///
    /// The `handler` that was used for `Client::activate` is returned on
    /// success. Its state may have changed due to Jack calling its methods.
    pub fn deactivate(&mut self) -> Result<Box<T>, ()> {
        if !self.handler.is_null() {
            let res = unsafe { j::jack_deactivate(self.client) };
            let handler_ptr = self.handler;
            self.handler = ptr::null_mut();
            unsafe { callbacks::clear_callbacks(self.client) };
            match res {
                0 => Ok(unsafe { Box::from_raw(handler_ptr) }),
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }

    /// Create a new port for the client. This is an object used for moving data
    /// of any type in or out of the client. Ports may be connected in various
    /// ways.
    ///
    /// Each port has a short name. The port's full name contains the name of
    /// the client concatenated with a colon (:) followed by its short
    /// name. `Port::name_size()` is the maximum length of the full
    /// name. Exceeding that will cause the port registration to fail and return
    /// `Err(())`.
    ///
    /// The `port_name` must be unique among all ports owned by this client. If
    /// the name is not unique, the registration will fail.
    ///
    /// All ports have a type, which may be any non empty string, passed as an
    /// argument. Some port types are built into the Jack API, like
    /// `DEFAULT_AUDIO_TYPE` and `DEFAULT_MIDI_TYPE`.
    ///
    /// # Parameters
    ///
    /// `port_name` - non-empty short name for the new port (not including the
    /// lading "client_name:"). Must be unique.
    ///
    /// `port_type` - port type name. If longer than `Port::type_size()`, only
    /// that many characters are significant.
    ///
    /// `flags` - `PortFlags` bit mask.
    ///
    /// `buffer_size` - Must be `Some(n)` if this is not a built-in
    /// `port_type`. Otherwise, it is ignored.
    pub fn register_port(&mut self,
                     port_name: &str,
                     port_type: &str,
                     flags: PortFlags,
                     buffer_size: Option<usize>)
                     -> Result<Port, ()> {
        let port_name = ffi::CString::new(port_name).unwrap();
        let port_type = ffi::CString::new(port_type).unwrap();
        let port_flags = flags.bits() as u64;
        let buffer_size = buffer_size.unwrap_or(0) as u64;
        let port = unsafe {
            let ptr = j::jack_port_register(self.client,
                                            port_name.as_ptr(),
                                            port_type.as_ptr(),
                                            port_flags,
                                            buffer_size);
            ptrs_to_port(self.client, ptr)
        };
        match port {
            Some(p) => Ok(p),
            None => Err(())
        }
    }

    /// Returns `true` if the port `port` belongs to this client.
    pub fn is_mine(&self, port: &Port) -> bool {
        match unsafe { j::jack_port_is_mine(self.client, port::port_pointer(port)) } {
            0 => false,
            _ => true,
        }
    }

    /// Toggle input monitoring for the port with name `port_name`.
    ///
    /// Only works if the port has the `CAN_MONITOR` flag, or else nothing
    /// happens.
    pub fn request_monitor(&self, port_name: &str, enable_monitor: bool) -> Result<(), ()> {
        let port_name = ffi::CString::new(port_name).unwrap();
        let onoff = match enable_monitor {
            true  => 1,
            false => 0,
        };
        let res = unsafe {
            j::jack_port_request_monitor_by_name(self.client,
                                                 port_name.as_ptr(),
                                                 onoff)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    /// Establish a connection between two ports.
    ///
    /// When a connection exists, data written to the source port will be
    /// available to be read at the destination port.
    ///
    /// # Preconditions
    /// 1. The port types must be identical
    /// 2. The port flags of the `source_port` must include `IS_OUTPUT`
    /// 3. The port flags of the `destination_port` must include `IS_INPUT`.
    ///
    /// # TODO
    /// * In a rare instance, Jack API specifies a possible error return value, so use that
    pub fn connect_ports(&self, source_port: &str, destination_port: &str) -> Result<(), ()> {
        let source_port = ffi::CString::new(source_port).unwrap();
        let destination_port = ffi::CString::new(destination_port).unwrap();
        match unsafe { j::jack_connect(self.client, source_port.as_ptr(), destination_port.as_ptr()) } {
            0              => Ok(()),
            ::libc::EEXIST => Err(()),
            _              => Err(())
        }
    }

    /// Remove a connection between two ports.
    pub fn disconnect_ports(&self, source_port: &str, destination_port: &str) -> Result<(), ()> {
        let source_port = ffi::CString::new(source_port).unwrap();
        let destination_port = ffi::CString::new(destination_port).unwrap();
        match unsafe { j::jack_disconnect(self.client, source_port.as_ptr(), destination_port.as_ptr()) } {
            0 => Ok(()),
            _ => Err(())
        }
    }

}

/// Closes the client, no need to manually call `Client::close()`.
impl<T: JackHandler> Drop for Client<T> {
    fn drop(&mut self) {
        let _ = self.deactivate(); // may be Ok or Err, doesn't matter. TODO: fix style
        if !self.client.is_null() {
            let res = unsafe { j::jack_client_close(self.client) };
            assert_eq!(res, 0);
            self.client = ptr::null_mut();
        }
    }
}
