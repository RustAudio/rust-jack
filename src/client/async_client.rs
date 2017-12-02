use std::ops::Deref;

use jack_sys as j;

use client::base::Client;
use client::common::{CREATE_OR_DESTROY_CLIENT_MUTEX, sleep_on_test};
use jack_enums::*;
use super::callbacks::{clear_callbacks, register_callbacks};

pub use super::callbacks::{NotificationHandler, ProcessHandler};

/// A JACK client that is processing data asynchronously, in real-time.
///
/// To create input or output (either sound or midi), a `Port` can be used within the `process`
/// callback. See `Client::register_port` on creating ports. Also, see `Port` for documentation on
/// the API for port.
///
/// # Example
/// ```
/// use jack::prelude as j;
///
/// // Create a client and a handler
/// let (client, _status) =
///     j::Client::new("my_client", j::client_options::NO_START_SERVER)
///         .unwrap();
/// let process_handler = j::ClosureProcessHandler::new(
///     move |_: &j::Client, _: &j::ProcessScope| {
///         j::JackControl::Continue
///     }
/// );
///
/// // An active async client is created, `client` is consumed.
/// let active_client = j::AsyncClient::new(client, (), process_handler).unwrap();
/// ```
#[derive(Debug)]
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
    /// On failure, either `Err(JackErr::CallbackRegistrationError)` or
    /// `Err(JackErr::ClientActivationError)` is returned.
    ///
    /// `notification_handler` and `process_handler` are consumed, but they are returned when
    /// `Client::deactivate` is called.
    pub fn new(
        client: Client,
        notification_handler: N,
        process_handler: P,
    ) -> Result<Self, JackErr> {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();
        unsafe {
            sleep_on_test();
            let handler_ptr = try!(register_callbacks(
                notification_handler,
                process_handler,
                client.as_ptr(),
            ));
            sleep_on_test();
            if handler_ptr.is_null() {
                Err(JackErr::CallbackRegistrationError)
            } else {
                let res = j::jack_activate(client.as_ptr());
                for _ in 0..4 {
                    sleep_on_test();
                }
                match res {
                    0 => {
                        Ok(AsyncClient {
                            client: Some(client),
                            handler: Some(handler_ptr),
                        })
                    }

                    _ => {
                        drop(Box::from_raw(handler_ptr));
                        Err(JackErr::ClientActivationError)
                    }
                }
            }
        }
    }


    /// Tell the JACK server to remove this client from the process graph. Also, disconnect all
    /// ports belonging to it since inactive clients have no port connections.
    ///
    /// The `handler` that was used for `Client::activate` is returned on success. Its state may
    /// have changed due to JACK calling its methods.
    ///
    /// In the case of error, the `Client` is destroyed because its state is unknown, and it is
    /// therefore unsafe to continue using.
    pub fn deactivate(mut self) -> Result<(Client, N, P), JackErr> {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();
        unsafe {
            let inner_client = self.client.take().unwrap();

            // deactivate
            sleep_on_test();
            if j::jack_deactivate(inner_client.as_ptr()) != 0 {
                return Err(JackErr::ClientDeactivationError);
            }

            // clear the callbacks
            sleep_on_test();
            try!(clear_callbacks(inner_client.as_ptr()));

            // done, take ownership of pointer
            let handler_box = Box::from_raw(self.handler.take().unwrap());
            let handler_tuple = *handler_box;
            let (n_handler, p_handler, _client_ptr) = handler_tuple;
            Ok((inner_client, n_handler, p_handler))
        }
    }
}

impl<N, P> Deref for AsyncClient<N, P>
where
    N: NotificationHandler,
    P: ProcessHandler,
{
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        self.client.as_ref().unwrap()
    }
}

/// Closes the client.
impl<N, P> Drop for AsyncClient<N, P>
where
    N: NotificationHandler,
    P: ProcessHandler,
{
    fn drop(&mut self) {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();
        unsafe {
            // Deactivate the handler
            sleep_on_test();
            if self.client.is_some() {
                j::jack_deactivate(self.as_ptr()); // result doesn't matter
            }
            sleep_on_test();

            // Drop the handler
            if self.handler.is_some() {
                drop(Box::from_raw(self.handler.unwrap()));
            }

            // The client will close itself once it goes out of scope.
            // self.client.unwrap().drop()
        }
    }
}
