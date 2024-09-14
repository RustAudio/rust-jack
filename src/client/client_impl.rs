use jack_sys as j;
use std::fmt::Debug;
use std::sync::Arc;
use std::{ffi, fmt, ptr};

use crate::client::common::{sleep_on_test, CREATE_OR_DESTROY_CLIENT_MUTEX};
use crate::jack_enums::CodeOrMessage;
use crate::jack_utils::collect_strs;
use crate::properties::PropertyChangeHandler;
use crate::transport::Transport;
use crate::{
    AsyncClient, ClientOptions, ClientStatus, Error, Frames, NotificationHandler, Port, PortFlags,
    PortId, PortSpec, ProcessHandler, Time, Unowned,
};

/// A client to interact with a JACK server.
///
/// # Example
/// ```
/// let c_res = jack::Client::new("rusty_client", jack::ClientOptions::default());
/// match c_res {
///     Ok((client, status)) => println!(
///         "Managed to open client {}, with
/// status {:?}!",
///         client.name(),
///         status
///     ),
///     Err(e) => println!("Failed to open client because of error: {:?}", e),
/// };
/// ```

pub type InternalClientID = j::jack_intclient_t;

#[allow(dead_code)]
pub struct Client(
    *mut j::jack_client_t,
    Arc<()>,
    Option<Box<dyn PropertyChangeHandler>>,
);

unsafe impl Send for Client {}
unsafe impl Sync for Client {}

impl Client {
    /// Opens a JACK client with the given name and options. If the client is successfully opened,
    /// then `Ok(client)` is returned. If the JACK server returned an error, then
    /// `Err(Error::ClientError(status))` will be returned. And if the `dynamic_loading` feature is
    /// enabled and the JACK library could not be loaded, an `Err(Error::LibraryError(message))` is
    /// returned.
    ///
    /// Although the client may be successful in opening, there still may be some errors minor
    /// errors when attempting to opening. To access these, check the returned `ClientStatus`.
    pub fn new(client_name: &str, options: ClientOptions) -> Result<(Self, ClientStatus), Error> {
        let _m = CREATE_OR_DESTROY_CLIENT_MUTEX.lock().ok();

        // All of the jack_sys functions below assume the client library is loaded and will panic if
        // it is not
        #[cfg(feature = "dynamic_loading")]
        if let Err(err) = jack_sys::library() {
            return Err(Error::LibraryError(err.to_string()));
        }

        crate::logging::maybe_init_logging();
        sleep_on_test();
        let mut status_bits = 0;
        let client = unsafe {
            let client_name = ffi::CString::new(client_name).unwrap();
            j::jack_client_open(client_name.as_ptr(), options.bits(), &mut status_bits)
        };
        sleep_on_test();
        let status = ClientStatus::from_bits(status_bits).unwrap_or_else(ClientStatus::empty);
        if client.is_null() {
            Err(Error::ClientError(status))
        } else {
            sleep_on_test();
            Ok((Client(client, Arc::default(), None), status))
        }
    }

    /// Begin processing in real-time using the specified `NotificationHandler` and
    /// `ProcessHandler`.
    pub fn activate_async<N, P>(
        self,
        notification_handler: N,
        process_handler: P,
    ) -> Result<AsyncClient<N, P>, Error>
    where
        N: 'static + Send + Sync + NotificationHandler,
        P: 'static + Send + ProcessHandler,
    {
        AsyncClient::new(self, notification_handler, process_handler)
    }

    /// Return JACK's current system time in microseconds, using the JACK clock
    /// source.
    ///
    /// Note: Although attached a `Client` method, this should use the same time clock as all
    /// clients.
    pub fn time(&self) -> Time {
        // Despite not needing a ptr to the client, this function often segfaults if a client has
        // not been initialized.
        unsafe { jack_sys::jack_get_time() }
    }

    /// The sample rate of the JACK system, as set by the user when jackd was
    /// started.
    pub fn sample_rate(&self) -> usize {
        let srate = unsafe { j::jack_get_sample_rate(self.raw()) };
        srate as usize
    }

    /// The current CPU load estimated by JACK. It is on a scale of `0.0` to `100.0`.
    ///
    /// This is a running average of the time it takes to execute a full process cycle for all
    /// clients as a percentage of the real time available per cycle determined by the buffer size
    /// and sample rate.
    pub fn cpu_load(&self) -> f32 {
        unsafe { j::jack_cpu_load(self.raw()) }
    }

    /// Get the name of the current client. This may differ from the name requested by `Client::new`
    /// as JACK will may rename a client if necessary (ie: name collision, name too long). The name
    /// will only the be different than the one passed to `Client::new` if the `ClientStatus` was
    /// `NAME_NOT_UNIQUE`.
    pub fn name(&self) -> &str {
        unsafe {
            let ptr = j::jack_get_client_name(self.raw());
            let cstr = ffi::CStr::from_ptr(ptr);
            cstr.to_str().unwrap()
        }
    }

    /// The current maximum size that will every be passed to the process
    /// callback.
    pub fn buffer_size(&self) -> Frames {
        unsafe { j::jack_get_buffer_size(self.raw()) }
    }

    /// Change the buffer size passed to the process callback.
    ///
    /// This operation stops the JACK engine process cycle, then calls all registered buffer size
    /// callback functions before restarting the process cycle. This will cause a gap in the audio
    /// flow, so it should only be done at appropriate stopping points.
    pub fn set_buffer_size(&self, n_frames: Frames) -> Result<(), Error> {
        let res = unsafe { j::jack_set_buffer_size(self.raw(), n_frames) };
        match res {
            0 => Ok(()),
            _ => Err(Error::SetBufferSizeError),
        }
    }

    /// Get the numeric `uuid` of this client.
    ///
    /// # Remarks
    ///
    /// * Deallocates, not realtime safe.
    pub fn uuid(&self) -> j::jack_uuid_t {
        unsafe {
            let mut uuid: j::jack_uuid_t = Default::default();
            let uuid_s = j::jack_client_get_uuid(self.raw());
            assert!(!uuid_s.is_null());
            assert_eq!(0, j::jack_uuid_parse(uuid_s, &mut uuid));
            j::jack_free(uuid_s as _);
            uuid
        }
    }

    /// Get the numeric `uuid` of a client by name; returns None if client does not exist
    /// # Remarks
    /// * Not realtime safe
    pub fn uuid_of_client_by_name(&self, name: &str) -> Option<jack_sys::jack_uuid_t> {
        Self::uuid_of_client_by_name_raw(self.raw(), name)
    }

    pub(crate) fn uuid_of_client_by_name_raw(
        raw: *mut jack_sys::jack_client_t,
        name: &str,
    ) -> Option<jack_sys::jack_uuid_t> {
        let name = ffi::CString::new(name).unwrap();
        let mut uuid: jack_sys::jack_uuid_t = Default::default();
        unsafe {
            let uuid_s = jack_sys::jack_get_uuid_for_client_name(raw, name.as_ptr());
            if uuid_s.is_null() {
                return None;
            }
            assert_eq!(0, jack_sys::jack_uuid_parse(uuid_s, &mut uuid));
            jack_sys::jack_free(uuid_s as _);
        }
        Some(uuid)
    }
    /// Get a String representation of the `uuid` of this client.
    ///
    /// # Remarks
    ///
    /// * Allocates & deallocates, not realtime safe.
    pub fn uuid_string(&self) -> String {
        unsafe {
            let uuid_s = j::jack_client_get_uuid(self.raw());
            assert!(!uuid_s.is_null());
            let uuid = ffi::CStr::from_ptr(uuid_s)
                .to_str()
                .expect("uuid is valid string")
                .to_string();
            j::jack_free(uuid_s as _);
            uuid
        }
    }

    //helper to get client name from uuid string.
    unsafe fn name_by_uuid_raw(&self, uuid: *const ::libc::c_char) -> Option<String> {
        let name_ptr = j::jack_get_client_name_by_uuid(self.raw(), uuid);
        if name_ptr.is_null() {
            None
        } else {
            Some(
                ffi::CStr::from_ptr(name_ptr)
                    .to_str()
                    .expect("name convert to a valid str")
                    .to_string(),
            )
        }
    }

    /// Get the name of a client by its numeric uuid.
    pub fn name_by_uuid(&self, uuid: j::jack_uuid_t) -> Option<String> {
        let mut uuid_s = ['\0' as _; 37]; //jack_uuid_unparse expects an array of length 37
        unsafe {
            j::jack_uuid_unparse(uuid, uuid_s.as_mut_ptr());
            self.name_by_uuid_raw(uuid_s.as_ptr())
        }
    }

    /// Get the name of a client by its `&str` uuid.
    pub fn name_by_uuid_str(&self, uuid: &str) -> Option<String> {
        let uuid = ffi::CString::new(uuid).unwrap();
        unsafe { self.name_by_uuid_raw(uuid.as_ptr()) }
    }

    /// Returns a vector of port names that match the specified arguments
    ///
    /// `port_name_pattern` - A regular expression used to select ports by name. If `None` or zero
    /// lengthed, no selection based on name will be carried out.
    ///
    /// `type_name_pattern` - A regular expression used to select ports by type. If `None` or zero
    /// lengthed, no selection based on type will be carried out. The port type is the same one
    /// returned by `PortSpec::jack_port_type()`. For example, `AudioIn` and `AudioOut` are both of
    /// type `"32 bit float mono audio"`.
    ///
    /// `flags` - A value used to select ports by their flags. Use `PortFlags::empty()` for no flag
    /// selection.
    pub fn ports(
        &self,
        port_name_pattern: Option<&str>,
        type_name_pattern: Option<&str>,
        flags: PortFlags,
    ) -> Vec<String> {
        let port_name_pattern_cstr = ffi::CString::new(port_name_pattern.unwrap_or("")).unwrap();
        let type_name_pattern_cstr = ffi::CString::new(type_name_pattern.unwrap_or("")).unwrap();
        let flags = libc::c_ulong::from(flags.bits());
        unsafe {
            let ports = j::jack_get_ports(
                self.raw(),
                port_name_pattern_cstr.as_ptr(),
                type_name_pattern_cstr.as_ptr(),
                flags,
            );
            collect_strs(ports)
        }
    }

    /// Create a new port for the client. This is an object used for moving data of any type in or
    /// out of the client. Ports may be connected in various ways.
    ///
    /// The `port_spec` specifies the IO direction and data type. Oftentimes, the built-in types
    /// (`AudioIn`, `AudioOut`, `MidiIn`, `MidiOut`) can be used.
    ///
    /// Each port has a short name. The port's full name contains the name of the client
    /// concatenated with a colon (:) followed by its short name. `Port::name_size()` is the maximum
    /// length of the full name. Exceeding that will cause the port registration to fail and return
    /// `Err(())`.
    ///
    /// The `port_name` must be unique among all ports owned by this client. If the name is not
    /// unique, the registration will fail.
    pub fn register_port<PS: PortSpec>(
        &self,
        port_name: &str,
        port_spec: PS,
    ) -> Result<Port<PS>, Error> {
        let port_name_c = ffi::CString::new(port_name).unwrap();
        let port_type_c = ffi::CString::new(port_spec.jack_port_type()).unwrap();
        let port_flags = port_spec.jack_flags().bits();
        let buffer_size = port_spec.jack_buffer_size();
        let pp = unsafe {
            j::jack_port_register(
                self.raw(),
                port_name_c.as_ptr(),
                port_type_c.as_ptr(),
                libc::c_ulong::from(port_flags),
                buffer_size,
            )
        };
        if pp.is_null() {
            Err(Error::PortRegistrationError(port_name.to_string()))
        } else {
            Ok(unsafe { Port::from_raw(port_spec, self.raw(), pp, Arc::downgrade(&self.1)) })
        }
    }

    /// Get a `Port` by its port id.
    pub fn port_by_id(&self, port_id: PortId) -> Option<Port<Unowned>> {
        let pp = unsafe { j::jack_port_by_id(self.raw(), port_id) };
        if pp.is_null() {
            None
        } else {
            Some(unsafe { Port::from_raw(Unowned {}, self.raw(), pp, Arc::downgrade(&self.1)) })
        }
    }

    /// Get a `Port` by its port name.
    pub fn port_by_name(&self, port_name: &str) -> Option<Port<Unowned>> {
        let port_name = ffi::CString::new(port_name).unwrap();
        let pp = unsafe { j::jack_port_by_name(self.raw(), port_name.as_ptr()) };
        if pp.is_null() {
            None
        } else {
            Some(unsafe { Port::from_raw(Unowned {}, self.raw(), pp, Arc::downgrade(&self.1)) })
        }
    }

    /// Load a (server) internal client
    ///
    /// This call will load a server-internal jack client. Internal clients run inside the jack
    /// server process and are provided as .so files by the jack package. Most internal clients
    /// provide low level interfaces, so as such are not expected to be needed by regular applications
    ///
    /// `client_name` is the name the new client will display as on the graph
    /// `client_bin_name` is the name of the internal client to load. This is the same as the .so file
    /// being loaded, without its file extension or path.
    /// `client_args` is an arbitrary string of parameters passed to the client. These are client specific
    ///
    /// This call will return the ID of the new client, needed to unload it again on success.
    /// It returns a ClientError on error.
    pub fn load_internal_client(
        &self,
        client_name: &str,
        client_bin_name: &str,
        client_args: &str,
    ) -> Result<InternalClientID, Error> {
        let ffi_client_name = ffi::CString::new(client_name).unwrap();
        let ffi_client_bin = ffi::CString::new(client_bin_name).unwrap();
        let ffi_client_args = ffi::CString::new(client_args).unwrap();

        let mut status_bits = 0;
        let options: j::Enum_JackOptions = j::JackLoadName | j::JackLoadInit;

        let intclient = unsafe {
            j::jack_internal_client_load(
                self.raw(),
                ffi_client_name.as_ptr(),
                options,
                &mut status_bits,
                ffi_client_bin.as_ptr(),
                ffi_client_args.as_ptr(),
            )
        };

        match intclient {
            Some(0) => {
                let status =
                    ClientStatus::from_bits(status_bits).unwrap_or_else(ClientStatus::empty);
                Err(Error::ClientError(status))
            }
            Some(i) => Ok(i),
            None => Err(Error::WeakFunctionNotFound("jack_internal_client_load")),
        }
    }

    /// Unload a (server) internal client
    ///
    /// This call will unload a server-internal jack client with an ID given from load_internal_client.
    ///
    /// `client` is the ID to unload
    ///
    /// This call will return `Ok(())` on success.
    /// It returns a ClientError on error.
    pub fn unload_internal_client(&self, client: InternalClientID) -> Result<(), Error> {
        let status = unsafe {
            let status = match j::jack_internal_client_unload(self.raw(), client) {
                Some(s) => s,
                None => return Err(Error::WeakFunctionNotFound("jack_internal_client_unload")),
            };
            ClientStatus::from_bits_retain(status)
        };
        if status.is_empty() {
            Ok(())
        } else {
            Err(Error::ClientError(status))
        }
    }

    /// The estimated time in frames that has passed since the JACK server began the current process
    /// cycle.
    pub fn frames_since_cycle_start(&self) -> Frames {
        unsafe { j::jack_frames_since_cycle_start(self.raw()) }
    }

    /// The estimated current time in frames. This function is intended for use in other threads
    /// (not the process callback). The return value can be compared with the value of
    /// `last_frame_time` to relate time in other threads to JACK time. To obtain better time
    /// information from within the process callback, see `ProcessScope`.
    pub fn frame_time(&self) -> Frames {
        unsafe { j::jack_frame_time(self.raw()) }
    }

    /// The estimated time in microseconds of the specified frame time
    pub fn frames_to_time(&self, n_frames: Frames) -> Time {
        unsafe { j::jack_frames_to_time(self.raw(), n_frames) }
    }

    /// The estimated time in frames for the specified system time.
    ///
    /// # TODO
    /// - Improve test
    pub fn time_to_frames(&self, t: Time) -> Frames {
        unsafe { j::jack_time_to_frames(self.raw(), t) }
    }

    /// Returns `true` if the port `port` belongs to this client.
    pub fn is_mine<PS: PortSpec>(&self, port: &Port<PS>) -> bool {
        matches!(unsafe { j::jack_port_is_mine(self.raw(), port.raw()) }, 1)
    }

    /// Toggle input monitoring for the port with name `port_name`.
    ///
    /// `Err(Error::PortMonitorError)` is returned on failure.
    ///
    /// Only works if the port has the `CAN_MONITOR` flag, or else nothing happens.
    pub fn request_monitor_by_name(
        &self,
        port_name: &str,
        enable_monitor: bool,
    ) -> Result<(), Error> {
        let port_name_cstr = ffi::CString::new(port_name).unwrap();
        let res = unsafe {
            j::jack_port_request_monitor_by_name(
                self.raw(),
                port_name_cstr.as_ptr(),
                i32::from(enable_monitor),
            )
        };
        match res {
            0 => Ok(()),
            _ => Err(Error::PortMonitorError),
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
    // pub pub fn set_freewheel(&self, enable: bool) -> Result<(), Error> {
    //     let onoff = match enable {
    //         true => 0,
    //         false => 1,
    //     };
    //     match unsafe { j::jack_set_freewheel(self.raw(), onoff) } {
    //         0 => Ok(()),
    //         _ => Err(Error::FreewheelError),
    //     }
    // }

    /// Establish a connection between two ports by their full name.
    ///
    /// When a connection exists, data written to the source port will be available to be read at
    /// the destination port.
    ///
    /// On failure, either a `PortAlreadyConnected` or `PortConnectionError` is returned.
    ///
    /// # Preconditions
    /// 1. The port types must be identical
    /// 2. The port flags of the `source_port` must include `IS_OUTPUT`
    /// 3. The port flags of the `destination_port` must include `IS_INPUT`.
    /// 4. Both ports must be owned by active clients.
    ///
    /// # Panics
    /// Panics if it is not possible to convert `source_port` or `destination_port` to a `CString`.
    pub fn connect_ports_by_name(
        &self,
        source_port: &str,
        destination_port: &str,
    ) -> Result<(), Error> {
        let source_cstr = ffi::CString::new(source_port).unwrap();
        let destination_cstr = ffi::CString::new(destination_port).unwrap();
        let res =
            unsafe { j::jack_connect(self.raw(), source_cstr.as_ptr(), destination_cstr.as_ptr()) };
        match res {
            0 => Ok(()),
            ::libc::EEXIST => Err(Error::PortAlreadyConnected(
                source_port.to_string(),
                destination_port.to_string(),
            )),
            code => {
                let code_or_message = if self
                    .port_by_name(source_port)
                    .map(|p| p.flags().contains(PortFlags::IS_INPUT))
                    .unwrap_or(false)
                {
                    CodeOrMessage::Message(
                        "source port does not produce a signal, it is not an input port",
                    )
                } else if self
                    .port_by_name(destination_port)
                    .map(|p| p.flags().contains(PortFlags::IS_OUTPUT))
                    .unwrap_or(false)
                {
                    CodeOrMessage::Message(
                        "destination port cannot be written to, it is not an output port",
                    )
                } else {
                    CodeOrMessage::Code(code)
                };
                Err(Error::PortConnectionError {
                    source: source_port.to_string(),
                    destination: destination_port.to_string(),
                    code_or_message,
                })
            }
        }
    }

    /// Establish a connection between two ports.
    ///
    /// When a connection exists, data written to the source port will be available to be read at
    /// the destination port.
    ///
    /// On failure, either a `PortAlreadyConnected` or `PortConnectionError` is returned.
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
    ) -> Result<(), Error> {
        let _m = CREATE_OR_DESTROY_CLIENT_MUTEX.lock().ok();
        self.connect_ports_by_name(&source_port.name()?, &destination_port.name()?)
    }

    /// Remove all connections to/from the port.
    pub fn disconnect<PS>(&self, port: &Port<PS>) -> Result<(), Error> {
        let res = unsafe { j::jack_port_disconnect(self.raw(), port.raw()) };
        match res {
            0 => Ok(()),
            _ => Err(Error::PortDisconnectionError),
        }
    }

    pub fn unregister_port<PS>(&self, port: Port<PS>) -> Result<(), Error> {
        let res = unsafe { j::jack_port_unregister(self.raw(), port.raw()) };
        match res {
            0 => Ok(()),
            _ => Err(Error::PortDisconnectionError),
        }
    }

    /// Remove a connection between two ports.
    pub fn disconnect_ports<A: PortSpec, B: PortSpec>(
        &self,
        source: &Port<A>,
        destination: &Port<B>,
    ) -> Result<(), Error> {
        self.disconnect_ports_by_name(&source.name()?, &destination.name()?)
    }

    /// Remove a connection between two ports.
    pub fn disconnect_ports_by_name(
        &self,
        source_port: &str,
        destination_port: &str,
    ) -> Result<(), Error> {
        let source_port = ffi::CString::new(source_port).unwrap();
        let destination_port = ffi::CString::new(destination_port).unwrap();
        let res = unsafe {
            j::jack_disconnect(self.raw(), source_port.as_ptr(), destination_port.as_ptr())
        };
        match res {
            0 => Ok(()),
            _ => Err(Error::PortDisconnectionError),
        }
    }

    /// The buffer size of a port type
    ///
    /// # Safety
    ///
    /// * This function may only be called in a buffer size callback.
    pub unsafe fn type_buffer_size(&self, port_type: &str) -> usize {
        let port_type = ffi::CString::new(port_type).unwrap();
        j::jack_port_type_get_buffer_size(self.raw(), port_type.as_ptr())
    }

    /// Expose the underlying ffi pointer.
    ///
    /// This is mostly for use within the jack crate itself.
    #[inline(always)]
    pub fn raw(&self) -> *mut j::jack_client_t {
        self.0
    }

    /// Create a `Client` from an ffi pointer.
    ///
    /// This is mostly for use within the jack crate itself.
    ///
    /// # Safety
    /// It is unsafe to create a `Client` from a raw pointer.
    pub unsafe fn from_raw(p: *mut j::jack_client_t) -> Self {
        Client(p, Arc::default(), None)
    }

    /// Get a `Transport` object associated with this client.
    ///
    /// # Remarks
    /// * The transport methods will only work during this client's lifetime.
    pub fn transport(&self) -> Transport {
        Transport {
            client_ptr: self.0,
            client_life: Arc::downgrade(&self.1),
        }
    }

    /// Register a property change handler for this client.
    ///
    /// # Remarks
    /// * The handler isn't called until after this client is activated.
    ///
    /// # Panics
    /// Calling this method more than once on any given client with cause a panic.
    pub fn register_property_change_handler<H: 'static + PropertyChangeHandler>(
        &mut self,
        handler: H,
    ) -> Result<(), Error> {
        assert!(self.2.is_none());
        let handler = Box::into_raw(Box::new(handler));
        unsafe {
            self.2 = Some(Box::from_raw(handler));
            let res = j::jack_set_property_change_callback(
                self.raw(),
                Some(crate::properties::property_changed::<H>),
                std::mem::transmute::<*mut H, *mut libc::c_void>(handler),
            );
            match res {
                0 => Ok(()),
                error_code => Err(Error::UnknownError { error_code }),
            }
        }
    }
}

/// Close the client.
impl Drop for Client {
    fn drop(&mut self) {
        let _m = CREATE_OR_DESTROY_CLIENT_MUTEX.lock().ok();
        debug_assert!(!self.raw().is_null()); // Rep invariant
                                              // Close the client
        sleep_on_test();
        let _res = unsafe { j::jack_client_close(self.raw()) }; // best effort: close the client
        sleep_on_test();
        //assert_eq!(res, 0); //do not assert here. connection could be broken
        self.0 = ptr::null_mut();
    }
}

impl Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Client")
            .field("name", &self.name())
            .field("sample_rate", &self.sample_rate())
            .field("buffer_size", &self.buffer_size())
            .field("cpu_usage", &format!("{}%", self.cpu_load() / 100.0))
            .field("frame_time", &self.frame_time())
            .finish()
    }
}

/// `ProcessScope` provides information on the client and frame time information within a process
/// callback.
#[derive(Debug)]
pub struct ProcessScope {
    client_ptr: *mut j::jack_client_t,

    // Used to allow safe access to IO port buffers
    n_frames: Frames,
}

impl ProcessScope {
    /// The number of frames in the current process cycle.
    #[inline(always)]
    pub fn n_frames(&self) -> Frames {
        self.n_frames
    }

    /// The precise time at the start of the current process cycle. This function may only be used
    /// from the process callback, and can be used to interpret timestamps generated by
    /// `self.frame_time()` in other threads, with respect to the current process cycle.
    pub fn last_frame_time(&self) -> Frames {
        unsafe { j::jack_last_frame_time(self.client_ptr()) }
    }

    /// The estimated time in frames that has passed since the JACK server began the current process
    /// cycle.
    pub fn frames_since_cycle_start(&self) -> Frames {
        unsafe { j::jack_frames_since_cycle_start(self.client_ptr()) }
    }

    /// Provides the internal cycle timing information as used by most of the other time related
    /// functions. This allows the caller to map between frame counts and microseconds with full
    /// precision (i.e. without rounding frame times to integers), and also provides e.g. the
    /// microseconds time of the start of the current cycle directly (it has to be computed
    /// otherwise).
    ///
    /// `Err(Error::TimeError)` is returned on failure.
    /// `Err(Error::WeakFunctionNotFound)` if the function does not exist.
    pub fn cycle_times(&self) -> Result<CycleTimes, Error> {
        let mut current_frames: Frames = 0;
        let mut current_usecs: Time = 0;
        let mut next_usecs: Time = 0;
        let mut period_usecs: libc::c_float = 0.0;

        let res = unsafe {
            (jack_sys::jack_get_cycle_times)(
                self.client_ptr(),
                &mut current_frames,
                &mut current_usecs,
                &mut next_usecs,
                &mut period_usecs,
            )
        };
        match res {
            None => Err(Error::WeakFunctionNotFound("jack_get_cycle_times")),
            Some(0) => Ok(CycleTimes {
                current_frames,
                current_usecs,
                next_usecs,
                period_usecs,
            }),
            _ => Err(Error::TimeError),
        }
    }

    /// Expose the `client_ptr` for low level purposes.
    ///
    /// This is mostly for use within the jack crate itself.
    #[inline(always)]
    pub fn client_ptr(&self) -> *mut j::jack_client_t {
        self.client_ptr
    }

    /// Create a `ProcessScope` for the client with the given pointer and the specified amount of
    /// frames.
    ///
    /// This is mostly for use within the jack crate itself.
    ///
    /// # Safety
    /// It is unsafe to create a `ProcessScope` since it may not be valid. For library user's, the
    /// `ProcessScope` is usually passed in as a parameter to a trait's method.
    pub unsafe fn from_raw(n_frames: Frames, client_ptr: *mut j::jack_client_t) -> Self {
        ProcessScope {
            n_frames,
            client_ptr,
        }
    }
}

/// Internal cycle timing information.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CycleTimes {
    pub current_frames: Frames,
    pub current_usecs: Time,
    pub next_usecs: Time,
    pub period_usecs: libc::c_float,
}
