use std::{ffi, fmt, ptr};

use jack_sys as j;
use libc;

use client::client_options::ClientOptions;
use client::client_status::ClientStatus;
use client::common::{CREATE_OR_DESTROY_CLIENT_MUTEX, sleep_on_test};
use jack_enums::*;
use jack_utils::collect_strs;
use port::{Port, PortSpec, Unowned, UnownedPort};
use port::port_flags::PortFlags;
use primitive_types as pt;

/// A client to interact with a JACK server.
///
/// For asynchronous operation, see the `AsyncClient` struct.
///
/// # Example
/// ```
/// use jack::prelude as j;
///
/// let c_res = j::Client::new("rusty_client",
/// j::client_options::NO_START_SERVER);
/// match c_res {
/// Ok((client, status)) => println!("Managed to open client {}, with
/// status {:?}!",
///                                      client.name(), status),
///     Err(e) => println!("Failed to open client because of error: {:?}", e),
/// };
///
/// ```
pub struct Client(*mut j::jack_client_t);

unsafe impl Send for Client {}

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
    pub fn new(client_name: &str, options: ClientOptions) -> Result<(Self, ClientStatus), JackErr> {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();
        sleep_on_test();
        let mut status_bits = 0;
        let client = unsafe {
            let client_name = ffi::CString::new(client_name).unwrap();
            j::jack_client_open(client_name.as_ptr(), options.bits(), &mut status_bits)
        };
        sleep_on_test();
        let status = ClientStatus::from_bits(status_bits).unwrap_or(ClientStatus::empty());
        if client.is_null() {
            Err(JackErr::ClientError(status))
        } else {
            Ok((Client(client), status))
        }
    }

    /// The sample rate of the JACK system, as set by the user when jackd was
    /// started.
    pub fn sample_rate(&self) -> usize {
        let srate = unsafe { j::jack_get_sample_rate(self.as_ptr()) };
        srate as usize
    }

    /// The current CPU load estimated by JACK. It is on a scale of `0.0` to
    /// `100.0`.
    ///
    /// This is a running average of the time it takes to execute a full
    /// process cycle for all
    /// clients as a percentage of the real time available per cycle determined
    /// by the buffer size
    /// and sample rate.
    pub fn cpu_load(&self) -> f32 {
        let load = unsafe { j::jack_cpu_load(self.as_ptr()) };
        load as f32
    }


    /// Get the name of the current client. This may differ from the name
    /// requested by
    /// `Client::new` as JACK will may rename a client if necessary (ie: name
    /// collision, name too
    /// long). The name will only the be different than the one passed to
    /// `Client::new` if the
    /// `ClientStatus` was `NAME_NOT_UNIQUE`.
    pub fn name<'a>(&'a self) -> &'a str {
        unsafe {
            let ptr = j::jack_get_client_name(self.as_ptr());
            let cstr = ffi::CStr::from_ptr(ptr);
            cstr.to_str().unwrap()
        }
    }

    /// The current maximum size that will every be passed to the process
    /// callback.
    pub fn buffer_size(&self) -> pt::JackFrames {
        unsafe { j::jack_get_buffer_size(self.as_ptr()) }
    }

    /// Change the buffer size passed to the process callback.
    ///
    /// This operation stops the JACK engine process cycle, then calls all
    /// registered buffer size
    /// callback functions before restarting the process cycle. This will cause
    /// a gap in the audio
    /// flow, so it should only be done at appropriate stopping points.
    pub fn set_buffer_size(&self, n_frames: pt::JackFrames) -> Result<(), JackErr> {
        let res = unsafe { j::jack_set_buffer_size(self.as_ptr(), n_frames) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::SetBufferSizeError),
        }
    }
    // TODO implement
    // /// Get the uuid of the current client.
    // pub fn uuid<'a>(&'a self) -> &'a str {
    //     self.uuid_by_name(self.name()).unwrap_or("")
    // }

    // TODO implement
    // // Get the name of the client with the UUID specified by `uuid`. If the
    // // client is found then `Some(name)` is returned, if not, then `None` is
    // // returned.
    // // pub fn name_by_uuid<'a>(&'a self, uuid: &str) -> Option<&'a str> {
    //     unsafe {
    //         let uuid = ffi::CString::new(uuid).unwrap();
    // let name_ptr = j::jack_get_client_name_by_uuid(self.as_ptr(),
    // uuid.as_ptr());
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
    // pub fn uuid_by_name<'a>(&'a self, name: &str) -> Option<&'a str> {
    //     unsafe {
    //         let name = ffi::CString::new(name).unwrap();
    // let uuid_ptr = j::jack_get_client_name_by_uuid(self.as_ptr(),
    // name.as_ptr());
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
    /// `type_name_pattern` - A regular expression used to select ports by
    /// type. If `None` or zero
    /// lengthed, no selection based on type will be carried out. The port type
    /// is the same one
    /// returned by `PortSpec::jack_port_type()`. For example, `AudioInSpec`
    /// and `AudioOutSpec` are
    /// both of type `"32 bit float mono audio"`.
    ///
    /// `flags` - A value used to select ports by their flags. Use
    /// `PortFlags::empty()` for no flag selection.
    pub fn ports(
        &self,
        port_name_pattern: Option<&str>,
        type_name_pattern: Option<&str>,
        flags: PortFlags,
    ) -> Vec<String> {
        let pnp = ffi::CString::new(port_name_pattern.unwrap_or("")).unwrap();
        let tnp = ffi::CString::new(type_name_pattern.unwrap_or("")).unwrap();
        let flags = flags.bits() as libc::c_ulong;
        unsafe {
            let ports = j::jack_get_ports(self.as_ptr(), pnp.as_ptr(), tnp.as_ptr(), flags);
            collect_strs(ports)
        }
    }

    /// Create a new port for the client. This is an object used for moving
    /// data of any type in or
    /// out of the client. Ports may be connected in various ways.
    ///
    /// The `port_spec` specifies the IO direction and data type. Oftentimes,
    /// the built-in types
    /// (`AudioInSpec`, `AudioOutSpec`, `MidiInSpec`, `MidiOutSpec`) can be
    /// used.
    ///
    /// Each port has a short name. The port's full name contains the name of
    /// the client concatenated with a colon (:) followed by its short
    /// name. `Port::name_size()` is the maximum length of the full
    /// name. Exceeding that will cause the port registration to fail and return
    /// `Err(())`.
    ///
    /// The `port_name` must be unique among all ports owned by this client. If
    /// the name is not unique, the registration will fail.
    pub fn register_port<PS: PortSpec>(
        &self,
        port_name: &str,
        port_spec: PS,
    ) -> Result<Port<PS>, JackErr> {
        let port_name_c = ffi::CString::new(port_name).unwrap();
        let port_type_c = ffi::CString::new(port_spec.jack_port_type()).unwrap();
        let port_flags = port_spec.jack_flags().bits();
        let buffer_size = port_spec.jack_buffer_size();
        let pp = unsafe {
            j::jack_port_register(
                self.as_ptr(),
                port_name_c.as_ptr(),
                port_type_c.as_ptr(),
                port_flags as libc::c_ulong,
                buffer_size,
            )
        };
        if pp.is_null() {
            Err(JackErr::PortRegistrationError(port_name.to_string()))
        } else {
            Ok(unsafe { Port::from_raw(port_spec, self.as_ptr(), pp) })
        }
    }



    // Get a `Port` by its port id.
    pub fn port_by_id(&self, port_id: pt::JackPortId) -> Option<UnownedPort> {
        let pp = unsafe { j::jack_port_by_id(self.as_ptr(), port_id) };
        if pp.is_null() {
            None
        } else {
            Some(unsafe { Port::from_raw(Unowned {}, self.as_ptr(), pp) })
        }
    }

    /// Get a `Port` by its port name.
    pub fn port_by_name(&self, port_name: &str) -> Option<UnownedPort> {
        let port_name = ffi::CString::new(port_name).unwrap();
        let pp = unsafe { j::jack_port_by_name(self.as_ptr(), port_name.as_ptr()) };
        if pp.is_null() {
            None
        } else {
            Some(unsafe { Port::from_raw(Unowned {}, self.as_ptr(), pp) })
        }
    }

    /// The estimated time in frames that has passed since the JACK server
    /// began the current process
    /// cycle.
    pub fn frames_since_cycle_start(&self) -> pt::JackFrames {
        unsafe { j::jack_frames_since_cycle_start(self.as_ptr()) }
    }

    /// The estimated current time in frames. This function is intended for use
    /// in other threads
    /// (not the process callback). The return value can be compared with the
    /// value of
    /// `last_frame_time` to relate time in other threads to JACK time. To
    /// obtain better time
    /// information from within the process callback, see `ProcessScope`.
    ///
    /// # TODO
    /// - test
    pub fn frame_time(&self) -> pt::JackFrames {
        unsafe { j::jack_frame_time(self.as_ptr()) }
    }

    /// The estimated time in microseconds of the specified frame time
    ///
    /// # TODO
    /// - Improve test
    pub fn frames_to_time(&self, n_frames: pt::JackFrames) -> pt::JackTime {
        unsafe { j::jack_frames_to_time(self.as_ptr(), n_frames) }
    }

    /// The estimated time in frames for the specified system time.
    ///
    /// # TODO
    /// - Improve test
    pub fn time_to_frames(&self, t: pt::JackTime) -> pt::JackFrames {
        unsafe { j::jack_time_to_frames(self.as_ptr(), t) }
    }

    /// Returns `true` if the port `port` belongs to this client.
    pub fn is_mine<PS: PortSpec>(&self, port: &Port<PS>) -> bool {
        match unsafe { j::jack_port_is_mine(self.as_ptr(), port.as_ptr()) } {
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
    pub fn request_monitor_by_name(
        &self,
        port_name: &str,
        enable_monitor: bool,
    ) -> Result<(), JackErr> {
        let port_name_cstr = ffi::CString::new(port_name).unwrap();
        let res = unsafe {
            j::jack_port_request_monitor_by_name(
                self.as_ptr(),
                port_name_cstr.as_ptr(),
                if enable_monitor { 1 } else { 0 },
            )
        };
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
    // pub pub fn set_freewheel(&self, enable: bool) -> Result<(), JackErr> {
    //     let onoff = match enable {
    //         true => 0,
    //         false => 1,
    //     };
    //     match unsafe { j::jack_set_freewheel(self.as_ptr(), onoff) } {
    //         0 => Ok(()),
    //         _ => Err(JackErr::FreewheelError),
    //     }
    // }



    /// Establish a connection between two ports by their full name.
    ///
    /// When a connection exists, data written to the source port will be
    /// available to be read at the destination port.
    ///
    /// On failure, either a `PortAlreadyConnected` or `PortConnectionError` is
    /// returned.
    ///
    /// # Preconditions
    /// 1. The port types must be identical
    /// 2. The port flags of the `source_port` must include `IS_OUTPUT`
    /// 3. The port flags of the `destination_port` must include `IS_INPUT`.
    /// 4. Both ports must be owned by active clients.
    pub fn connect_ports_by_name(
        &self,
        source_port: &str,
        destination_port: &str,
    ) -> Result<(), JackErr> {
        let source_cstr = ffi::CString::new(source_port).unwrap();
        let destination_cstr = ffi::CString::new(destination_port).unwrap();

        let res = unsafe {
            j::jack_connect(
                self.as_ptr(),
                source_cstr.as_ptr(),
                destination_cstr.as_ptr(),
            )
        };
        match res {
            0 => Ok(()),
            ::libc::EEXIST => {
                Err(JackErr::PortAlreadyConnected(
                    source_port.to_string(),
                    destination_port.to_string(),
                ))
            }
            _ => {
                Err(JackErr::PortConnectionError(
                    source_port.to_string(),
                    destination_port.to_string(),
                ))
            }
        }
    }

    /// Establish a connection between two ports.
    ///
    /// When a connection exists, data written to the source port will be
    /// available to be read at the destination port.
    ///
    /// On failure, either a `PortAlreadyConnected` or `PortConnectionError` is
    /// returned.
    ///
    /// # Preconditions
    /// 1. The port types must be identical
    /// 2. The port flags of the `source_port` must include `IS_OUTPUT`
    /// 3. The port flags of the `destination_port` must include `IS_INPUT`.
    /// 4. Both ports must be owned by active clients.
    pub fn connect_ports<A: PortSpec, B: PortSpec>(
        &self,
        source_port: &Port<A>,
        destination_port: &Port<B>,
    ) -> Result<(), JackErr> {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();
        self.connect_ports_by_name(source_port.name(), destination_port.name())
    }

    /// Remove a connection between two ports.
    pub fn disconnect_ports<A: PortSpec, B: PortSpec>(
        &self,
        source: &Port<A>,
        destination: &Port<B>,
    ) -> Result<(), JackErr> {
        self.disconnect_ports_by_name(source.name(), destination.name())
    }

    /// Remove a connection between two ports.
    pub fn disconnect_ports_by_name(
        &self,
        source_port: &str,
        destination_port: &str,
    ) -> Result<(), JackErr> {
        let source_port = ffi::CString::new(source_port).unwrap();
        let destination_port = ffi::CString::new(destination_port).unwrap();
        let res = unsafe {
            j::jack_disconnect(
                self.as_ptr(),
                source_port.as_ptr(),
                destination_port.as_ptr(),
            )
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
    pub unsafe fn type_buffer_size(&self, port_type: &str) -> usize {
        let port_type = ffi::CString::new(port_type).unwrap();
        let n = j::jack_port_type_get_buffer_size(self.as_ptr(), port_type.as_ptr());
        n
    }

    /// Expose the underlying ffi pointer.
    ///
    /// This is mostly for use within the jack crate itself.
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut j::jack_client_t {
        self.0
    }

    /// Create a `Client` from an ffi pointer.
    ///
    /// This is mostly for use within the jack crate itself.
    pub unsafe fn from_raw(p: *mut j::jack_client_t) -> Self {
        Client(p)
    }
}

/// Close the client.
impl Drop for Client {
    fn drop(&mut self) {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();

        debug_assert!(!self.as_ptr().is_null()); // Rep invariant
        // Close the client
        sleep_on_test();
        let res = unsafe { j::jack_client_close(self.as_ptr()) }; // close the client
        sleep_on_test();
        assert_eq!(res, 0);
        self.0 = ptr::null_mut();
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", DebugInfo::new(self))
    }
}

/// `ProcessScope` provides information on the client and frame time
/// information within a process
/// callback.
#[derive(Debug)]
pub struct ProcessScope {
    client_ptr: *mut j::jack_client_t,

    // Used to allow safe access to IO port buffers
    n_frames: pt::JackFrames,
}

impl ProcessScope {
    /// The number of frames in the current process cycle.
    #[inline(always)]
    pub fn n_frames(&self) -> pt::JackFrames {
        self.n_frames
    }

    /// The precise time at the start of the current process cycle. This
    /// function may only be used
    /// from the process callback, and can be used to interpret timestamps
    /// generated by
    /// `self.frame_time()` in other threads, with respect to the current
    /// process cycle.
    pub fn last_frame_time(&self) -> pt::JackFrames {
        unsafe { j::jack_last_frame_time(self.client_ptr()) }
    }

    /// The estimated time in frames that has passed since the JACK server
    /// began the current process
    /// cycle.
    pub fn frames_since_cycle_start(&self) -> pt::JackFrames {
        unsafe { j::jack_frames_since_cycle_start(self.client_ptr()) }
    }

    /// Provides the internal cycle timing information as used by most of the
    /// other time related
    /// functions. This allows the caller to map between frame counts and
    /// microseconds with full
    /// precision (i.e. without rounding frame times to integers), and also
    /// provides e.g. the
    /// microseconds time of the start of the current cycle directly (it has to
    /// be computed
    /// otherwise).
    ///
    /// `Err(JackErr::TimeError)` is returned on failure.
    /// `Err(JackErr::WeakFunctionNotFound)` if the function does not exist.
    pub fn cycle_times(&self) -> Result<CycleTimes, JackErr> {
        let mut current_frames: pt::JackFrames = 0;
        let mut current_usecs: pt::JackTime = 0;
        let mut next_usecs: pt::JackTime = 0;
        let mut period_usecs: libc::c_float = 0.0;

        let jack_get_cycle_times = {
            match *j::jack_get_cycle_times {
                Some(f) => f,
                None => return Err(JackErr::WeakFunctionNotFound),
            }
        };
        let res = unsafe {
            (jack_get_cycle_times)(
                self.client_ptr(),
                &mut current_frames,
                &mut current_usecs,
                &mut next_usecs,
                &mut period_usecs,
            )
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

    /// Expose the `client_ptr` for low level purposes.
    ///
    /// This is mostly for use within the jack crate itself.
    #[inline(always)]
    pub fn client_ptr(&self) -> *mut j::jack_client_t {
        self.client_ptr
    }

    /// Create a `ProcessScope` for the client with the given pointer and the
    /// specified amount of
    /// frames.
    ///
    /// This is mostly for use within the jack crate itself.
    pub unsafe fn from_raw(n_frames: pt::JackFrames, client_ptr: *mut j::jack_client_t) -> Self {
        ProcessScope {
            n_frames: n_frames,
            client_ptr: client_ptr,
        }
    }
}

/// Internal cycle timing information.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CycleTimes {
    pub current_frames: pt::JackFrames,
    pub current_usecs: pt::JackTime,
    pub next_usecs: pt::JackTime,
    pub period_usecs: libc::c_float,
}

#[derive(Debug)]
struct DebugInfo {
    name: String,
    sample_rate: usize,
    buffer_size: u32,
    cpu_usage: String,
    ports: Vec<String>,
    frame_time: pt::JackFrames,
}

impl DebugInfo {
    fn new(c: &Client) -> DebugInfo {
        DebugInfo {
            name: c.name().into(),
            sample_rate: c.sample_rate(),
            buffer_size: c.buffer_size(),
            cpu_usage: format!("{}%", c.cpu_load() / 100.0),
            ports: c.ports(None, None, PortFlags::empty()),
            frame_time: c.frame_time(),
        }
    }
}
