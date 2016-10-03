use std::{ffi, ptr};
use jack_sys as j;
use callbacks;
use port;
use enums::*;
use flags::*;
use port::*;
use utils;
use std::mem::transmute;
use callbacks::JackHandler;

#[derive(Clone, Copy, Debug)]
pub struct CycleTimes {
    pub current_frames: u32,
    pub current_usecs: u64,
    pub next_usecs: u64,
    pub period_usecs: f32,
}

/// A client to interact with a Jack server.
///
/// # Example
/// ```
/// // TODO: make example
/// ```
#[derive(Debug)]
pub struct Client<T: JackHandler> {
    client: *mut j::jack_client_t,
    handler: *mut T,
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
    /// # Unsafe
    ///
    /// * This function may only be called in a buffer size callback.
    pub unsafe fn type_buffer_size(&self, port_type: &str) -> usize {
        let port_type = ffi::CString::new(port_type).unwrap();
        let n = j::jack_port_type_get_buffer_size(self.client, port_type.as_ptr());
        n
    }

    /// Opens a Jack client with the given name and options. If the client is
    /// successfully opened, then `Ok(client)` is returned. If there is a
    /// failure, then `Err(JackErr::ClientError(status))` will be returned.
    ///
    /// Although the client may be successful in opening, there still may be
    /// some errors minor errors when attempting to opening. To access these,
    /// check the returned `ClientStatus`.
    pub fn open(client_name: &str, options: ClientOptions) -> Result<(Self, ClientStatus), JackErr> {
        let mut status_bits = 0;
        let client = unsafe {
            let client_name = ffi::CString::new(client_name).unwrap();
            j::jack_client_open(ffi::CString::new(client_name).unwrap().as_ptr(),
                                options.bits(),
                                &mut status_bits)
        };
        let status = ClientStatus::from_bits(status_bits).unwrap_or(UNKNOWN_ERROR);
        if client.is_null() {
            Err(JackErr::ClientError(status))
        } else {
            Ok((Client {
                client: client,
                handler: ptr::null_mut(),
            }, status))
        }
    }

    /// Disconnects the client from the Jack server. This does not need to
    /// manually be called, as the client will automatically close when the
    /// client object is dropped.
    pub fn close(self) {
        drop(self)
    }

    /// Get the name of the current client. This may differ from the name
    /// requested by `Client::open` as Jack will may rename a client if
    /// necessary (ie: name collision, name too long). The name will only
    /// the be different than the one passed to `Client::open` if the
    /// `ClientStatus` was `NAME_NOT_UNIQUE`.
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

    /// Returns a vector of port names that match the specified arguments
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
                 flags: PortFlags)
                 -> Vec<String> {
        let pnp = ffi::CString::new(port_name_pattern.unwrap_or("")).unwrap();
        let tnp = ffi::CString::new(type_name_pattern.unwrap_or("")).unwrap();
        let flags = flags.bits() as u64;
        unsafe {
            utils::collect_strs(j::jack_get_ports(self.client, pnp.as_ptr(), tnp.as_ptr(), flags))
        }
    }

    /// Get a `Port` by its port name.
    pub fn port_by_name(&self, port_name: &str) -> Option<Port<Unowned>> {
        let port_name = ffi::CString::new(port_name).unwrap();
        unsafe {
            ptrs_to_port(self.client,
                         j::jack_port_by_name(self.client, port_name.as_ptr()))
        }
    }

    /// Get a `Port` by its port id.
    pub fn port_by_id(&self, port_id: u32) -> Option<Port<Unowned>> {
        unsafe { ptrs_to_port(self.client, j::jack_port_by_id(self.client, port_id)) }
    }

    /// Tell the Jack server that the program is ready to start processing
    /// audio. Jack will call the methods specified by the `JackHandler` trait, from `handler`.
    ///
    /// On failure, either `Err(JackErr::CallbackRegistrationError)` or
    /// `Err(JackErr::ClientActivationError)` is returned.
    ///
    /// `handler` is consumed, but it is returned when `Client::deactivate` is
    /// called.
    pub fn activate(&mut self, handler: T) -> Result<(), JackErr> {
        let handler = try!(unsafe { callbacks::register_callbacks(self.client, handler) });
        if handler.is_null() {
            Err(JackErr::CallbackRegistrationError)
        } else {
            let res = unsafe { j::jack_activate(self.client) };
            match res {
                0 => {
                    self.handler = handler;
                    Ok(())
                }
                _ => {
                    unsafe { Box::from_raw(handler) };
                    Err(JackErr::ClientActivationError)
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
    pub fn deactivate(&mut self) -> Result<Box<T>, JackErr> {
        if self.handler.is_null() {
            return Err(JackErr::InvalidDeactivation);
        }
        let res = unsafe { j::jack_deactivate(self.client) };
        let handler_ptr = self.handler;
        self.handler = ptr::null_mut();
        try!(unsafe { callbacks::clear_callbacks(self.client) });
        match res {
            0 => Ok(unsafe { Box::from_raw(handler_ptr) }),
            _ => Err(JackErr::ClientDeactivationError),
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
    pub fn register_port<OPKind: OwnedPortKind>(&mut self,
                         port_name: &str,
                         port_type: &str,
                         flags: PortFlags,
                         buffer_size: Option<usize>)
                         -> Result<Port<OPKind>, JackErr> {
        let port_name = ffi::CString::new(port_name).unwrap();
        let port_type = ffi::CString::new(port_type).unwrap();
        let port_flags = (flags | OPKind::necessary_flags()).bits() as u64;
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
            Some(p) => Ok(unsafe{ claim_kind(p) }),
            None => Err(JackErr::PortRegistrationError),
        }
    }

    /// Returns `true` if the port `port` belongs to this client.
    pub fn is_mine<PKind: PortKind>(&self, port: &Port<PKind>) -> bool {
        match unsafe { j::jack_port_is_mine(self.client, port::port_pointer(port)) } {
            1 => true,
            _ => false,
        }
    }

    /// Toggle input monitoring for the port with name `port_name`.
    ///
    /// `Err(JackErr::PortMonitorError)` is returned on failure.
    ///
    /// Only works if the port has the `CAN_MONITOR` flag, or else nothing
    /// happens.
    pub fn request_monitor(&self, port_name: &str, enable_monitor: bool) -> Result<(), JackErr> {
        let port_name = ffi::CString::new(port_name).unwrap();
        let onoff = match enable_monitor {
            true => 1,
            false => 0,
        };
        let res =
            unsafe { j::jack_port_request_monitor_by_name(self.client, port_name.as_ptr(), onoff) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortMonitorError),
        }
    }

    /// Establish a connection between two ports.
    ///
    /// When a connection exists, data written to the source port will be
    /// available to be read at the destination port.
    ///
    /// On failure, either a `PortNotFound` or `PortConnectionError` is returned.
    ///
    /// # Preconditions
    /// 1. The port types must be identical
    /// 2. The port flags of the `source_port` must include `IS_OUTPUT`
    /// 3. The port flags of the `destination_port` must include `IS_INPUT`.
    pub fn connect_ports(&self, source_port: &str, destination_port: &str) -> Result<(), JackErr> {
        let source_port = ffi::CString::new(source_port).unwrap();
        let destination_port = ffi::CString::new(destination_port).unwrap();

        let res = unsafe {
            j::jack_connect(self.client, source_port.as_ptr(), destination_port.as_ptr())
        };
        match res {
            0 => Ok(()),
            ::libc::EEXIST => Err(JackErr::PortNotFound),
            _ => Err(JackErr::PortConnectionError),
        }
    }

    /// Remove a connection between two ports.
    pub fn disconnect_ports(&self,
                            source_port: &str,
                            destination_port: &str)
                            -> Result<(), JackErr> {
        let source_port = ffi::CString::new(source_port).unwrap();
        let destination_port = ffi::CString::new(destination_port).unwrap();
        let res = unsafe {
            j::jack_disconnect(self.client, source_port.as_ptr(), destination_port.as_ptr())
        };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortDisconnectionError),
        }
    }

    /// The sample rate of the jack system, as set by the user when jackd was
    /// started.
    pub fn sample_rate(&self) -> usize {
        let srate = unsafe { j::jack_get_sample_rate(self.client) };
        srate as usize
    }

    /// The current maximum size that will every be passed to the process
    /// callback.
    ///
    /// It should only be used *before* the client has been activated. This size
    /// may change,c lients that depend on it must register a buffer size
    /// callback so they will be notified if it does.
    pub fn buffer_size(&self) -> usize {
        let bsize = unsafe { j::jack_get_buffer_size(self.client) };
        bsize as usize
    }

    /// The current CPU load estimated by Jack.
    ///
    /// This is a running average of the time it takes to execute a full process
    /// cycle for all clients as a percentage of the real time available per
    /// cycle determined by the buffer size and sample rate.
    pub fn cpu_load(&self) -> f32 {
        let load = unsafe { j::jack_cpu_load(self.client) };
        load
    }

    /// Start/Stop Jack's "freewheel" mode.
    ///
    /// When in "freewheel" mode, Jack no longer waits for any external event to
    /// begin the start of the next process cycle. As a result, freewheel mode
    /// causes "faster than real-time" execution of a Jack graph. If possessed,
    /// real-time scheduling is dropped when entering freewheel mode, and if
    /// appropriate it is reacquired when stopping.
    ///
    /// IMPORTANT: on systems using capabilities to provide real-time scheduling
    /// (i.e. Linux Kernel 2.4), if enabling freewheel, this function must be
    /// called from the thread that originally called `self.activate()`. This
    /// restriction does not apply to other systems (e.g. Linux Kernel 2.6 or OS
    /// X).
    pub fn set_freewheel(&self, enable: bool) -> Result<(), JackErr> {
        let onoff = match enable {
            true => 0,
            false => 1,
        };
        match unsafe { j::jack_set_freewheel(self.client, onoff) } {
            0 => Ok(()),
            _ => Err(JackErr::FreewheelError),
        }
    }

    /// Change the buffer size passed to the process callback.
    ///
    /// This operation stops the jack engine process cycle, then calls all
    /// registered buffer size callback functions before restarting the process
    /// cycle. This will cause a gap in the audio flow, so it should only be
    /// done at appropriate stopping points.
    pub fn set_buffer_size(&self, n_frames: usize) -> Result<(), JackErr> {
        let n_frames = n_frames as u32;
        let res = unsafe { j::jack_set_buffer_size(self.client, n_frames) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::SetBufferSizeError),
        }
    }

    /// The estimated time in frames that has passed since the Jack server began
    /// the current process cycle.
    pub fn frames_since_cycle_start(&self) -> u32 {
        unsafe { j::jack_frames_since_cycle_start(self.client) }
    }

    /// The estimated current time in frames. This function is intended for use
    /// in other threads (not the process callback). The return value can be
    /// compared with the value of `last_frame_time` to relate time in other
    /// threads to Jack time.
    pub fn frame_time(&self) -> u32 {
        unsafe { j::jack_frame_time(self.client) }
    }

    /// The precise time at the start of the current process cycle. This
    /// function may only be used from the process callback, and can be used to
    /// interpret timestamps generated by `self.frame_time()` in other threads,
    /// with respect to the current process cycle.
    pub fn last_frame_time(&self) -> u32 {
        unsafe { j::jack_last_frame_time(self.client) }
    }

    /// This function may only be used from the process callback. It provides
    /// the internal cycle timing information as used by most of the other time
    /// related functions. This allows the caller to map between frame counts
    /// and microseconds with full precision (i.e. without rounding frame times
    /// to integers), and also provides e.g. the microseconds time of the start
    /// of the current cycle directly (it has to be computed otherwise).
    ///
    /// `Err(JackErr::TimeError)` is returned on failure.
    pub fn cycle_times(&self) -> Result<CycleTimes, JackErr> {
        let mut current_frames: u32 = 0;
        let mut current_usecs: u64 = 0;
        let mut next_usecs: u64 = 0;
        let mut period_usecs: f32 = 0.0;
        let res = unsafe {
            j::jack_get_cycle_times(self.client,
                                    &mut current_frames,
                                    &mut current_usecs,
                                    &mut next_usecs,
                                    &mut period_usecs)
        };
        match res {
            0 => {
                Ok(CycleTimes {
                    current_frames: current_frames,
                    current_usecs: current_usecs,
                    next_usecs: next_usecs,
                    period_usecs: period_usecs,
                })
            },
            _ => Err(JackErr::TimeError),
        }
    }

    /// The estimated time in microseconds of the specified frame time
    pub fn frames_to_time(&self, n_frames: u32) -> u64 {
        unsafe { j::jack_frames_to_time(self.client, n_frames) }
    }

    /// The estimated time in frames for the specified system time.
    pub fn time_to_frames(&self, t: u64) -> u32 {
        unsafe { j::jack_time_to_frames(self.client, t) }
    }

    pub fn client_ptr(&self) -> &j::jack_client_t {
        unsafe { transmute(self.client) }
    }

    /// Remove the port from the client, disconnecting any existing connections.
    /// The port must have been created with this client.
    pub fn unregister_port<OPKind: OwnedPortKind>(&mut self, port: Port<OPKind>) -> Result<(), JackErr> {
        port.unregister(self)
    }
}

/// Closes the client, no need to manually call `Client::close()`.
impl<T: JackHandler> Drop for Client<T> {
    fn drop(&mut self) {
        let _ = self.deactivate(); // may be Ok or Err, doesn't matter. TODO: fix style
        debug_assert!(!self.client.is_null()); // Rep invariant
        let res = unsafe { j::jack_client_close(self.client) };
        assert_eq!(res, 0);
        self.client = ptr::null_mut();
    }
}
