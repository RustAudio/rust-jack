use jack_sys as j;

use std::mem;
use std::{ffi, ptr};

use callbacks::{JackHandler, ProcessScope, register_callbacks, clear_callbacks};
use jack_enums::*;
use jack_flags::client_options::ClientOptions;
use jack_flags::client_status::{ClientStatus, UNKNOWN_ERROR};
use jack_flags::port_flags::PortFlags;
use jack_utils::collect_strs;
use port::{Port, PortSpec, UnownedPort};
use port;

/// The maximum length of the JACK client name string. Unlike the "C" JACK
/// API, this does not take into account the final `NULL` character and
/// instead corresponds directly to `.len()`. This value is constant.
fn client_name_size() -> usize {
    let s = unsafe { j::jack_client_name_size() - 1 };
    s as usize
}

lazy_static! {
    /// The maximum string length for port names.
    pub static ref CLIENT_NAME_SIZE: usize = client_name_size();
}

/// Internal cycle timing information.
#[derive(Clone, Copy, Debug)]
pub struct CycleTimes {
    pub current_frames: u32,
    pub current_usecs: u64,
    pub next_usecs: u64,
    pub period_usecs: f32,
}

/// A client to interact with a JACK server.
///
/// # Example
/// ```
/// // TODO: make example
/// ```
#[derive(Debug)]
pub struct Client {
    client: *mut j::jack_client_t,
}

/// A `JackClient` that is currently active. Active clients may
/// contain `JackHandler`s that are processing data in real-time.
#[derive(Debug)]
pub struct ActiveClient<JH: JackHandler> {
    client: *mut j::jack_client_t,
    handler: *mut (JH, *mut j::jack_client_t),
}

unsafe impl JackClient for Client {
    fn client_ptr(&self) -> *mut j::jack_client_t {
        self.client
    }
}

unsafe impl<JH: JackHandler> JackClient for ActiveClient<JH> {
    fn client_ptr(&self) -> *mut j::jack_client_t {
        self.client
    }
}

/// Common JACK client functionality that can be accessed for both
/// inactive and active clients.
pub unsafe trait JackClient: Sized {
    #[inline(always)]
    fn client_ptr(&self) -> *mut j::jack_client_t;

    /// The sample rate of the JACK system, as set by the user when jackd was
    /// started.
    fn sample_rate(&self) -> usize {
        let srate = unsafe { j::jack_get_sample_rate(self.client_ptr()) };
        srate as usize
    }

    /// The current CPU load estimated by JACK.
    ///
    /// This is a running average of the time it takes to execute a full process
    /// cycle for all clients as a percentage of the real time available per
    /// cycle determined by the buffer size and sample rate.
    fn cpu_load(&self) -> f32 {
        let load = unsafe { j::jack_cpu_load(self.client_ptr()) };
        load
    }


    /// Get the name of the current client. This may differ from the name
    /// requested by `Client::open` as JACK will may rename a client if
    /// necessary (ie: name collision, name too long). The name will only
    /// the be different than the one passed to `Client::open` if the
    /// `ClientStatus` was `NAME_NOT_UNIQUE`.
    fn name<'a>(&'a self) -> &'a str {
        unsafe {
            let ptr = j::jack_get_client_name(self.client_ptr());
            let cstr = ffi::CStr::from_ptr(ptr);
            cstr.to_str().unwrap()
        }
    }

    /// The current maximum size that will every be passed to the process
    /// callback.
    fn buffer_size(&self) -> u32 {
        unsafe { j::jack_get_buffer_size(self.client_ptr()) }
    }

    /// Change the buffer size passed to the process callback.
    ///
    /// This operation stops the JACK engine process cycle, then calls all
    /// registered buffer size callback functions before restarting the process
    /// cycle. This will cause a gap in the audio flow, so it should only be
    /// done at appropriate stopping points.
    fn set_buffer_size(&self, n_frames: u32) -> Result<(), JackErr> {
        let res = unsafe { j::jack_set_buffer_size(self.client_ptr(), n_frames) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::SetBufferSizeError),
        }
    }
    // TODO implement
    // /// Get the uuid of the current client.
    // fn uuid<'a>(&'a self) -> &'a str {
    //     self.uuid_by_name(self.name()).unwrap_or("")
    // }

    // TODO implement
    // // Get the name of the client with the UUID specified by `uuid`. If the
    // // client is found then `Some(name)` is returned, if not, then `None` is
    // // returned.
    // // fn name_by_uuid<'a>(&'a self, uuid: &str) -> Option<&'a str> {
    //     unsafe {
    //         let uuid = ffi::CString::new(uuid).unwrap();
    //         let name_ptr = j::jack_get_client_name_by_uuid(self.client_ptr(), uuid.as_ptr());
    //         if name_ptr.is_null() {
    //             None
    //         } else {
    //             Some(ffi::CStr::from_ptr(name_ptr).to_str().unwrap())
    //         }
    //     }
    // }

    // TODO implement
    // /// Get the uuid of the client with the name specified by `name`. If the
    // /// client is found then `Some(uuid)` is returned, if not, then `None` is
    // /// returned.
    // fn uuid_by_name<'a>(&'a self, name: &str) -> Option<&'a str> {
    //     unsafe {
    //         let name = ffi::CString::new(name).unwrap();
    //         let uuid_ptr = j::jack_get_client_name_by_uuid(self.client_ptr(), name.as_ptr());
    //         if uuid_ptr.is_null() {
    //             None
    //         } else {
    //             Some(ffi::CStr::from_ptr(uuid_ptr).to_str().unwrap())
    //         }
    //     }
    // }

    /// Returns a vector of port names that match the specified arguments
    ///
    /// `port_name_pattern` - A regular expression used to select ports by
    /// name. If `None` or zero lengthed, no selection based on name will be
    /// carried out.
    ///
    /// `type_name_pattern` - A regular expression used to select ports by type. If `None` or zero
    /// lengthed, no selection based on type will be carried out. The port type is the same one
    /// returned by `PortSpec::jack_port_type()`. For example, `AudioInSpec` and `AudioOutSpec` are
    /// both of type `"32 bit float mono audio"`.
    ///
    /// `flags` - A value used to select ports by their flags. Use
    /// `PortFlags::empty()` for no flag selection.
    fn ports(&self,
             port_name_pattern: Option<&str>,
             type_name_pattern: Option<&str>,
             flags: PortFlags)
             -> Vec<String> {
        let pnp = ffi::CString::new(port_name_pattern.unwrap_or("")).unwrap();
        let tnp = ffi::CString::new(type_name_pattern.unwrap_or("")).unwrap();
        let flags = flags.bits() as u64;
        unsafe {
            let ports = j::jack_get_ports(self.client_ptr(), pnp.as_ptr(), tnp.as_ptr(), flags);
            collect_strs(ports)
        }
    }

    // TODO implement
    // // Get a `Port` by its port id.
    // fn port_by_id(&self, port_id: u32) -> Option<UnownedPort> {
    //     let pp = unsafe { j::jack_port_by_id(self.client_ptr(), port_id) };
    //     if pp.is_null() {
    //         None
    //     } else {
    //         Some(unsafe { Port::from_raw(port::Unowned {}, self.client_ptr(), pp) })
    //     }
    // }

    /// Get a `Port` by its port name.
    fn port_by_name(&self, port_name: &str) -> Option<UnownedPort> {
        let port_name = ffi::CString::new(port_name).unwrap();
        let pp = unsafe { j::jack_port_by_name(self.client_ptr(), port_name.as_ptr()) };
        if pp.is_null() {
            None
        } else {
            Some(unsafe { Port::from_raw(port::Unowned {}, self.client_ptr(), pp) })
        }
    }

    /// The estimated time in frames that has passed since the JACK server began
    /// the current process cycle.
    ///
    /// # TODO
    /// - test
    fn frames_since_cycle_start(&self) -> u32 {
        unsafe { j::jack_frames_since_cycle_start(self.client_ptr()) }
    }

    /// The estimated current time in frames. This function is intended for use
    /// in other threads (not the process callback). The return value can be
    /// compared with the value of `last_frame_time` to relate time in other
    /// threads to JACK time.
    ///
    /// # TODO
    /// - test
    fn frame_time(&self) -> u32 {
        unsafe { j::jack_frame_time(self.client_ptr()) }
    }

    /// The precise time at the start of the current process cycle. This
    /// function may only be used from the process callback, and can be used to
    /// interpret timestamps generated by `self.frame_time()` in other threads,
    /// with respect to the current process cycle.
    /// # TODO
    /// - test
    fn last_frame_time(&self, _ps: &ProcessScope) -> u32 {
        unsafe { j::jack_last_frame_time(self.client_ptr()) }
    }

    /// This function may only be used from the process callback. It provides
    /// the internal cycle timing information as used by most of the other time
    /// related functions. This allows the caller to map between frame counts
    /// and microseconds with full precision (i.e. without rounding frame times
    /// to integers), and also provides e.g. the microseconds time of the start
    /// of the current cycle directly (it has to be computed otherwise).
    ///
    /// `Err(JackErr::TimeError)` is returned on failure.
    ///
    /// TODO
    /// - test
    fn cycle_times(&self) -> Result<CycleTimes, JackErr> {
        let mut current_frames: u32 = 0;
        let mut current_usecs: u64 = 0;
        let mut next_usecs: u64 = 0;
        let mut period_usecs: f32 = 0.0;
        let res = unsafe {
            j::jack_get_cycle_times(self.client_ptr(),
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
            }
            _ => Err(JackErr::TimeError),
        }
    }

    /// The estimated time in microseconds of the specified frame time
    ///
    /// TODO
    fn frames_to_time(&self, n_frames: u32) -> u64 {
        unsafe { j::jack_frames_to_time(self.client_ptr(), n_frames) }
    }

    /// The estimated time in frames for the specified system time.
    ///
    /// # TODO
    fn time_to_frames(&self, t: u64) -> u32 {
        unsafe { j::jack_time_to_frames(self.client_ptr(), t) }
    }

    /// Returns `true` if the port `port` belongs to this client.
    fn is_mine<PS: PortSpec>(&self, port: &Port<PS>) -> bool {
        match unsafe { j::jack_port_is_mine(self.client_ptr(), port.port_ptr()) } {
            1 => true,
            _ => false,
        }
    }

    /// Establish a connection between two ports by their full name.
    ///
    /// When a connection exists, data written to the source port will be
    /// available to be read at the destination port.
    ///
    /// On failure, either a `PortAlreadyConnected` or `PortConnectionError` is returned.
    ///
    /// # Preconditions
    /// 1. The port types must be identical
    /// 2. The port flags of the `source_port` must include `IS_OUTPUT`
    /// 3. The port flags of the `destination_port` must include `IS_INPUT`.
    /// 4. Both ports must be owned by active clients.
    fn connect_ports_by_name(&self,
                             source_port: &str,
                             destination_port: &str)
                             -> Result<(), JackErr> {
        let source_cstr = ffi::CString::new(source_port).unwrap();
        let destination_cstr = ffi::CString::new(destination_port).unwrap();

        let res = unsafe {
            j::jack_connect(self.client_ptr(),
                            source_cstr.as_ptr(),
                            destination_cstr.as_ptr())
        };
        match res {
            0 => Ok(()),
            ::libc::EEXIST => {
                Err(JackErr::PortAlreadyConnected(source_port.to_string(),
                                                  destination_port.to_string()))
            }
            _ => {
                Err(JackErr::PortConnectionError(source_port.to_string(),
                                                 destination_port.to_string()))
            }
        }
    }

    /// Establish a connection between two ports.
    ///
    /// When a connection exists, data written to the source port will be
    /// available to be read at the destination port.
    ///
    /// On failure, either a `PortAlreadyConnected` or `PortConnectionError` is returned.
    ///
    /// # Preconditions
    /// 1. The port types must be identical
    /// 2. The port flags of the `source_port` must include `IS_OUTPUT`
    /// 3. The port flags of the `destination_port` must include `IS_INPUT`.
    /// 4. Both ports must be owned by active clients.
    fn connect_ports<A: PortSpec, B: PortSpec>(&self,
                                               source_port: &Port<A>,
                                               destination_port: &Port<B>)
                                               -> Result<(), JackErr> {
        self.connect_ports_by_name(source_port.name(), destination_port.name())
    }

    /// Remove a connection between two ports.
    fn disconnect_ports<A: PortSpec, B: PortSpec>(&self,
                                                  source: &Port<A>,
                                                  destination: &Port<B>)
                                                  -> Result<(), JackErr> {
        self.disconnect_ports_by_name(source.name(), destination.name())
    }

    /// Remove a connection between two ports.
    fn disconnect_ports_by_name(&self,
                                source_port: &str,
                                destination_port: &str)
                                -> Result<(), JackErr> {
        let source_port = ffi::CString::new(source_port).unwrap();
        let destination_port = ffi::CString::new(destination_port).unwrap();
        let res = unsafe {
            j::jack_disconnect(self.client_ptr(),
                               source_port.as_ptr(),
                               destination_port.as_ptr())
        };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortDisconnectionError),
        }
    }

    /// The buffer size of a port type
    ///
    /// # Unsafe
    ///
    /// * This function may only be called in a buffer size callback.
    unsafe fn type_buffer_size(&self, port_type: &str) -> usize {
        let port_type = ffi::CString::new(port_type).unwrap();
        let n = j::jack_port_type_get_buffer_size(self.client_ptr(), port_type.as_ptr());
        n
    }
}

impl Client {
    /// The maximum length of the JACK client name string. Unlike the "C" JACK
    /// API, this does not take into account the final `NULL` character and
    /// instead corresponds directly to `.len()`.

    /// Opens a JACK client with the given name and options. If the client is
    /// successfully opened, then `Ok(client)` is returned. If there is a
    /// failure, then `Err(JackErr::ClientError(status))` will be returned.
    ///
    /// Although the client may be successful in opening, there still may be
    /// some errors minor errors when attempting to opening. To access these,
    /// check the returned `ClientStatus`.
    pub fn open(client_name: &str,
                options: ClientOptions)
                -> Result<(Self, ClientStatus), JackErr> {
        let mut status_bits = 0;
        let client = unsafe {
            let client_name = ffi::CString::new(client_name).unwrap();
            j::jack_client_open(client_name.as_ptr(), options.bits(), &mut status_bits)
        };
        let status = ClientStatus::from_bits(status_bits).unwrap_or(UNKNOWN_ERROR);
        if client.is_null() {
            Err(JackErr::ClientError(status))
        } else {
            Ok((Client { client: client }, status))
        }
    }


    /// Tell the JACK server that the program is ready to start processing
    /// audio. JACK will call the methods specified by the `JackHandler` trait, from `handler`.
    ///
    /// On failure, either `Err(JackErr::CallbackRegistrationError)` or
    /// `Err(JackErr::ClientActivationError)` is returned.
    ///
    /// `handler` is consumed, but it is returned when `Client::deactivate` is
    /// called.
    pub fn activate<JH: JackHandler>(self, handler: JH) -> Result<ActiveClient<JH>, JackErr> {
        unsafe {
            let handler_ptr = try!(register_callbacks(handler, self.client, self.client_ptr()));
            if handler_ptr.is_null() {
                Err(JackErr::CallbackRegistrationError)
            } else {
                let res = j::jack_activate(self.client);
                match res {
                    0 => {
                        let Client { client } = self;

                        // Don't run destructor -- we want the client to stay open
                        mem::forget(self);

                        Ok(ActiveClient {
                            client: client,
                            handler: handler_ptr,
                        })
                    }
                    _ => {
                        drop(Box::from_raw(handler_ptr));
                        Err(JackErr::ClientActivationError)
                    }
                }
            }
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
    pub fn register_port<PS: PortSpec>(&mut self,
                                       port_name: &str,
                                       port_spec: PS)
                                       -> Result<Port<PS>, JackErr> {
        let port_name_c = ffi::CString::new(port_name).unwrap();
        let port_type_c = ffi::CString::new(port_spec.jack_port_type()).unwrap();
        let port_flags = port_spec.jack_flags().bits() as u64;
        let buffer_size = port_spec.jack_buffer_size() as u64;
        let pp = unsafe {
            j::jack_port_register(self.client,
                                  port_name_c.as_ptr(),
                                  port_type_c.as_ptr(),
                                  port_flags,
                                  buffer_size)
        };
        if pp.is_null() {
            Err(JackErr::PortRegistrationError(port_name.to_string()))
        } else {
            Ok(unsafe { Port::from_raw(port_spec, self.client_ptr(), pp) })
        }
    }


    /// Toggle input monitoring for the port with name `port_name`.
    ///
    /// `Err(JackErr::PortMonitorError)` is returned on failure.
    ///
    /// Only works if the port has the `CAN_MONITOR` flag, or else nothing
    /// happens.
    pub fn request_monitor_by_name(&self,
                                   port_name: &str,
                                   enable_monitor: bool)
                                   -> Result<(), JackErr> {
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


    // TODO implement
    // /// Start/Stop JACK's "freewheel" mode.
    // ///
    // /// When in "freewheel" mode, JACK no longer waits for any external event to
    // /// begin the start of the next process cycle. As a result, freewheel mode
    // /// causes "faster than real-time" execution of a JACK graph. If possessed,
    // /// real-time scheduling is dropped when entering freewheel mode, and if
    // /// appropriate it is reacquired when stopping.
    // ///
    // /// IMPORTANT: on systems using capabilities to provide real-time scheduling
    // /// (i.e. Linux Kernel 2.4), if enabling freewheel, this function must be
    // /// called from the thread that originally called `self.activate()`. This
    // /// restriction does not apply to other systems (e.g. Linux Kernel 2.6 or OS
    // /// X).
    // pub fn set_freewheel(&self, enable: bool) -> Result<(), JackErr> {
    //     let onoff = match enable {
    //         true => 0,
    //         false => 1,
    //     };
    //     match unsafe { j::jack_set_freewheel(self.client_ptr(), onoff) } {
    //         0 => Ok(()),
    //         _ => Err(JackErr::FreewheelError),
    //     }
    // }
}

impl<JH: JackHandler> ActiveClient<JH> {
    /// Tell the JACK server to remove this client from the process graph. Also,
    /// disconnect all ports belonging to it since inactive clients have no port
    /// connections.
    ///
    /// The `handler` that was used for `Client::activate` is returned on
    /// success. Its state may have changed due to JACK calling its methods.
    ///
    /// In the case of error, the `Client` is destroyed because its state is
    /// unknown, and it is therefore unsafe to continue using.
    pub fn deactivate(self) -> Result<(Client, JH), JackErr> {
        unsafe {
            let ActiveClient { client, handler } = self;

            // Prevent destructor from running, as this would cause double-deactivation
            mem::forget(self);

            let res = match j::jack_deactivate(client) {
                // We own the handler post-deactivation
                0 => Ok(Box::from_raw(handler)),

                // We may still own the handler here, but it's not safe to say
                // without more information about the error condition
                _ => Err(JackErr::ClientDeactivationError),
            };

            let callback_res = clear_callbacks(client);

            match (res, callback_res) {
                (Ok(handler_ptr), Ok(())) => {
                    let (handler, _) = *handler_ptr;
                    Ok(( Client { client: client }, handler ))
                }
                (Err(err), _) | (_, Err(err)) => {
                    // We've invalidated the client, so it must be closed
                    j::jack_client_close(client);
                    Err(err)
                }
            }
        }
    }
}

/// Close the client, deactivating if necessary.
impl Drop for Client {
    fn drop(&mut self) {
        debug_assert!(!self.client.is_null()); // Rep invariant

        // Client isn't active, so no need to deactivate

        let res = unsafe { j::jack_client_close(self.client) }; // close the client
        assert_eq!(res, 0);
        self.client = ptr::null_mut();
    }
}

/// Closes the client.
impl<JH: JackHandler> Drop for ActiveClient<JH> {
    fn drop(&mut self) {
        unsafe {
            debug_assert!(!self.client.is_null()); // Rep invariant

            j::jack_deactivate(self.client); // result doesn't matter
            drop(Box::from_raw(self.handler)); // drop the handler

            let res = j::jack_client_close(self.client); // close the client
            assert_eq!(res, 0);
            self.client = ptr::null_mut();
        }
    }
}
