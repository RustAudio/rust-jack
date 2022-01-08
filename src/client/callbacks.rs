use jack_sys as j;
use std::ffi;

use crate::{Client, ClientStatus, Control, Error, Frames, PortId, ProcessScope};

/// Specifies callbacks for JACK.
pub trait NotificationHandler: Send {
    /// Called just once after the creation of the thread in which all other
    /// callbacks will be
    /// handled.
    ///
    /// It does not need to be suitable for real-time execution.
    fn thread_init(&self, _: &Client) {}

    /// Called when the JACK server shuts down the client thread. The function
    /// must be written as if
    /// it were an asynchronous POSIX signal handler --- use only async-safe
    /// functions, and remember
    /// that it is executed from another thread. A typical funcion might set a
    /// flag or write to a
    /// pipe so that the rest of the application knows that the JACK client
    /// thread has shut down.
    fn shutdown(&mut self, _status: ClientStatus, _reason: &str) {}

    /// Called whenever "freewheel" mode is entered or leaving.
    fn freewheel(&mut self, _: &Client, _is_freewheel_enabled: bool) {}

    /// Called whenever the system sample rate changes.
    fn sample_rate(&mut self, _: &Client, _srate: Frames) -> Control {
        Control::Continue
    }

    /// Called whenever a client is registered or unregistered
    fn client_registration(&mut self, _: &Client, _name: &str, _is_registered: bool) {}

    /// Called whenever a port is registered or unregistered
    fn port_registration(&mut self, _: &Client, _port_id: PortId, _is_registered: bool) {}

    /// Called whenever a port is renamed.
    fn port_rename(
        &mut self,
        _: &Client,
        _port_id: PortId,
        _old_name: &str,
        _new_name: &str,
    ) -> Control {
        Control::Continue
    }

    /// Called whenever ports are connected/disconnected to/from each other.
    fn ports_connected(
        &mut self,
        _: &Client,
        _port_id_a: PortId,
        _port_id_b: PortId,
        _are_connected: bool,
    ) {
    }

    /// Called whenever the processing graph is reordered.
    fn graph_reorder(&mut self, _: &Client) -> Control {
        Control::Continue
    }

    /// Called whenever an xrun occurs.
    ///
    /// An xrun is a buffer under or over run, which means some data has been
    /// missed.
    fn xrun(&mut self, _: &Client) -> Control {
        Control::Continue
    }
}

/// Specifies real-time processing.
pub trait ProcessHandler: Send {
    /// Called whenever there is work to be done.
    ///
    /// It needs to be suitable for real-time execution. That means that it
    /// cannot call functions
    /// that might block for a long time. This includes all I/O functions
    /// (disk, TTY, network),
    /// malloc, free, printf, pthread_mutex_lock, sleep, wait, poll, select,
    /// pthread_join,
    /// pthread_cond_wait, etc, etc.
    ///
    /// Should return `Control::Continue` on success, and
    /// `Control::Quit` on error.
    fn process(&mut self, _: &Client, _process_scope: &ProcessScope) -> Control;

    /// Called whenever the size of the buffer that will be passed to `process`
    /// is about to change, and once before the first call to `process`.
    ///
    /// It is called on the same thread as `process`, but as an exception, does
    /// not need to be suitable for real-time execution, so it is allowed to
    /// allocate new buffers to accomodate the buffer size for example.
    fn buffer_size(&mut self, _: &Client, _size: Frames) -> Control {
        Control::Continue
    }
}

unsafe extern "C" fn thread_init_callback<N, P>(data: *mut libc::c_void)
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let ctx = CallbackContext::<N, P>::from_raw(data);
    ctx.notification.thread_init(&ctx.client)
}

unsafe extern "C" fn shutdown<N, P>(
    code: j::jack_status_t,
    reason: *const libc::c_char,
    data: *mut libc::c_void,
) where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let ctx = CallbackContext::<N, P>::from_raw(data);
    let cstr = ffi::CStr::from_ptr(reason);
    let reason_str = cstr.to_str().unwrap_or("Failed to interpret error.");
    ctx.notification.shutdown(
        ClientStatus::from_bits(code).unwrap_or_else(ClientStatus::empty),
        reason_str,
    )
}

unsafe extern "C" fn process<N, P>(n_frames: Frames, data: *mut libc::c_void) -> libc::c_int
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let ctx = CallbackContext::<N, P>::from_raw(data);
    let scope = ProcessScope::from_raw(n_frames, ctx.client.raw());
    ctx.process.process(&ctx.client, &scope).to_ffi()
}

unsafe extern "C" fn freewheel<N, P>(starting: libc::c_int, data: *mut libc::c_void)
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let ctx = CallbackContext::<N, P>::from_raw(data);
    let is_starting = !matches!(starting, 0);
    ctx.notification.freewheel(&ctx.client, is_starting)
}

unsafe extern "C" fn buffer_size<N, P>(n_frames: Frames, data: *mut libc::c_void) -> libc::c_int
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let ctx = CallbackContext::<N, P>::from_raw(data);
    ctx.process.buffer_size(&ctx.client, n_frames).to_ffi()
}

unsafe extern "C" fn sample_rate<N, P>(n_frames: Frames, data: *mut libc::c_void) -> libc::c_int
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let ctx = CallbackContext::<N, P>::from_raw(data);
    ctx.notification.sample_rate(&ctx.client, n_frames).to_ffi()
}

unsafe extern "C" fn client_registration<N, P>(
    name: *const libc::c_char,
    register: libc::c_int,
    data: *mut libc::c_void,
) where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let ctx = CallbackContext::<N, P>::from_raw(data);
    let name = ffi::CStr::from_ptr(name).to_str().unwrap();
    let register = !matches!(register, 0);
    ctx.notification
        .client_registration(&ctx.client, name, register)
}

unsafe extern "C" fn port_registration<N, P>(
    port_id: PortId,
    register: libc::c_int,
    data: *mut libc::c_void,
) where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let ctx = CallbackContext::<N, P>::from_raw(data);
    let register = !matches!(register, 0);
    ctx.notification
        .port_registration(&ctx.client, port_id, register)
}

#[allow(dead_code)] // TODO: remove once it can be registered
unsafe extern "C" fn port_rename<N, P>(
    port_id: PortId,
    old_name: *const libc::c_char,
    new_name: *const libc::c_char,
    data: *mut libc::c_void,
) -> libc::c_int
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let ctx = CallbackContext::<N, P>::from_raw(data);
    let old_name = ffi::CStr::from_ptr(old_name).to_str().unwrap();
    let new_name = ffi::CStr::from_ptr(new_name).to_str().unwrap();
    ctx.notification
        .port_rename(&ctx.client, port_id, old_name, new_name)
        .to_ffi()
}

unsafe extern "C" fn port_connect<N, P>(
    port_id_a: PortId,
    port_id_b: PortId,
    connect: libc::c_int,
    data: *mut libc::c_void,
) where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let ctx = CallbackContext::<N, P>::from_raw(data);
    let are_connected = !matches!(connect, 0);
    ctx.notification
        .ports_connected(&ctx.client, port_id_a, port_id_b, are_connected)
}

unsafe extern "C" fn graph_order<N, P>(data: *mut libc::c_void) -> libc::c_int
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let ctx = CallbackContext::<N, P>::from_raw(data);
    ctx.notification.graph_reorder(&ctx.client).to_ffi()
}

unsafe extern "C" fn xrun<N, P>(data: *mut libc::c_void) -> libc::c_int
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let ctx = CallbackContext::<N, P>::from_raw(data);
    ctx.notification.xrun(&ctx.client).to_ffi()
}

/// Unsafe ffi wrapper that clears the callbacks registered to `client`.
///
/// This is mostly for use within the jack crate itself.
///
/// Returns `Err(Error::CallbackDeregistrationError)` on failure.
///
/// # Unsafe
///
/// * Uses ffi calls, be careful.
///
/// # TODO
///
/// * Implement correctly. Freezes on my system.
pub unsafe fn clear_callbacks(_client: *mut j::jack_client_t) -> Result<(), Error> {
    // j::jack_set_thread_init_callback(client, None, ptr::null_mut());
    // j::jack_set_process_callback(client, None, ptr::null_mut());
    Ok(())
}

pub struct CallbackContext<N, P> {
    pub client: Client,
    pub notification: N,
    pub process: P,
}

impl<N, P> CallbackContext<N, P>
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    pub unsafe fn from_raw<'a>(ptr: *mut libc::c_void) -> &'a mut CallbackContext<N, P> {
        debug_assert!(!ptr.is_null());
        let obj_ptr = ptr as *mut CallbackContext<N, P>;
        &mut *obj_ptr
    }

    fn raw(b: &mut Box<Self>) -> *mut libc::c_void {
        let ptr: *mut Self = b.as_mut();
        ptr as *mut libc::c_void
    }

    /// Registers methods from `handler` to be used by JACK with `client`.
    ///
    /// This is mostly for use within the jack crate itself.
    ///
    /// Returns `Ok(handler_ptr)` on success, or
    /// `Err(Error::CallbackRegistrationError)` on failure.
    ///
    /// `handler_ptr` here is a pointer to a heap-allocated pair `(T, *mut
    /// j::jack_client_t)`.
    ///
    /// Registers `handler` with JACK. All JACK calls to `client` will be handled by
    /// `handler`. `handler` is consumed, but it is not deallocated. `handler`
    /// should be manually
    /// deallocated when JACK will no longer make calls to it, such as when
    /// registering new callbacks
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
    pub unsafe fn register_callbacks(b: &mut Box<Self>) -> Result<(), Error> {
        let data_ptr = CallbackContext::raw(b);
        let client = b.client.raw();
        j::jack_set_thread_init_callback(client, Some(thread_init_callback::<N, P>), data_ptr);
        j::jack_on_info_shutdown(client, Some(shutdown::<N, P>), data_ptr);
        j::jack_set_process_callback(client, Some(process::<N, P>), data_ptr);
        j::jack_set_freewheel_callback(client, Some(freewheel::<N, P>), data_ptr);
        j::jack_set_buffer_size_callback(client, Some(buffer_size::<N, P>), data_ptr);
        j::jack_set_sample_rate_callback(client, Some(sample_rate::<N, P>), data_ptr);
        j::jack_set_client_registration_callback(
            client,
            Some(client_registration::<N, P>),
            data_ptr,
        );
        j::jack_set_port_registration_callback(client, Some(port_registration::<N, P>), data_ptr);
        // doesn't compile for testing since it is a weak export
        // j::jack_set_port_rename_callback(client, Some(port_rename::<N, P), data_ptr);
        j::jack_set_port_connect_callback(client, Some(port_connect::<N, P>), data_ptr);
        j::jack_set_graph_order_callback(client, Some(graph_order::<N, P>), data_ptr);
        j::jack_set_xrun_callback(client, Some(xrun::<N, P>), data_ptr);
        Ok(())
    }
}
