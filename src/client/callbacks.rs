use jack_sys as j;
use std::{
    cell::UnsafeCell,
    ffi,
    panic::catch_unwind,
    sync::atomic::{AtomicBool, Ordering},
    sync::{Mutex, MutexGuard},
};

use crate::{Client, ClientStatus, Control, Error, Frames, PortId, ProcessScope};

/// Specifies callbacks for JACK.
pub trait NotificationHandler: Send {
    /// Called just once after the creation of the thread in which all other
    /// callbacks will be
    /// handled.
    ///
    /// It does not need to be suitable for real-time execution.
    ///
    /// Unlike the other methods, this takes `&self`: it runs on JACK's process thread, possibly at
    /// the same time as other notifications on the notification thread. Use interior mutability
    /// (for example an atomic) for any state it updates.
    fn thread_init(&self, _: &Client) {}

    /// Called when the JACK server shuts down the client thread. The function must be written as if
    /// it were an asynchronous POSIX signal handler --- use only async-safe functions, and remember
    /// that it is executed from another thread. A typical function might set a flag or write to a
    /// pipe so that the rest of the application knows that the JACK client thread has shut down.
    ///
    /// See <https://man7.org/linux/man-pages/man7/signal-safety.7.html> for details about
    /// what is legal in an async-signal-safe callback. This is a requirement on the
    /// implementation, which is why the method itself is safe to call.
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
    /// Indicates whether or not this process handler represents a
    /// slow-sync client
    const SLOW_SYNC: bool = false;

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

    /// For slow-sync clients, called periodically when the transport position
    /// is changed.  The transport will not start rolling until all clients
    /// indicate they are ready, or a timeout occurs.
    ///
    /// It should return `false` until the handler is ready process audio.
    ///
    /// Ignored unless Self::SLOW_SYNC == true.
    fn sync(
        &mut self,
        _: &Client,
        _state: crate::TransportState,
        _pos: &crate::TransportPosition,
    ) -> bool {
        true
    }
}

unsafe extern "C" fn thread_init_callback<N, P>(data: *mut libc::c_void)
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let _defer_cancel = crate::jack_utils::DeferThreadCancellation::new();
    let res = catch_unwind(|| {
        let Some(ctx) = CallbackContext::<N, P>::from_raw(data) else {
            return;
        };
        ctx.notification().thread_init(&ctx.client);
    });
    if let Err(err) = res {
        if let Some(ctx) = CallbackContext::<N, P>::from_raw(data) {
            ctx.mark_invalid(true)
        }
        eprintln!("{err:?}");
        std::mem::forget(err);
    }
}

unsafe extern "C" fn shutdown<N, P>(
    code: j::jack_status_t,
    reason: *const libc::c_char,
    data: *mut libc::c_void,
) where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let _defer_cancel = crate::jack_utils::DeferThreadCancellation::new();
    let res = catch_unwind(|| {
        let Some(ctx) = CallbackContext::<N, P>::from_raw(data) else {
            return;
        };
        let reason = if reason.is_null() {
            std::borrow::Cow::Borrowed("")
        } else {
            ffi::CStr::from_ptr(reason).to_string_lossy()
        };
        ctx.notification().shutdown(
            ClientStatus::from_bits(code).unwrap_or_else(ClientStatus::empty),
            &reason,
        );
    });
    if let Err(err) = res {
        if let Some(ctx) = CallbackContext::<N, P>::from_raw(data) {
            ctx.mark_invalid(true)
        }
        eprintln!("{err:?}");
        std::mem::forget(err);
    }
}

unsafe extern "C" fn process<N, P>(n_frames: Frames, data: *mut libc::c_void) -> libc::c_int
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let _defer_cancel = crate::jack_utils::DeferThreadCancellation::new();
    let Some(ctx_ref) = (unsafe { (data as *const CallbackContext<N, P>).as_ref() }) else {
        return Control::Quit.to_ffi();
    };
    let Ok(_guard) = ctx_ref.callback_lock.try_lock() else {
        return Control::Quit.to_ffi();
    };
    let res = catch_unwind(|| {
        let Some(ctx) = CallbackContext::<N, P>::from_raw(data) else {
            return Control::Quit;
        };
        let scope = ProcessScope::from_raw(n_frames, ctx.client.raw());
        let c = ctx.process().process(&ctx.client, &scope);
        if c == Control::Quit {
            ctx.mark_invalid(false);
        }
        c
    });
    match res {
        Ok(res) => res.to_ffi(),
        Err(err) => {
            if let Some(ctx) = CallbackContext::<N, P>::from_raw(data) {
                ctx.mark_invalid(true)
            }
            eprintln!("JACK process callback panicked.\n{err:?}");
            std::mem::forget(err);
            Control::Quit.to_ffi()
        }
    }
}

unsafe extern "C" fn sync<N, P>(
    state: jack_sys::jack_transport_state_t,
    pos: *mut jack_sys::jack_position_t,
    data: *mut libc::c_void,
) -> libc::c_int
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let _defer_cancel = crate::jack_utils::DeferThreadCancellation::new();
    let res = catch_unwind(|| {
        let Some(ctx) = CallbackContext::<N, P>::from_raw(data) else {
            return false;
        };
        let is_ready = ctx.process().sync(
            &ctx.client,
            crate::Transport::state_from_ffi(state),
            &*(pos as *mut crate::TransportPosition),
        );
        is_ready
    });
    match res {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(err) => {
            if let Some(ctx) = CallbackContext::<N, P>::from_raw(data) {
                ctx.mark_invalid(true)
            }
            eprintln!("{err:?}");
            std::mem::forget(err);
            0
        }
    }
}

unsafe extern "C" fn freewheel<N, P>(starting: libc::c_int, data: *mut libc::c_void)
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let _defer_cancel = crate::jack_utils::DeferThreadCancellation::new();
    let res = catch_unwind(|| {
        let Some(ctx) = CallbackContext::<N, P>::from_raw(data) else {
            return;
        };
        let is_starting = !matches!(starting, 0);
        ctx.notification().freewheel(&ctx.client, is_starting);
    });
    if let Err(err) = res {
        if let Some(ctx) = CallbackContext::<N, P>::from_raw(data) {
            ctx.mark_invalid(true)
        }
        eprintln!("{err:?}");
        std::mem::forget(err);
    }
}

unsafe extern "C" fn buffer_size<N, P>(n_frames: Frames, data: *mut libc::c_void) -> libc::c_int
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let _defer_cancel = crate::jack_utils::DeferThreadCancellation::new();
    let Some(ctx_ref) = (unsafe { (data as *const CallbackContext<N, P>).as_ref() }) else {
        return Control::Quit.to_ffi();
    };
    let Ok(_guard) = ctx_ref.callback_lock.try_lock() else {
        return Control::Quit.to_ffi();
    };
    let res = catch_unwind(|| {
        let Some(ctx) = CallbackContext::<N, P>::from_raw(data) else {
            return Control::Quit;
        };
        let c = ctx.process().buffer_size(&ctx.client, n_frames);
        if c == Control::Quit {
            ctx.mark_invalid(false);
        }
        c
    });
    match res {
        Ok(c) => c.to_ffi(),
        Err(err) => {
            if let Some(ctx) = CallbackContext::<N, P>::from_raw(data) {
                ctx.mark_invalid(true)
            }
            eprintln!("{err:?}");
            std::mem::forget(err);
            Control::Quit.to_ffi()
        }
    }
}

unsafe extern "C" fn sample_rate<N, P>(n_frames: Frames, data: *mut libc::c_void) -> libc::c_int
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let _defer_cancel = crate::jack_utils::DeferThreadCancellation::new();
    let res = catch_unwind(|| {
        let Some(ctx) = CallbackContext::<N, P>::from_raw(data) else {
            return Control::Quit;
        };
        let c = ctx.notification().sample_rate(&ctx.client, n_frames);
        if c == Control::Quit {
            ctx.mark_invalid(false);
        }
        c
    });
    match res {
        Ok(c) => c.to_ffi(),
        Err(err) => {
            if let Some(ctx) = CallbackContext::<N, P>::from_raw(data) {
                ctx.mark_invalid(true)
            }
            eprintln!("{err:?}");
            std::mem::forget(err);
            Control::Quit.to_ffi()
        }
    }
}

unsafe extern "C" fn client_registration<N, P>(
    name: *const libc::c_char,
    register: libc::c_int,
    data: *mut libc::c_void,
) where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let _defer_cancel = crate::jack_utils::DeferThreadCancellation::new();
    let res = catch_unwind(|| {
        let Some(ctx) = CallbackContext::<N, P>::from_raw(data) else {
            return;
        };
        let name = ffi::CStr::from_ptr(name).to_string_lossy();
        let register = !matches!(register, 0);
        ctx.notification()
            .client_registration(&ctx.client, &name, register);
    });
    if let Err(err) = res {
        if let Some(ctx) = CallbackContext::<N, P>::from_raw(data) {
            ctx.mark_invalid(true)
        }
        eprintln!("{err:?}");
        std::mem::forget(err);
    }
}

unsafe extern "C" fn port_registration<N, P>(
    port_id: PortId,
    register: libc::c_int,
    data: *mut libc::c_void,
) where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let _defer_cancel = crate::jack_utils::DeferThreadCancellation::new();
    let res = catch_unwind(|| {
        let Some(ctx) = CallbackContext::<N, P>::from_raw(data) else {
            return;
        };
        let register = !matches!(register, 0);
        ctx.notification()
            .port_registration(&ctx.client, port_id, register);
    });
    if let Err(err) = res {
        if let Some(ctx) = CallbackContext::<N, P>::from_raw(data) {
            ctx.mark_invalid(true)
        }
        eprintln!("{err:?}");
        std::mem::forget(err);
    }
}

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
    let _defer_cancel = crate::jack_utils::DeferThreadCancellation::new();
    let res = catch_unwind(|| {
        let Some(ctx) = CallbackContext::<N, P>::from_raw(data) else {
            return Control::Quit;
        };
        let old_name = ffi::CStr::from_ptr(old_name).to_string_lossy();
        let new_name = ffi::CStr::from_ptr(new_name).to_string_lossy();
        let c = ctx
            .notification()
            .port_rename(&ctx.client, port_id, &old_name, &new_name);
        if c == Control::Quit {
            ctx.mark_invalid(false);
        }
        c
    });
    match res {
        Ok(c) => c.to_ffi(),
        Err(err) => {
            if let Some(ctx) = CallbackContext::<N, P>::from_raw(data) {
                ctx.mark_invalid(true)
            }
            eprintln!("{err:?}");
            std::mem::forget(err);
            Control::Quit.to_ffi()
        }
    }
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
    let _defer_cancel = crate::jack_utils::DeferThreadCancellation::new();
    let res = catch_unwind(|| {
        let Some(ctx) = CallbackContext::<N, P>::from_raw(data) else {
            return;
        };
        let are_connected = !matches!(connect, 0);
        ctx.notification()
            .ports_connected(&ctx.client, port_id_a, port_id_b, are_connected);
    });
    if let Err(err) = res {
        if let Some(ctx) = CallbackContext::<N, P>::from_raw(data) {
            ctx.mark_invalid(true)
        }
        eprintln!("{err:?}");
        std::mem::forget(err);
    }
}

unsafe extern "C" fn graph_order<N, P>(data: *mut libc::c_void) -> libc::c_int
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let _defer_cancel = crate::jack_utils::DeferThreadCancellation::new();
    let res = catch_unwind(|| {
        let Some(ctx) = CallbackContext::<N, P>::from_raw(data) else {
            return Control::Quit;
        };
        let c = ctx.notification().graph_reorder(&ctx.client);
        if c == Control::Quit {
            ctx.mark_invalid(false);
        }
        c
    });
    match res {
        Ok(c) => c.to_ffi(),
        Err(err) => {
            if let Some(ctx) = CallbackContext::<N, P>::from_raw(data) {
                ctx.mark_invalid(true)
            }
            eprintln!("{err:?}");
            std::mem::forget(err);
            Control::Quit.to_ffi()
        }
    }
}

unsafe extern "C" fn xrun<N, P>(data: *mut libc::c_void) -> libc::c_int
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let _defer_cancel = crate::jack_utils::DeferThreadCancellation::new();
    let res = catch_unwind(|| {
        let Some(ctx) = CallbackContext::<N, P>::from_raw(data) else {
            return Control::Quit;
        };
        let c = ctx.notification().xrun(&ctx.client);
        if c == Control::Quit {
            ctx.mark_invalid(false);
        }
        c
    });
    match res {
        Ok(c) => c.to_ffi(),
        Err(err) => {
            if let Some(ctx) = CallbackContext::<N, P>::from_raw(data) {
                ctx.mark_invalid(true)
            }
            eprintln!("{err:?}");
            std::mem::forget(err);
            Control::Quit.to_ffi()
        }
    }
}

/// Unsafe ffi wrapper that clears the callbacks registered to `client`.
///
/// This is mostly for use within the jack crate itself.
///
/// Every callback that `CallbackContext::register_callbacks` sets must be cleared here. Otherwise
/// JACK keeps a pointer to the `CallbackContext` after it has been freed and may call into it, for
/// example when the server shuts down while the deactivated `Client` is still open.
///
/// # Safety
///
/// * `client` must be a valid, deactivated client.
pub unsafe fn clear_callbacks(client: *mut j::jack_client_t) -> Result<(), Error> {
    let null = std::ptr::null_mut();
    j::jack_set_thread_init_callback(client, None, null);
    j::jack_on_info_shutdown(client, None, null);
    j::jack_set_process_callback(client, None, null);
    j::jack_set_sync_callback(client, None, null);
    j::jack_set_freewheel_callback(client, None, null);
    j::jack_set_buffer_size_callback(client, None, null);
    j::jack_set_sample_rate_callback(client, None, null);
    j::jack_set_client_registration_callback(client, None, null);
    j::jack_set_port_registration_callback(client, None, null);
    j::jack_set_port_connect_callback(client, None, null);
    j::jack_set_graph_order_callback(client, None, null);
    j::jack_set_xrun_callback(client, None, null);
    Ok(())
}

/// The information used by JACK to process data.
///
/// JACK calls into this context from several threads at once: the process thread (`process`,
/// `sync` and `thread_init`) and the notification thread (all other callbacks). Callbacks therefore
/// only ever get a shared `&CallbackContext`. The handlers are reached through interior mutability:
///
/// * `notification` is behind a `Mutex`. Notifications are not real-time, and `thread_init` runs
///   on the process thread while the notification thread may be using the handler.
/// * `process` is behind an `UnsafeCell`. JACK calls `process` and `sync` on the process thread,
///   and calls `buffer_size` only while the process cycle is stopped, so the accesses never
///   overlap.
pub struct CallbackContext<N, P> {
    /// The underlying JACK client.
    pub client: Client,
    /// The handler for notifications.
    notification: Mutex<N>,
    /// The handler for processing.
    process: UnsafeCell<P>,
    /// True if the callback is valid for callbacks.
    ///
    /// This becomes false after quit an event that causes processing to quit.
    pub is_valid_for_callback: AtomicBool,
    /// True if the callback has panicked.
    pub has_panic: AtomicBool,
    /// Guard held while a callback is actively executing, so `deactivate` waits for any
    /// in-flight callback (and any panic unwind catching) to finish before calling
    /// `jack_deactivate`.
    pub callback_lock: std::sync::Mutex<()>,
}

impl<N, P> CallbackContext<N, P> {
    /// Create a new context that is valid for callbacks.
    pub fn new(client: Client, notification: N, process: P) -> Self {
        CallbackContext {
            client,
            notification: Mutex::new(notification),
            process: UnsafeCell::new(process),
            is_valid_for_callback: AtomicBool::new(true),
            has_panic: AtomicBool::new(false),
            callback_lock: std::sync::Mutex::new(()),
        }
    }

    /// Consume the context and return its parts.
    pub fn into_parts(self) -> (Client, N, P) {
        let notification = self
            .notification
            .into_inner()
            .unwrap_or_else(|err| err.into_inner());
        (self.client, notification, self.process.into_inner())
    }
}

impl<N, P> CallbackContext<N, P>
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    pub unsafe fn from_raw<'a>(ptr: *mut libc::c_void) -> Option<&'a CallbackContext<N, P>> {
        debug_assert!(!ptr.is_null());
        let obj_ptr = ptr as *const CallbackContext<N, P>;
        let obj_ref = &*obj_ptr;
        if obj_ref.is_valid_for_callback.load(Ordering::Relaxed) {
            Some(obj_ref)
        } else {
            None
        }
    }

    /// Lock the notification handler.
    ///
    /// A poisoned lock is still handed out. A panic in a handler already marks the context as
    /// invalid, so no further callbacks will reach it.
    fn notification(&self) -> MutexGuard<'_, N> {
        self.notification
            .lock()
            .unwrap_or_else(|err| err.into_inner())
    }

    /// Get the process handler.
    ///
    /// # Safety
    /// Must only be called from JACK's process thread, or from the buffer size callback while the
    /// process cycle is stopped. The returned reference must not outlive the callback.
    #[allow(clippy::mut_from_ref)]
    unsafe fn process(&self) -> &mut P {
        &mut *self.process.get()
    }

    /// Mark the callback context as invalid.
    ///
    /// This usually happens after a panic or when a callback returns `Control::Quit`.
    ///
    /// TODO: `pipewire-jack` ignores non-zero return values (`Control::Quit`) from callbacks such
    /// as `buffer_size` and continues invoking `process`, so we rely on `is_valid_for_callback` to
    /// prevent invoking user callbacks after quitting or panicking.
    #[cold]
    pub fn mark_invalid(&self, did_panic: bool) {
        self.is_valid_for_callback.store(false, Ordering::Relaxed);
        if did_panic {
            self.has_panic.store(true, Ordering::Relaxed);
        }
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
    /// If any registration fails, the callbacks are cleared again and
    /// `Err(Error::CallbackRegistrationError)` is returned. JACK refuses registrations while the
    /// client is active, for example.
    ///
    /// # TODO
    ///
    ///
    /// # Unsafe
    ///
    /// * makes ffi calls
    /// * `handler` will not be automatically deallocated.
    pub unsafe fn register_callbacks(b: &mut Box<Self>) -> Result<(), Error> {
        let data_ptr = CallbackContext::raw(b);
        let client = b.client.raw();
        j::jack_on_info_shutdown(client, Some(shutdown::<N, P>), data_ptr);
        let results = [
            j::jack_set_thread_init_callback(client, Some(thread_init_callback::<N, P>), data_ptr),
            j::jack_set_process_callback(client, Some(process::<N, P>), data_ptr),
            if P::SLOW_SYNC {
                j::jack_set_sync_callback(client, Some(sync::<N, P>), data_ptr)
            } else {
                0
            },
            j::jack_set_freewheel_callback(client, Some(freewheel::<N, P>), data_ptr),
            j::jack_set_buffer_size_callback(client, Some(buffer_size::<N, P>), data_ptr),
            j::jack_set_sample_rate_callback(client, Some(sample_rate::<N, P>), data_ptr),
            j::jack_set_client_registration_callback(
                client,
                Some(client_registration::<N, P>),
                data_ptr,
            ),
            j::jack_set_port_registration_callback(
                client,
                Some(port_registration::<N, P>),
                data_ptr,
            ),
            j::jack_set_port_rename_callback(client, Some(port_rename::<N, P>), data_ptr),
            j::jack_set_port_connect_callback(client, Some(port_connect::<N, P>), data_ptr),
            j::jack_set_graph_order_callback(client, Some(graph_order::<N, P>), data_ptr),
            j::jack_set_xrun_callback(client, Some(xrun::<N, P>), data_ptr),
        ];
        if results.iter().any(|&res| res != 0) {
            clear_callbacks(client)?;
            return Err(Error::CallbackRegistrationError);
        }
        Ok(())
    }
}

#[cfg(test)]
mod non_utf8_tests {
    use super::*;

    #[derive(Default)]
    struct Recorder {
        client_names: Vec<String>,
        shutdown_reasons: Vec<String>,
    }

    impl NotificationHandler for Recorder {
        fn shutdown(&mut self, _: ClientStatus, reason: &str) {
            self.shutdown_reasons.push(reason.to_string());
        }

        fn client_registration(&mut self, _: &Client, name: &str, _: bool) {
            self.client_names.push(name.to_string());
        }
    }

    fn new_context() -> Box<CallbackContext<Recorder, ()>> {
        let (client, _) = Client::new("", crate::ClientOptions::default()).unwrap();
        Box::new(CallbackContext::new(client, Recorder::default(), ()))
    }

    #[test]
    fn non_utf8_client_name_is_delivered_lossily() {
        let mut ctx = new_context();
        let data = CallbackContext::raw(&mut ctx);
        let name = b"bad\xff\0";
        unsafe { client_registration::<Recorder, ()>(name.as_ptr().cast(), 1, data) };
        assert!(!ctx.has_panic.load(Ordering::Relaxed));
        assert_eq!(
            ctx.notification().client_names,
            vec!["bad\u{fffd}".to_string()]
        );
    }

    #[test]
    fn null_shutdown_reason_is_delivered_as_empty() {
        let mut ctx = new_context();
        let data = CallbackContext::raw(&mut ctx);
        unsafe { shutdown::<Recorder, ()>(0, std::ptr::null(), data) };
        assert!(!ctx.has_panic.load(Ordering::Relaxed));
        assert_eq!(ctx.notification().shutdown_reasons, vec![String::new()]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;

    struct CountingHandler {
        process_calls: Arc<AtomicUsize>,
        sync_results: Vec<bool>,
        process_result: Control,
    }

    impl ProcessHandler for CountingHandler {
        const SLOW_SYNC: bool = true;

        fn process(&mut self, _: &Client, _: &ProcessScope) -> Control {
            self.process_calls.fetch_add(1, Ordering::Relaxed);
            self.process_result
        }

        fn sync(
            &mut self,
            _: &Client,
            _: crate::TransportState,
            _: &crate::TransportPosition,
        ) -> bool {
            self.sync_results.remove(0)
        }
    }

    fn new_context(
        sync_results: Vec<bool>,
        process_result: Control,
    ) -> (Box<CallbackContext<(), CountingHandler>>, Arc<AtomicUsize>) {
        let (client, _) = Client::new("", crate::ClientOptions::default()).unwrap();
        let process_calls = Arc::new(AtomicUsize::new(0));
        let ctx = Box::new(CallbackContext::new(
            client,
            (),
            CountingHandler {
                process_calls: process_calls.clone(),
                sync_results,
                process_result,
            },
        ));
        (ctx, process_calls)
    }

    #[test]
    fn quit_from_process_stops_further_process_calls() {
        let (mut ctx, process_calls) = new_context(vec![], Control::Quit);
        let data = CallbackContext::raw(&mut ctx);
        unsafe {
            assert_eq!(process::<(), CountingHandler>(128, data), -1);
            assert_eq!(process::<(), CountingHandler>(128, data), -1);
        }
        assert_eq!(process_calls.load(Ordering::Relaxed), 1);
        assert!(!ctx.has_panic.load(Ordering::Relaxed));
    }

    #[test]
    fn sync_not_ready_does_not_stop_processing() {
        let (mut ctx, process_calls) = new_context(vec![false, true], Control::Continue);
        let data = CallbackContext::raw(&mut ctx);
        let mut pos = crate::TransportPosition::default();
        let pos_ptr = &mut pos as *mut crate::TransportPosition as *mut j::jack_position_t;
        unsafe {
            assert_eq!(
                sync::<(), CountingHandler>(j::JackTransportStarting, pos_ptr, data),
                0
            );
            assert_eq!(
                sync::<(), CountingHandler>(j::JackTransportStarting, pos_ptr, data),
                1
            );
            assert_eq!(process::<(), CountingHandler>(128, data), 0);
        }
        assert_eq!(process_calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn quit_after_panic_keeps_panic_flag() {
        let (ctx, _) = new_context(vec![], Control::Quit);
        ctx.mark_invalid(true);
        ctx.mark_invalid(false);
        assert!(ctx.has_panic.load(Ordering::Relaxed));
    }
}

#[cfg(test)]
mod concurrency_tests {
    use super::*;
    use std::sync::Arc;

    #[derive(Default)]
    struct Counts {
        process: usize,
        thread_init: std::sync::atomic::AtomicUsize,
        client_registration: usize,
    }

    impl ProcessHandler for Counts {
        fn process(&mut self, _: &Client, _: &ProcessScope) -> Control {
            self.process += 1;
            Control::Continue
        }
    }

    impl NotificationHandler for Counts {
        fn thread_init(&self, _: &Client) {
            self.thread_init.fetch_add(1, Ordering::Relaxed);
        }

        fn client_registration(&mut self, _: &Client, _: &str, _: bool) {
            self.client_registration += 1;
        }
    }

    struct SendPtr(*mut libc::c_void);
    unsafe impl Send for SendPtr {}
    unsafe impl Sync for SendPtr {}

    #[test]
    fn process_and_notification_callbacks_can_run_concurrently() {
        // JACK calls process (and thread_init) on its process thread while notifications arrive on
        // another thread. Callbacks must only share the context, never hold `&mut` to it. Without
        // Miri this cannot detect aliasing UB, but it does exercise the concurrent paths and checks
        // that no update is lost.
        const ITERATIONS: usize = 10_000;
        let (client, _) = Client::new("", crate::ClientOptions::default()).unwrap();
        let mut ctx = Box::new(CallbackContext::new(
            client,
            Counts::default(),
            Counts::default(),
        ));
        let data = Arc::new(SendPtr(CallbackContext::raw(&mut ctx)));
        let name = ffi::CString::new("other").unwrap();

        let process_thread = {
            let data = data.clone();
            std::thread::spawn(move || {
                for _ in 0..ITERATIONS {
                    unsafe {
                        assert_eq!(process::<Counts, Counts>(64, data.0), 0);
                        thread_init_callback::<Counts, Counts>(data.0);
                    }
                }
            })
        };
        let notification_thread = {
            let data = data.clone();
            let name_ptr = name.as_ptr() as usize;
            std::thread::spawn(move || {
                for _ in 0..ITERATIONS {
                    unsafe {
                        client_registration::<Counts, Counts>(
                            name_ptr as *const libc::c_char,
                            1,
                            data.0,
                        );
                    }
                }
            })
        };
        process_thread.join().unwrap();
        notification_thread.join().unwrap();

        let (_client, notification, process) = ctx.into_parts();
        assert_eq!(process.process, ITERATIONS);
        assert_eq!(notification.client_registration, ITERATIONS);
        assert_eq!(notification.thread_init.load(Ordering::Relaxed), ITERATIONS);
    }
}
