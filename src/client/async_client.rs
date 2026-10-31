use jack_sys as j;
use std::fmt;
use std::fmt::Debug;
use std::sync::atomic::AtomicBool;

use super::callbacks::clear_callbacks;
use super::callbacks::{CallbackContext, NotificationHandler, ProcessHandler};
use crate::Error;
use crate::client::client_impl::Client;
use crate::client::common::CREATE_OR_DESTROY_CLIENT_MUTEX;

/// A JACK client that is processing data asynchronously, in real-time.
///
/// To create input or output (either sound or midi), a `Port` can be used within the `process`
/// callback. See `Client::register_port` on creating ports. Also, see `Port` for documentation on
/// the API for port.
///
/// # Example
/// ```
/// // Create a client and a handler
/// let (client, _status) =
///     jack::Client::new("my_client", jack::ClientOptions::default()).unwrap();
/// let process_handler = jack::contrib::ClosureProcessHandler::new(
///     move |_: &jack::Client, _: &jack::ProcessScope| jack::Control::Continue,
/// );
///
/// // An active async client is created, `client` is consumed.
/// let active_client = client.activate_async((), process_handler).unwrap();
/// // When done, deactivate the client.
/// if let Err(err) = active_client.deactivate() {
///     eprintln!("Error deactivating client: {err}");
/// };
/// ```
#[must_use = "Processing stops when the AsyncClient is dropped. You most likely want to keep this alive and manually tear down with `AsyncClient::deactivate`."]
pub struct AsyncClient<N, P> {
    callback: Option<Box<CallbackContext<N, P>>>,
}

unsafe impl<N, P> Send for AsyncClient<N, P> {}
unsafe impl<N, P> Sync for AsyncClient<N, P> {}

impl<N, P> AsyncClient<N, P>
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    /// Tell the JACK server that the program is ready to start processing audio. JACK will call the
    /// methods specified by the `NotificationHandler` and `ProcessHandler` objects.
    ///
    /// On failure, either `Err(Error::CallbackRegistrationError)` or
    /// `Err(Error::ClientActivationError)` is returned. Callbacks are detached and both handlers
    /// are dropped. Ports and transport objects may keep the JACK client open.
    /// If callback cleanup also fails, the callback context is retained to keep JACK's pointers valid.
    ///
    /// `notification_handler` and `process_handler` are consumed, but they are returned when
    /// `Client::deactivate` is called.
    pub fn new(client: Client, notification_handler: N, process_handler: P) -> Result<Self, Error> {
        Self::new_with_activation(
            client,
            notification_handler,
            process_handler,
            |client| unsafe { j::jack_activate(client.raw()) },
        )
    }

    // Inject activation so tests can exercise failures independently of the JACK implementation.
    fn new_with_activation(
        client: Client,
        notification_handler: N,
        process_handler: P,
        activate: impl FnOnce(&Client) -> libc::c_int,
    ) -> Result<Self, Error> {
        let mut callback_context = Box::new(CallbackContext {
            client,
            notification: notification_handler,
            process: process_handler,
            is_valid_for_callback: AtomicBool::new(true),
            has_panic: AtomicBool::new(false),
            callback_lock: std::sync::Mutex::new(()),
        });
        // Declare the guard after the context so it is released before error cleanup:
        // ClientHandle::drop also acquires this mutex.
        let _m = CREATE_OR_DESTROY_CLIENT_MUTEX.lock().ok();
        let registration = unsafe { CallbackContext::register_callbacks(&mut callback_context) };
        let result = registration.and_then(|()| {
            if activate(&callback_context.client) == 0 {
                Ok(())
            } else {
                Err(Error::ClientActivationError)
            }
        });
        if let Err(error) = result {
            if Self::detach_callbacks(&callback_context).is_err() {
                crate::logging::log_error(
                    b"Failed to detach callbacks after activation error. Leaking callback context.\0",
                );
                // JACK may still hold pointers into the context. Retain it rather
                // than free memory that a callback could still access.
                Box::leak(callback_context);
            }
            return Err(error);
        }
        Ok(AsyncClient {
            callback: Some(callback_context),
        })
    }
}

impl<N, P> AsyncClient<N, P> {
    /// Return the underlying `jack::Client`.
    #[inline(always)]
    pub fn as_client(&self) -> &Client {
        let callback = self.callback.as_ref().unwrap();
        &callback.client
    }

    /// Tell the JACK server to remove this client from the process graph.  Also, disconnect all
    /// ports belonging to it since inactive clients have no port connections.
    ///
    /// The `handler` that was used for `Client::activate` is returned on success. Its state may
    /// have changed due to JACK calling its methods.
    ///
    /// In the case of error, the `Client` is dropped. Ports and transport objects may keep the
    /// underlying JACK client open.
    /// If callback cleanup fails, the callback context is retained to keep JACK's pointers valid.
    pub fn deactivate(self) -> Result<(Client, N, P), Error> {
        let mut c = self;
        c.maybe_deactivate()
            .map(|c| (c.client, c.notification, c.process))
    }

    // Helper function for deactivating. Any function that calls this should
    // have ownership of self and no longer use it after this call.
    fn maybe_deactivate(&mut self) -> Result<Box<CallbackContext<N, P>>, Error> {
        let m = CREATE_OR_DESTROY_CLIENT_MUTEX.lock();
        if self.callback.is_none() {
            drop(m);
            return Err(Error::ClientIsNoLongerAlive);
        }
        let cb = self.callback.take().ok_or(Error::ClientIsNoLongerAlive)?;
        if let Err(error) = Self::detach_callbacks(&cb) {
            crate::logging::log_error(
                b"Failed to detach callbacks on deactivation. Leaking callback context.\0",
            );
            // Keep the callback data alive if JACK could still use it.
            Box::leak(cb);
            drop(m);
            return Err(error);
        }

        // done, take ownership of callback
        if cb.has_panic.load(std::sync::atomic::Ordering::Relaxed) {
            drop(m);
            return Err(Error::ClientPanicked);
        }
        Ok(cb)
    }

    fn detach_callbacks(cb: &CallbackContext<N, P>) -> Result<(), Error> {
        {
            let _guard = cb.callback_lock.lock();
            cb.is_valid_for_callback
                .store(false, std::sync::atomic::Ordering::Relaxed);
        }
        if unsafe { j::jack_deactivate(cb.client.raw()) } != 0 {
            return Err(Error::ClientDeactivationError);
        }
        unsafe { clear_callbacks(cb.client.raw()) }
    }
}

/// Stops processing and releases this reference to the client.
impl<N, P> Drop for AsyncClient<N, P> {
    // Deactivate and release the client.
    fn drop(&mut self) {
        let _ = self.maybe_deactivate();
    }
}

impl<N, P> Debug for AsyncClient<N, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_tuple("AsyncClient")
            .field(&self.as_client())
            .finish()
    }
}

impl<N, P> AsRef<Client> for AsyncClient<N, P> {
    fn as_ref(&self) -> &Client {
        self.as_client()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AudioIn, ClientOptions, Control, Port, ProcessScope, Unowned};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc::{SyncSender, sync_channel},
    };

    struct Handler {
        drops: Arc<AtomicUsize>,
        port: Port<Unowned>,
        processed: Option<SyncSender<()>>,
    }

    impl NotificationHandler for Handler {}

    impl ProcessHandler for Handler {
        fn process(&mut self, _: &Client, _: &ProcessScope) -> Control {
            assert_eq!(self.drops.load(Ordering::SeqCst), 0);
            if let Some(sender) = &self.processed {
                sender.try_send(()).ok();
            }
            Control::Continue
        }
    }

    impl Drop for Handler {
        fn drop(&mut self) {
            // Each handler's port keeps the client alive during destruction.
            assert!(self.port.name().is_ok());
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn check_activation_failure(start_processing: bool) {
        let (client, _) = Client::new("", ClientOptions::default()).unwrap();
        let port = client.register_port("input", AudioIn::default()).unwrap();
        let transport = client.transport();
        let life = Arc::downgrade(&transport.client);
        let notification_drops = Arc::new(AtomicUsize::new(0));
        let process_drops = Arc::new(AtomicUsize::new(0));
        let (send, recv) = sync_channel(1);
        let notification = Handler {
            drops: notification_drops.clone(),
            port: port.clone_unowned(),
            processed: None,
        };
        let process = Handler {
            drops: process_drops.clone(),
            port: port.clone_unowned(),
            processed: Some(send),
        };
        let result = AsyncClient::new_with_activation(client, notification, process, |client| {
            if start_processing {
                // Model a failure after JACK has started using the registered callback pointers.
                assert_eq!(unsafe { j::jack_activate(client.raw()) }, 0);
                recv.recv_timeout(std::time::Duration::from_secs(2))
                    .unwrap();
            }
            -1
        });
        assert_eq!(result.err(), Some(Error::ClientActivationError));
        assert_eq!(notification_drops.load(Ordering::SeqCst), 1);
        assert_eq!(process_drops.load(Ordering::SeqCst), 1);
        assert!(port.name().is_ok());
        // The retained client must be safe to activate after its original
        // callback context and handlers have been freed.
        assert_eq!(unsafe { j::jack_activate(port.client_ptr()) }, 0);
        assert_eq!(unsafe { j::jack_deactivate(port.client_ptr()) }, 0);
        drop(port);
        assert!(life.upgrade().is_some());
        drop(transport);
        assert!(life.upgrade().is_none());
    }

    #[test]
    fn activation_failure_detaches_callbacks_and_drops_handlers() {
        check_activation_failure(false);
    }

    #[test]
    fn activation_failure_after_start_detaches_callbacks_and_drops_handlers() {
        check_activation_failure(true);
    }

    #[test]
    fn dropping_async_client_detaches_callbacks_while_port_keeps_client_alive() {
        let (client, _) = Client::new("", ClientOptions::default()).unwrap();
        let port = client.register_port("input", AudioIn::default()).unwrap();
        let transport = client.transport();
        let life = Arc::downgrade(&transport.client);
        let drops = Arc::new(AtomicUsize::new(0));
        let handler = || Handler {
            drops: drops.clone(),
            port: port.clone_unowned(),
            processed: None,
        };
        let active = client.activate_async(handler(), handler()).unwrap();

        drop(active);
        assert_eq!(drops.load(Ordering::SeqCst), 2);
        assert!(port.name().is_ok());
        assert_eq!(unsafe { j::jack_activate(port.client_ptr()) }, 0);
        assert_eq!(unsafe { j::jack_deactivate(port.client_ptr()) }, 0);

        drop(port);
        drop(transport);
        assert!(life.upgrade().is_none());
    }
}
