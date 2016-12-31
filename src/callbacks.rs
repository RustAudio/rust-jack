use std::{ffi, mem};
use libc::c_void;
use jack_sys as j;
use jack_enums::*;
use jack_flags::client_status::{ClientStatus, UNKNOWN_ERROR};

/// `ProcessScope` provides information on the client and frame
/// timings within a `process` callback.
#[derive(Debug)]
pub struct ProcessScope {
    // To be used _only_ for runtime verification that the client who wrote
    // that the only ports being used are ones created by the client whose
    // handler is being run.
    client_ptr: *mut j::jack_client_t,

    // Used to allow safe access to IO port buffers
    n_frames: u32,
}

impl ProcessScope {
    /// Create a `ProcessScope` for the client with the given pointer
    /// and the specified amount of frames.
    pub unsafe fn from_raw(n_frames: u32, client_ptr: *mut j::jack_client_t) -> Self {
        ProcessScope {
            n_frames: n_frames,
            client_ptr: client_ptr,
        }
    }

    #[inline(always)]
    pub fn n_frames(&self) -> u32 {
        self.n_frames
    }

    #[inline(always)]
    pub fn client_ptr(&self) -> *mut j::jack_client_t {
        self.client_ptr
    }
}

/// Specifies callbacks for JACK.
///
/// All callbacks happen on the same thread (not concurrently), unless otherwise
/// stated.
///
/// # TODO
/// * convert C enum return values to Rust enums.
pub trait JackHandler: Send {
    /// Called just once after the creation of the thread in which all other
    /// callbacks will be handled.
    ///
    /// It does not need to be suitable for real-time execution.
    fn thread_init(&mut self) {}

    /// Called when the JACK server shuts down the client thread. The function
    /// must be written as if it were an asynchronous POSIX signal handler ---
    /// use only async-safe functions, and remember that it is executed from
    /// another thread. A typical funcion might set a flag or write to a pipe so
    /// that the rest of the application knows that the JACK client thread has
    /// shut down.
    fn shutdown(&mut self, _status: ClientStatus, _reason: &str) {}

    /// Called whenever there is work to be done.
    ///
    /// It needs to be suitable for real-time execution. That means that it
    /// cannot call functions that might block for a long time. This includes
    /// all I/O functions (disk, TTY, network), malloc, free, printf,
    /// pthread_mutex_lock, sleep, wait, poll, select, pthread_join,
    /// pthread_cond_wait, etc, etc.
    ///
    /// Should return `0` on success, and non-zero on error.
    fn process(&mut self, process_scope: &ProcessScope) -> JackControl {
        let _ = process_scope;
        JackControl::Continue
    }

    /// Called whenever "freewheel" mode is entered or leaving.
    fn freewheel(&mut self, _is_freewheel_enabled: bool) {}

    /// Called whenever the size of the buffer that will be passed to `process`
    /// is about to change.
    fn buffer_size(&mut self, _size: u32) -> JackControl {
        JackControl::Continue
    }

    /// Called whenever the system sample rate changes.
    fn sample_rate(&mut self, _srate: u32) -> JackControl {
        JackControl::Continue
    }

    /// Called whenever a client is registered or unregistered
    fn client_registration(&mut self, _name: &str, _is_registered: bool) {}

    /// Called whenever a port is registered or unregistered
    fn port_registration(&mut self, _port_id: u32, _is_registered: bool) {}

    /// Called whenever a port is renamed.
    fn port_rename(&mut self, _port_id: u32, _old_name: &str, _new_name: &str) -> JackControl {
        JackControl::Continue
    }

    /// Called whenever ports are connected/disconnected to/from each other.
    fn ports_connected(&mut self, _port_id_a: u32, _port_id_b: u32, _are_connected: bool) {}

    /// Called whenever the processing graph is reordered.
    fn graph_reorder(&mut self) -> JackControl {
        JackControl::Continue
    }

    /// Called whenever an xrun occurs.
    ///
    /// An xrun is a buffer under or over run, which means some data has been
    /// missed.
    fn xrun(&mut self) -> JackControl {
        JackControl::Continue
    }

    /// Called whenever it is necessary to recompute the latencies for some or
    /// all JACK ports.
    ///
    /// It will be called twice each time it is needed, once being passed
    /// `CaptureLatency` and once with `PlayBackLatency. See managing and
    /// determining latency for the definition of each type of latency and
    /// related functions. TODO: clear up the "see managing and ..." in the
    /// docstring.
    ///
    /// IMPORTANT: Most JACK clients do NOT need to register a latency callback.
    ///
    /// Clients that meed any of the following conditions do NOT need to
    /// register a latency callback:
    ///
    /// * have only input ports
    ///
    /// * have only output ports
    ///
    /// * their output is totally unrelated to their input
    ///
    /// * their output is not delayed relative to their input (i.e. data that
    /// arrives in a `process` is processed and output again in the same
    /// callback)
    ///
    /// Clients NOT registering a latency callback MUST also satisfy this condition
    ///
    /// * have no multiple distinct internal signal pathways
    ///
    /// This means that if your client has more than 1 input and output port,
    /// and considers them always "correlated" (e.g. as a stereo pair), then
    /// there is only 1 (e.g. stereo) signal pathway through the client. This
    /// would be true, for example, of a stereo FX rack client that has a
    /// left/right input pair and a left/right output pair.
    ///
    /// However, this is somewhat a matter of perspective. The same FX rack
    /// client could be connected so that its two input ports were connected to
    /// entirely separate sources. Under these conditions, the fact that the
    /// client does not register a latency callback MAY result in port latency
    /// values being incorrect.
    ///
    /// Clients that do not meet any of those conditions SHOULD register a
    /// latency callback.
    ///
    /// See the documentation for `jack_port_set_latency_range()` on how the
    /// callback should operate. Remember that the mode argument given to the
    /// latency callback will need to be passed into
    /// jack_port_set_latency_range()
    fn latency(&mut self, _mode: LatencyType) {}
}

impl<F: 'static + Send + FnMut(&ProcessScope) -> JackControl> JackHandler for F {
    fn process(&mut self, ps: &ProcessScope) -> JackControl {
        (self)(ps)
    }
}

unsafe fn handler_and_ptr_from_void<'a, T: JackHandler>(ptr: *mut c_void)
                                                        -> &'a mut (T, *mut j::jack_client_t) {
    assert!(!ptr.is_null());
    let obj_ptr: *mut (T, *mut j::jack_client_t) = mem::transmute(ptr);
    &mut *obj_ptr
}

unsafe extern "C" fn thread_init_callback<T: JackHandler>(data: *mut c_void) {
    let obj: &mut (T, _) = handler_and_ptr_from_void(data);
    obj.0.thread_init()
}

unsafe extern "C" fn shutdown<T: JackHandler>(code: j::jack_status_t,
                                              reason: *const i8,
                                              data: *mut c_void) {
    let obj: &mut (T, _) = handler_and_ptr_from_void(data);
    let cstr = ffi::CStr::from_ptr(reason);
    let reason_str = match cstr.to_str() {
        Ok(s) => s,
        Err(_) => "Failed to interpret error.",
    };
    obj.0.shutdown(ClientStatus::from_bits(code).unwrap_or(UNKNOWN_ERROR),
                   reason_str)
}

unsafe extern "C" fn process<T: JackHandler>(n_frames: u32, data: *mut c_void) -> i32 {
    let obj: &mut (T, *mut j::jack_client_t) = handler_and_ptr_from_void(data);
    let scope = ProcessScope::from_raw(n_frames, obj.1);
    obj.0.process(&scope).to_ffi()
}

unsafe extern "C" fn freewheel<T: JackHandler>(starting: i32, data: *mut c_void) {
    let obj: &mut (T, *mut j::jack_client_t) = handler_and_ptr_from_void(data);
    let is_starting = match starting {
        0 => false,
        _ => true,
    };
    obj.0.freewheel(is_starting)
}

unsafe extern "C" fn buffer_size<T: JackHandler>(n_frames: u32, data: *mut c_void) -> i32 {
    let obj: &mut (T, _) = handler_and_ptr_from_void(data);
    obj.0.buffer_size(n_frames).to_ffi()
}

unsafe extern "C" fn sample_rate<T: JackHandler>(n_frames: u32, data: *mut c_void) -> i32 {
    let obj: &mut (T, _) = handler_and_ptr_from_void(data);
    obj.0.sample_rate(n_frames).to_ffi()
}

unsafe extern "C" fn client_registration<T: JackHandler>(name: *const i8,
                                                         register: i32,
                                                         data: *mut c_void) {
    let obj: &mut (T, _) = handler_and_ptr_from_void(data);
    let name = ffi::CStr::from_ptr(name).to_str().unwrap();
    let register = match register {
        0 => false,
        _ => true,
    };
    obj.0.client_registration(name, register)
}


unsafe extern "C" fn port_registration<T: JackHandler>(port_id: u32,
                                                       register: i32,
                                                       data: *mut c_void) {
    let obj: &mut (T, _) = handler_and_ptr_from_void(data);
    let register = match register {
        0 => false,
        _ => true,
    };
    obj.0.port_registration(port_id, register)
}

#[allow(dead_code)] // TODO: remove once it can be registered
unsafe extern "C" fn port_rename<T: JackHandler>(port_id: u32,
                                                 old_name: *const i8,
                                                 new_name: *const i8,
                                                 data: *mut c_void)
                                                 -> i32 {
    let obj: &mut (T, _) = handler_and_ptr_from_void(data);
    let old_name = ffi::CStr::from_ptr(old_name).to_str().unwrap();
    let new_name = ffi::CStr::from_ptr(new_name).to_str().unwrap();
    obj.0.port_rename(port_id, old_name, new_name).to_ffi()
}

unsafe extern "C" fn port_connect<T: JackHandler>(port_id_a: u32,
                                                  port_id_b: u32,
                                                  connect: i32,
                                                  data: *mut c_void) {
    let obj: &mut (T, _) = handler_and_ptr_from_void(data);
    let are_connected = match connect {
        0 => false,
        _ => true,
    };
    obj.0.ports_connected(port_id_a, port_id_b, are_connected)
}

unsafe extern "C" fn graph_order<T: JackHandler>(data: *mut c_void) -> i32 {
    let obj: &mut (T, _) = handler_and_ptr_from_void(data);
    obj.0.graph_reorder().to_ffi()
}

unsafe extern "C" fn xrun<T: JackHandler>(data: *mut c_void) -> i32 {
    let obj: &mut (T, _) = handler_and_ptr_from_void(data);
    obj.0.xrun().to_ffi()
}

unsafe extern "C" fn latency<T: JackHandler>(mode: j::jack_latency_callback_mode_t,
                                             data: *mut c_void) {
    let obj: &mut (T, _) = handler_and_ptr_from_void(data);
    let mode = match mode {
        j::JackCaptureLatency => LatencyType::Capture,
        j::JackPlaybackLatency => LatencyType::Playback,
        _ => unreachable!(),
    };
    obj.0.latency(mode)
}

/// Clears the callbacks registered to `client`.
///
/// Returns `Err(JackErr::CallbackDeregistrationError)` on failure.
///
/// # Unsafe
/// * Uses ffi calls, be careful.
///
/// # TODO
/// * Implement correctly. Freezes on my system.
pub unsafe fn clear_callbacks(_client: *mut j::jack_client_t) -> Result<(), JackErr> {
    // j::jack_set_thread_init_callback(client, None, ptr::null_mut());
    // j::jack_set_process_callback(client, None, ptr::null_mut());
    Ok(())
}

/// Registers methods from `handler` to be used by JACK with `client`.
///
/// Returns `Ok(handler_ptr)` on success, or
/// `Err(JackErr::CallbackRegistrationError)` on failure.
///
/// `handler_ptr` here is a pointer to a heap-allocated pair `(T, *mut j::jack_client_t)`.
///
/// Registers `handler` with JACK. All JACK calls to `client` will be handled by
/// `handler`. `handler` is consumed, but it is not deallocated. `handler`
/// should be manually deallocated when JACK will no longer make calls to it,
/// such as when registering new callbacks with the same client, or dropping the
/// client.
///
/// # TODO
/// * Handled failed registrations
/// * Fix `jack_set_port_rename_callback`
///
/// # Unsafe
/// * `handler` will not be automatically deallocated.
pub unsafe fn register_callbacks<T: JackHandler>
    (handler: T,
     client: *mut j::jack_client_t,
     client_id: *mut j::jack_client_t)
     -> Result<*mut (T, *mut j::jack_client_t), JackErr> {
    let handler_ptr: *mut (T, *mut j::jack_client_t) = Box::into_raw(Box::new((handler,
                                                                               client_id)));
    let data_ptr = mem::transmute(handler_ptr);
    j::jack_set_thread_init_callback(client, Some(thread_init_callback::<T>), data_ptr);
    j::jack_on_info_shutdown(client, Some(shutdown::<T>), data_ptr);
    j::jack_set_process_callback(client, Some(process::<T>), data_ptr);
    j::jack_set_freewheel_callback(client, Some(freewheel::<T>), data_ptr);
    j::jack_set_buffer_size_callback(client, Some(buffer_size::<T>), data_ptr);
    j::jack_set_sample_rate_callback(client, Some(sample_rate::<T>), data_ptr);
    j::jack_set_client_registration_callback(client, Some(client_registration::<T>), data_ptr);
    j::jack_set_port_registration_callback(client, Some(port_registration::<T>), data_ptr);
    // doesn't compile for testing
    // j::jack_set_port_rename_callback(client, Some(port_rename::<T>), data_ptr);
    j::jack_set_port_connect_callback(client, Some(port_connect::<T>), data_ptr);
    j::jack_set_graph_order_callback(client, Some(graph_order::<T>), data_ptr);
    j::jack_set_xrun_callback(client, Some(xrun::<T>), data_ptr);
    j::jack_set_latency_callback(client, Some(latency::<T>), data_ptr);
    Ok(handler_ptr)
}
