use jack_sys as j;
use std::fmt;

use super::callbacks::{NotificationHandler, ProcessHandler};
use super::callbacks::{clear_callbacks, register_callbacks};
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
pub struct AsyncClient<N: NotificationHandler, P: ProcessHandler> {
    client: Option<Client>,
    handler: Option<*mut (N, P, *mut j::jack_client_t)>,
}

unsafe impl<N, P> Send for AsyncClient<N, P>
where
    N: NotificationHandler,
    P: ProcessHandler,
{
}

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
            let handler_ptr =
                register_callbacks(notification_handler, process_handler, client.raw())?;
            sleep_on_test();
            if handler_ptr.is_null() {
                Err(Error::CallbackRegistrationError)
            } else {
                let res = j::jack_activate(client.raw());
                for _ in 0..4 {
                    sleep_on_test();
                }
                match res {
                    0 => Ok(AsyncClient {
                        client: Some(client),
                        handler: Some(handler_ptr),
                    }),

                    _ => {
                        drop(Box::from_raw(handler_ptr));
                        Err(Error::ClientActivationError)
                    }
                }
            }
        }
    }

    /// Return the underlying `jack::Client`.
    #[inline(always)]
    pub fn as_client(&self) -> &Client {
        self.client.as_ref().unwrap()
    }

    /// Tell the JACK server to remove this client from the process graph.  Also, disconnect all
    /// ports belonging to it since inactive clients have no port connections.
    ///
    /// The `handler` that was used for `Client::activate` is returned on success. Its state may
    /// have changed due to JACK calling its methods.
    ///
    /// In the case of error, the `Client` is destroyed because its state is unknown, and it is
    /// therefore unsafe to continue using.
    pub fn deactivate(mut self) -> Result<(Client, N, P), Error> {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();
        unsafe {
            let inner_client = self.client.take().unwrap();

            // deactivate
            sleep_on_test();
            if j::jack_deactivate(inner_client.raw()) != 0 {
                return Err(Error::ClientDeactivationError);
            }

            // clear the callbacks
            sleep_on_test();
            clear_callbacks(inner_client.raw())?;

            // done, take ownership of pointer
            let handler_box = Box::from_raw(self.handler.take().unwrap());
            let handler_tuple = *handler_box;
            let (n_handler, p_handler, _client_ptr) = handler_tuple;
            Ok((inner_client, n_handler, p_handler))
        }
    }
}

/// Closes the client.
impl<N, P> Drop for AsyncClient<N, P>
where
    N: NotificationHandler,
    P: ProcessHandler,
{
    /// Deactivate and close the client.
    fn drop(&mut self) {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();
        unsafe {
            // Deactivate the handler
            sleep_on_test();
            if self.client.is_some() {
                j::jack_deactivate(self.as_client().raw()); // result doesn't matter
            }

            sleep_on_test();
            // Drop the handler
            if self.handler.is_some() {
                drop(Box::from_raw(self.handler.unwrap()));
            }

            // The client will close itself once it goes out of scope.
            // self.client.take()
        }
    }
}

impl<N, P> fmt::Debug for AsyncClient<N, P>
where
    N: NotificationHandler,
    P: ProcessHandler,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "AsyncClient({:?})", self.as_client())
    }
}
