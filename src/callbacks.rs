use std::{ffi, mem};
use libc::c_void;
use jack_sys as j;
use flags::*;

/// Specifies callbacks for Jack.
///
/// All callbacks happen on the same thread (not concurrently), unless otherwise
/// stated.
///
/// # TODO
/// * convert C enum return values to rust enums.
pub trait JackHandler {
    /// Called just once after the creation of the thread in which all other
    /// callbacks will be handled.
    ///
    /// It does not need to be suitable for real-time execution.
    fn thread_init(&mut self) {}

    /// Called when the Jack server shuts down the client thread. The function
    /// must be written as if it were an asynchronous POSIX signal handler ---
    /// use only async-safe functions, and remember that it is executed from
    /// another thread. A typical funcion might set a flag or write to a pipe so
    /// that the rest of the application knows that the Jack client thread has
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
    fn process(&mut self, _n_frames: u32) -> i32 {
        0
    }

    /// Called whenever "freewheel" mode is entered or leaving.
    fn freewheel(&mut self, _is_freewheel_enabled: bool) {}

    /// Called whenever the size of the buffer that will be passed to `process`
    /// is about to change.
    fn buffer_size(&mut self, _size: u32) -> i32 {
        0
    }

    /// Called whenever the system sample rate changes.
    fn sample_rate(&mut self, _srate: u32) -> i32 {
        0
    }

    /// Called whenever a client is registered or unregistered
    fn client_registration(&mut self, _name: &str, _is_registered: bool) {}

    /// Called whenever a port is registered or unregistered
    fn port_registration(&mut self, _port_id: u32, _is_registered: bool) {}

    /// Called whenever a port is renamed.
    ///
    /// # TODO
    /// * Possibly fix description, Jack API docs have same description
    /// for this as port registration.
    fn port_rename(&mut self, _port_id: u32, _old_name: &str, _new_name: &str) -> i32 {
        0
    }

    /// Called whenever ports are connected/disconnected to/from each other.
    fn ports_connected(&mut self, _port_id_a: u32, _port_id_b: u32, _are_connected: bool) {}

    /// Called whenever the processing graph is reordered.
    fn graph_reorder(&mut self) -> i32 { 0 }

    /// Called whenever an xrun occurs.
    ///
    /// An xrun is a buffer under or over run, which means some data has been
    /// missed.
    fn xrun(&mut self) -> i32 { 0 }

    /// Called whenever it is necessary to recompute the latencies for some or
    /// all Jack ports.
    ///
    /// It will be called twice each time it is needed, once being passed
    /// `CaptureLatency` and once with `PlayBackLatency. See managing and
    /// determining latency for the definition of each type of latency and
    /// related functions. TODO: clear up the "see managing and ..." in the
    /// docstring.
    ///
    /// IMPORTANT: Most Jack clients do NOT need to register a latency callback.
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
    fn latency(&mut self, _mode: LatencyType) { }
}

unsafe fn from_void<'a, T: JackHandler>(ptr: *mut c_void) -> &'a mut T {
    assert!(!ptr.is_null());
    let obj_ptr: *mut T = mem::transmute(ptr);
    &mut *obj_ptr
}

extern "C" fn thread_init_callback<T: JackHandler>(data: *mut c_void) {
    let obj: &mut T = unsafe { from_void(data) };
    obj.thread_init()
}

extern "C" fn shutdown<T: JackHandler>(code: j::jack_status_t,
                                       reason: *const i8,
                                       data: *mut c_void) {
    let obj: &mut T = unsafe { from_void(data) };
    let reason_str = unsafe {
        let cstr = ffi::CStr::from_ptr(reason);
        match cstr.to_str() {
            Ok(s) => s,
            Err(_) => "Failed to interpret error.",
        }
    };
    obj.shutdown(ClientStatus::from_bits(code).unwrap_or(UNKNOWN_ERROR),
                 reason_str)
}

extern "C" fn process<T: JackHandler>(n_frames: u32, data: *mut c_void) -> i32 {
    let obj: &mut T = unsafe { from_void(data) };
    obj.process(n_frames)
}

extern "C" fn freewheel<T: JackHandler>(starting: i32, data: *mut c_void) {
    let obj: &mut T = unsafe { from_void(data) };
    let is_starting = match starting {
        0 => false,
        _ => true,
    };
    obj.freewheel(is_starting)
}

extern "C" fn buffer_size<T: JackHandler>(n_frames: u32, data: *mut c_void) -> i32 {
    let obj: &mut T = unsafe { from_void(data) };
    obj.buffer_size(n_frames)
}

extern "C" fn sample_rate<T: JackHandler>(n_frames: u32, data: *mut c_void) -> i32 {
    let obj: &mut T = unsafe { from_void(data) };
    obj.sample_rate(n_frames)
}

extern "C" fn client_registration<T: JackHandler>(name: *const i8,
                                                  register: i32,
                                                  data: *mut c_void) {
    let obj: &mut T = unsafe { from_void(data) };
    let name = unsafe { ffi::CStr::from_ptr(name).to_str().unwrap() };
    let register = match register {
        0 => false,
        _ => true,
    };
    obj.client_registration(name, register)
}

extern "C" fn port_registration<T: JackHandler>(port_id: u32, register: i32, data: *mut c_void) {
    let obj: &mut T = unsafe { from_void(data) };
    let register = match register {
        0 => false,
        _ => true,
    };
    obj.port_registration(port_id, register)
}

#[allow(dead_code)] // TODO: remove once it can be registered
extern "C" fn port_rename<T: JackHandler>(port_id: u32,
                                          old_name: *const i8,
                                          new_name: *const i8,
                                          data: *mut c_void)
                                          -> i32 {
    let obj: &mut T = unsafe { from_void(data) };
    let old_name = unsafe { ffi::CStr::from_ptr(old_name).to_str().unwrap() };
    let new_name = unsafe { ffi::CStr::from_ptr(new_name).to_str().unwrap() };
    obj.port_rename(port_id, old_name, new_name)
}

extern "C" fn port_connect<T: JackHandler>(port_id_a: u32,
                                           port_id_b: u32,
                                           connect: i32,
                                           data: *mut c_void) {
    let obj: &mut T = unsafe { from_void(data) };
    let are_connected = match connect {
        0 => false,
        _ => true,
    };
    obj.ports_connected(port_id_a, port_id_b, are_connected)
}

extern "C" fn graph_order<T: JackHandler>(data: *mut c_void) -> i32 {
    let obj: &mut T = unsafe { from_void(data) };
    obj.graph_reorder()
}

extern "C" fn xrun<T: JackHandler>(data: *mut c_void) -> i32 {
    let obj: &mut T = unsafe { from_void(data) };
    obj.xrun()
}

extern "C" fn latency<T: JackHandler>(mode: j::jack_latency_callback_mode_t,
                                      data: *mut c_void) {
    let obj: &mut T = unsafe { from_void(data) };
    let mode = match mode {
        j::JackCaptureLatency => LatencyType::Capture,
        j::JackPlaybackLatency => LatencyType::Playback,
        _ => unreachable!(),
    };
    obj.latency(mode)
}

/// Clears the callbacks registered to `client`.
/// # Unsafe
/// * Uses ffi calls, be careful.
///
/// # TODO
/// * Implement correctly. Freezes on my system.
pub unsafe fn clear_callbacks(_client: *mut j::jack_client_t) {
    // j::jack_set_thread_init_callback(client, None, ptr::null_mut());
    // j::jack_set_process_callback(client, None, ptr::null_mut());
}

/// Registers methods from `handler` to be used by Jack with `client`.
///
/// Registers `handler` with jack. All jack calls to `client` will be handled by
/// `handler`. `handler` is consumed, but it is not deallocated. `handler`
/// should be manually deallocated when jack will no longer make calls to it,
/// such as when registering new callbacks with the same client, or dropping the
/// client.
///
/// # TODO
/// * Handled failed registrations
/// * Fix `jack_set_port_rename_callback`
///
/// # Unsafe
/// * `handler` will not be automatically deallocated.
pub unsafe fn register_callbacks<T: JackHandler>(client: *mut j::jack_client_t,
                                                 handler: T)
                                                 -> Result<*mut T, ()> {
    let handler_ptr: *mut T = Box::into_raw(Box::new(handler));
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
