use std::{ffi, mem};

use jack_sys as j;
use libc;

use client::ProcessScope;
use client::base::Client;
use client::client_status::ClientStatus;
use jack_enums::*;
use primitive_types as pt;

/// Specifies callbacks for JACK.
pub trait NotificationHandler: Send {
    /// Called just once after the creation of the thread in which all other callbacks will be
    /// handled.
    ///
    /// It does not need to be suitable for real-time execution.
    fn thread_init(&self, _: &Client) {}

    /// Called when the JACK server shuts down the client thread. The function must be written as if
    /// it were an asynchronous POSIX signal handler --- use only async-safe functions, and remember
    /// that it is executed from another thread. A typical funcion might set a flag or write to a
    /// pipe so that the rest of the application knows that the JACK client thread has shut down.
    fn shutdown(&mut self, _status: ClientStatus, _reason: &str) {}

    /// Called whenever "freewheel" mode is entered or leaving.
    fn freewheel(&mut self, _: &Client, _is_freewheel_enabled: bool) {}

    /// Called whenever the size of the buffer that will be passed to `process` is about to change.
    fn buffer_size(&mut self, _: &Client, _size: pt::JackFrames) -> JackControl {
        JackControl::Continue
    }

    /// Called whenever the system sample rate changes.
    fn sample_rate(&mut self, _: &Client, _srate: pt::JackFrames) -> JackControl {
        JackControl::Continue
    }

    /// Called whenever a client is registered or unregistered
    fn client_registration(&mut self, _: &Client, _name: &str, _is_registered: bool) {}

    /// Called whenever a port is registered or unregistered
    fn port_registration(&mut self, _: &Client, _port_id: pt::JackPortId, _is_registered: bool) {}

    /// Called whenever a port is renamed.
    fn port_rename(&mut self,
                   _: &Client,
                   _port_id: pt::JackPortId,
                   _old_name: &str,
                   _new_name: &str)
                   -> JackControl {
        JackControl::Continue
    }

    /// Called whenever ports are connected/disconnected to/from each other.
    fn ports_connected(&mut self,
                       _: &Client,
                       _port_id_a: pt::JackPortId,
                       _port_id_b: pt::JackPortId,
                       _are_connected: bool) {
    }

    /// Called whenever the processing graph is reordered.
    fn graph_reorder(&mut self, _: &Client) -> JackControl {
        JackControl::Continue
    }

    /// Called whenever an xrun occurs.
    ///
    /// An xrun is a buffer under or over run, which means some data has been missed.
    fn xrun(&mut self, _: &Client) -> JackControl {
        JackControl::Continue
    }

    /// Called whenever it is necessary to recompute the latencies for some or all JACK ports.
    ///
    /// It will be called twice each time it is needed, once being passed `CaptureLatency` and once
    /// with `PlayBackLatency. See managing and determining latency for the definition of each type
    /// of latency and related functions. TODO: clear up the "see managing and ..." in the
    /// docstring.
    ///
    /// IMPORTANT: Most JACK clients do NOT need to register a latency callback.
    ///
    /// Clients that meed any of the following conditions do NOT need to register a latency
    /// callback:
    ///
    /// * have only input ports
    ///
    /// * have only output ports
    ///
    /// * their output is totally unrelated to their input
    ///
    /// * their output is not delayed relative to their input (i.e. data that arrives in a `process`
    /// is processed and output again in the same callback)
    ///
    /// Clients NOT registering a latency callback MUST also satisfy this condition
    ///
    /// * have no multiple distinct internal signal pathways
    ///
    /// This means that if your client has more than 1 input and output port, and considers them
    /// always "correlated" (e.g. as a stereo pair), then there is only 1 (e.g. stereo) signal
    /// pathway through the client. This would be true, for example, of a stereo FX rack client that
    /// has a left/right input pair and a left/right output pair.
    ///
    /// However, this is somewhat a matter of perspective. The same FX rack client could be
    /// connected so that its two input ports were connected to entirely separate sources. Under
    /// these conditions, the fact that the client does not register a latency callback MAY result
    /// in port latency values being incorrect.
    ///
    /// Clients that do not meet any of those conditions SHOULD register a latency callback.
    ///
    /// See the documentation for `jack_port_set_latency_range()` on how the callback should
    /// operate. Remember that the mode argument given to the latency callback will need to be
    /// passed into jack_port_set_latency_range()
    fn latency(&mut self, _: &Client, _mode: LatencyType) {}
}

pub trait ProcessHandler : Send {
    /// Called whenever there is work to be done.
    ///
    /// It needs to be suitable for real-time execution. That means that it cannot call functions
    /// that might block for a long time. This includes all I/O functions (disk, TTY, network),
    /// malloc, free, printf, pthread_mutex_lock, sleep, wait, poll, select, pthread_join,
    /// pthread_cond_wait, etc, etc.
    ///
    /// Should return `JackControl::Continue` on success, and `JackControl::Quit` on error.
    fn process(&mut self, _: &Client, _process_scope: &ProcessScope) -> JackControl;
}

unsafe fn handler_and_ptr_from_void<'a, N, P>(ptr: *mut libc::c_void) -> &'a mut (N, P, Client)
    where N: NotificationHandler,
          P: ProcessHandler
{
    assert!(!ptr.is_null());
    let obj_ptr: *mut (N, P, Client) = mem::transmute(ptr);
    &mut *obj_ptr
}

unsafe extern "C" fn thread_init_callback<N, P>(data: *mut libc::c_void)
    where N: NotificationHandler,
          P: ProcessHandler
{
    let obj: &mut (N, P, Client) = handler_and_ptr_from_void(data);
    obj.0.thread_init(&obj.2)
}

unsafe extern "C" fn shutdown<N, P>(code: j::jack_status_t,
                                    reason: *const libc::c_char,
                                    data: *mut libc::c_void)
    where N: NotificationHandler,
          P: ProcessHandler
{
    let obj: &mut (N, P, _) = handler_and_ptr_from_void(data);
    let cstr = ffi::CStr::from_ptr(reason);
    let reason_str = match cstr.to_str() {
        Ok(s) => s,
        Err(_) => "Failed to interpret error.",
    };
    obj.0.shutdown(ClientStatus::from_bits(code).unwrap_or(ClientStatus::empty()),
                   reason_str)
}

unsafe extern "C" fn process<N, P>(n_frames: pt::JackFrames, data: *mut libc::c_void) -> libc::c_int
    where N: NotificationHandler,
          P: ProcessHandler
{
    let obj: &mut (N, P, Client) = handler_and_ptr_from_void(data);
    let scope = ProcessScope::from_raw(n_frames, obj.2.as_ptr());
    obj.1.process(&obj.2, &scope).to_ffi()
}

unsafe extern "C" fn freewheel<N, P>(starting: libc::c_int, data: *mut libc::c_void)
    where N: NotificationHandler,
          P: ProcessHandler
{
    let obj: &mut (N, P, Client) = handler_and_ptr_from_void(data);
    let is_starting = match starting {
        0 => false,
        _ => true,
    };
    obj.0.freewheel(&obj.2, is_starting)
}

unsafe extern "C" fn buffer_size<N, P>(n_frames: pt::JackFrames,
                                       data: *mut libc::c_void)
                                       -> libc::c_int
    where N: NotificationHandler,
          P: ProcessHandler
{
    let obj: &mut (N, P, Client) = handler_and_ptr_from_void(data);
    obj.0.buffer_size(&obj.2, n_frames).to_ffi()
}

unsafe extern "C" fn sample_rate<N, P>(n_frames: pt::JackFrames,
                                       data: *mut libc::c_void)
                                       -> libc::c_int
    where N: NotificationHandler,
          P: ProcessHandler
{
    let obj: &mut (N, P, Client) = handler_and_ptr_from_void(data);
    obj.0.sample_rate(&obj.2, n_frames).to_ffi()
}

unsafe extern "C" fn client_registration<N, P>(name: *const libc::c_char,
                                               register: libc::c_int,
                                               data: *mut libc::c_void)
    where N: NotificationHandler,
          P: ProcessHandler
{
    let obj: &mut (N, P, Client) = handler_and_ptr_from_void(data);
    let name = ffi::CStr::from_ptr(name).to_str().unwrap();
    let register = match register {
        0 => false,
        _ => true,
    };
    obj.0.client_registration(&obj.2, name, register)
}


unsafe extern "C" fn port_registration<N, P>(port_id: pt::JackPortId,
                                             register: libc::c_int,
                                             data: *mut libc::c_void)
    where N: NotificationHandler,
          P: ProcessHandler
{
    let obj: &mut (N, P, Client) = handler_and_ptr_from_void(data);
    let register = match register {
        0 => false,
        _ => true,
    };
    obj.0.port_registration(&obj.2, port_id, register)
}

#[allow(dead_code)] // TODO: remove once it can be registered
unsafe extern "C" fn port_rename<N, P>(port_id: pt::JackPortId,
                                       old_name: *const libc::c_char,
                                       new_name: *const libc::c_char,
                                       data: *mut libc::c_void)
                                       -> libc::c_int
    where N: NotificationHandler,
          P: ProcessHandler
{
    let obj: &mut (N, P, Client) = handler_and_ptr_from_void(data);
    let old_name = ffi::CStr::from_ptr(old_name).to_str().unwrap();
    let new_name = ffi::CStr::from_ptr(new_name).to_str().unwrap();
    obj.0.port_rename(&obj.2, port_id, old_name, new_name).to_ffi()
}

unsafe extern "C" fn port_connect<N, P>(port_id_a: pt::JackPortId,
                                        port_id_b: pt::JackPortId,
                                        connect: libc::c_int,
                                        data: *mut libc::c_void)
    where N: NotificationHandler,
          P: ProcessHandler
{
    let obj: &mut (N, P, Client) = handler_and_ptr_from_void(data);
    let are_connected = match connect {
        0 => false,
        _ => true,
    };
    obj.0.ports_connected(&obj.2, port_id_a, port_id_b, are_connected)
}

unsafe extern "C" fn graph_order<N, P>(data: *mut libc::c_void) -> libc::c_int
    where N: NotificationHandler,
          P: ProcessHandler
{
    let obj: &mut (N, P, Client) = handler_and_ptr_from_void(data);
    obj.0.graph_reorder(&obj.2).to_ffi()
}

unsafe extern "C" fn xrun<N, P>(data: *mut libc::c_void) -> libc::c_int
    where N: NotificationHandler,
          P: ProcessHandler
{
    let obj: &mut (N, P, Client) = handler_and_ptr_from_void(data);
    obj.0.xrun(&obj.2).to_ffi()
}

unsafe extern "C" fn latency<N, P>(mode: j::jack_latency_callback_mode_t, data: *mut libc::c_void)
    where N: NotificationHandler,
          P: ProcessHandler
{
    let obj: &mut (N, P, Client) = handler_and_ptr_from_void(data);
    let mode = match mode {
        j::JackCaptureLatency => LatencyType::Capture,
        j::JackPlaybackLatency => LatencyType::Playback,
        _ => unreachable!(),
    };
    obj.0.latency(&obj.2, mode)
}

/// Unsafe ffi wrapper that clears the callbacks registered to `client`.
///
/// This is mostly for use within the jack crate itself.
///
/// Returns `Err(JackErr::CallbackDeregistrationError)` on failure.
///
/// # Unsafe
///
/// * Uses ffi calls, be careful.
///
/// # TODO
///
/// * Implement correctly. Freezes on my system.
pub unsafe fn clear_callbacks(_client: *mut j::jack_client_t) -> Result<(), JackErr> {
    // j::jack_set_thread_init_callback(client, None, ptr::null_mut());
    // j::jack_set_process_callback(client, None, ptr::null_mut());
    Ok(())
}

/// Registers methods from `handler` to be used by JACK with `client`.
///
/// This is mostly for use within the jack crate itself.
///
/// Returns `Ok(handler_ptr)` on success, or `Err(JackErr::CallbackRegistrationError)` on failure.
///
/// `handler_ptr` here is a pointer to a heap-allocated pair `(T, *mut j::jack_client_t)`.
///
/// Registers `handler` with JACK. All JACK calls to `client` will be handled by
/// `handler`. `handler` is consumed, but it is not deallocated. `handler` should be manually
/// deallocated when JACK will no longer make calls to it, such as when registering new callbacks
/// with the same client, or dropping the client.
///
/// # TODO
///
/// * Handled failed registrations
/// * Fix `jack_set_port_rename_callback`
///
/// # Unsafe
///
/// * makes ffi calls
/// * `handler` will not be automatically deallocated.
pub unsafe fn register_callbacks<N, P>(notification_handler: N,
                                       process_handler: P,
                                       client: *mut j::jack_client_t)
                                       -> Result<*mut (N, P, *mut j::jack_client_t), JackErr>
    where N: NotificationHandler,
          P: ProcessHandler
{
    let handler_ptr: *mut (N, P, *mut j::jack_client_t) =
        Box::into_raw(Box::new((notification_handler, process_handler, client)));
    let data_ptr = mem::transmute(handler_ptr);
    j::jack_set_thread_init_callback(client, Some(thread_init_callback::<N, P>), data_ptr);
    j::jack_on_info_shutdown(client, Some(shutdown::<N, P>), data_ptr);
    j::jack_set_process_callback(client, Some(process::<N, P>), data_ptr);
    j::jack_set_freewheel_callback(client, Some(freewheel::<N, P>), data_ptr);
    j::jack_set_buffer_size_callback(client, Some(buffer_size::<N, P>), data_ptr);
    j::jack_set_sample_rate_callback(client, Some(sample_rate::<N, P>), data_ptr);
    j::jack_set_client_registration_callback(client, Some(client_registration::<N, P>), data_ptr);
    j::jack_set_port_registration_callback(client, Some(port_registration::<N, P>), data_ptr);
    // doesn't compile for testing since it is a weak export
    // j::jack_set_port_rename_callback(client, Some(port_rename::<N, P), data_ptr);
    j::jack_set_port_connect_callback(client, Some(port_connect::<N, P>), data_ptr);
    j::jack_set_graph_order_callback(client, Some(graph_order::<N, P>), data_ptr);
    j::jack_set_xrun_callback(client, Some(xrun::<N, P>), data_ptr);
    j::jack_set_latency_callback(client, Some(latency::<N, P>), data_ptr);
    Ok(handler_ptr)
}
