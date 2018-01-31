use jack_sys as j;
use std::fmt;

use super::callbacks::{CallbackContext, NotificationHandler, ProcessHandler};
use super::callbacks::clear_callbacks;
use Error;
use client::client::Client;
use client::common::{sleep_on_test, CREATE_OR_DESTROY_CLIENT_MUTEX};

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
///     jack::Client::new("my_client", jack::ClientOptions::NO_START_SERVER).unwrap();
/// let process_handler = jack::ClosureProcessHandler::new(
///     move |_: &jack::Client, _: &jack::ProcessScope| jack::Control::Continue,
/// );
///
/// // An active async client is created, `client` is consumed.
/// let active_client = client.activate_async((), process_handler).unwrap();
/// ```
pub struct AsyncClient<N, P> {
    callback: Option<Box<CallbackContext<N, P>>>,
}

unsafe impl<N: Send, P: Send> Send for AsyncClient<N, P> {}

impl<N, P> AsyncClient<N, P>
where
    N: NotificationHandler,
    P: ProcessHandler,
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
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();
        unsafe {
            sleep_on_test();
            let mut callback_context = Box::new(CallbackContext {
                client: client,
                notification: notification_handler,
                process: process_handler,
            });
            CallbackContext::register_callbacks(&mut callback_context)?;
            sleep_on_test();
            let res = j::jack_activate(callback_context.client.raw());
            for _ in 0..4 {
                sleep_on_test();
            }
            match res {
                0 => Ok(AsyncClient {
                    callback: Some(callback_context),
                }),
                _ => Err(Error::ClientActivationError),
            }
        }
    }
}


impl<N, P> AsyncClient<N, P> {
    /// Return the underlying `jack::Client`.
    #[inline(always)]
    pub fn as_client<'a>(&'a self) -> &'a Client {
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

    // Helper function for deactivating. Any function that calls this should no
    // longer use self.
    fn maybe_deactivate(&mut self) -> Result<CallbackContext<N, P>, Error> {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();
        if self.callback.is_none() {
            return Err(Error::ClientIsNoLongerAlive);
        }
        let client = self.callback.take().unwrap().client.raw();
        // Prevent the callback from being deallocated in case deactivation
        // fails.
        let callback = Box::into_raw(self.callback.take().unwrap());

        // deactivate
        sleep_on_test();
        if unsafe { j::jack_deactivate(client) != 0 } {
            return Err(Error::ClientDeactivationError);
        }

        // clear the callbacks
        sleep_on_test();
        unsafe { clear_callbacks(client)? };

        // done, take ownership of callback
        Ok(*unsafe { Box::from_raw(callback) })
    }
}

/// Closes the client.
impl<N, P> Drop for AsyncClient<N, P> {
    /// Deactivate and close the client.
    fn drop(&mut self) {
        let _ = self.maybe_deactivate();
    }
}

impl<N, P> fmt::Debug for AsyncClient<N, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "AsyncClient({:?})", self.as_client())
    }
}
