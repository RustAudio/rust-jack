use jack_sys as j;
use std::fmt;
use std::fmt::Debug;
use std::mem;
use std::sync::atomic::AtomicBool;

use super::callbacks::clear_callbacks;
use super::callbacks::{CallbackContext, NotificationHandler, ProcessHandler};
use crate::client::client_impl::Client;
use crate::client::common::CREATE_OR_DESTROY_CLIENT_MUTEX;
use crate::Error;

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
#[must_use = "The jack client is shut down when the AsyncClient is dropped. You most likely want to keep this alive and manually tear down with `AsyncClient::deactivate`."]
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
    /// `Err(Error::ClientActivationError)` is returned.
    ///
    /// `notification_handler` and `process_handler` are consumed, but they are returned when
    /// `Client::deactivate` is called.
    pub fn new(client: Client, notification_handler: N, process_handler: P) -> Result<Self, Error> {
        let _m = CREATE_OR_DESTROY_CLIENT_MUTEX.lock().ok();
        unsafe {
            let mut callback_context = Box::new(CallbackContext {
                client,
                notification: notification_handler,
                process: process_handler,
                is_valid_for_callback: AtomicBool::new(true),
                has_panic: AtomicBool::new(false),
            });
            CallbackContext::register_callbacks(&mut callback_context)?;
            let res = j::jack_activate(callback_context.client.raw());
            match res {
                0 => Ok(AsyncClient {
                    callback: Some(callback_context),
                }),
                _ => {
                    mem::forget(callback_context);
                    Err(Error::ClientActivationError)
                }
            }
        }
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
    /// In the case of error, the `Client` is destroyed because its state is unknown, and it is
    /// therefore unsafe to continue using.
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
        // deactivate
        if unsafe { j::jack_deactivate(cb.client.raw()) } != 0 {
            drop(m);
            return Err(Error::ClientDeactivationError);
        }

        // clear the callbacks
        unsafe { clear_callbacks(cb.client.raw()) }?;
        // done, take ownership of callback
        if cb.has_panic.load(std::sync::atomic::Ordering::Relaxed) {
            drop(m);
            return Err(Error::ClientPanicked);
        }
        Ok(cb)
    }
}

/// Closes the client.
impl<N, P> Drop for AsyncClient<N, P> {
    // Deactivate and close the client.
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
