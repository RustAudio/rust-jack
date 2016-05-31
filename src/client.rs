use std::{ffi, ptr};
use jack_sys as j;
use callbacks;
use port;
use flags::*;
use port::*;
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
        unsafe {
            port::port_register(self.client,
                                port_name,
                                port_type,
                                flags,
                                buffer_size.unwrap_or(0))
        }
    }

    /// Returns `true` if the port `port` belongs to this client.
    pub fn is_mine(&self, port: &Port) -> bool {
        match unsafe { j::jack_port_is_mine(self.client, port::port_pointer(port)) } {
            0 => false,
            _ => true,
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
