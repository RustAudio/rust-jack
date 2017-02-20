use std::mem;
use std::ops::Deref;

use jack_sys as j;

use client::base::Client;
use client::common::{CREATE_OR_DESTROY_CLIENT_MUTEX, sleep_on_test};
use jack_enums::*;
use super::callbacks::{clear_callbacks, register_callbacks};

pub use super::callbacks::{NotificationHandler, ProcessHandler, ClosureProcessHandler};

/// A JACK client that is processing data asynchronously, in real-time.
///
/// # Example
/// ```
/// use jack::prelude as j;
/// 
/// // Create a client and a handler
/// let (client, _status) = j::Client::new("my_client", j::client_options::NO_START_SERVER).unwrap();
/// let process_handler = j::ClosureProcessHandler::new(move |_: &j::Client, _: &j::ProcessScope| {
///     j::JackControl::Continue
/// });
///
/// // An active async client is created, `client` is consumed.
/// let active_client = j::AsyncClient::new(client, (), process_handler).unwrap();
/// ```
#[derive(Debug)]
pub struct AsyncClient<N: NotificationHandler, P: ProcessHandler> {
    client: Client,
    handler: *mut (N, P, *mut j::jack_client_t),
}

impl<N, P> AsyncClient<N, P> where N: NotificationHandler, P: ProcessHandler{
    /// Tell the JACK server that the program is ready to start processing audio. JACK will call the
    /// methods specified by the `NotificationHandler` and `ProcessHandler` objects.
    ///
    /// On failure, either `Err(JackErr::CallbackRegistrationError)` or
    /// `Err(JackErr::ClientActivationError)` is returned.
    ///
    /// `handler` is consumed, but it is returned when `Client::deactivate` is
    /// called.
    pub fn new(client: Client, notification_handler: N, process_handler: P) -> Result<Self, JackErr> {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();
        unsafe {
            sleep_on_test();
            let handler_ptr = try!(register_callbacks(notification_handler, process_handler, client.as_ptr()));
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
                            client: client,
                            handler: handler_ptr,
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
    pub fn deactivate(self) -> Result<(Client, N, P), JackErr> {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();
        unsafe {
            // Collect contents, cleanup will be manual, instead of automatic as we don't want to
            // drop our inner client, since it may still be open.
            let (client_ptr, handler_ptr) = (self.client.as_ptr(), self.handler);
            mem::forget(self); // we're deactivating now, so no need to do it on drop

            // deactivate
            sleep_on_test();
            if j::jack_deactivate(client_ptr) != 0 {
                return Err(JackErr::ClientDeactivationError);
            }

            // clear the callbacks
            sleep_on_test();
            try!(clear_callbacks(client_ptr));

            // done, take ownership of pointer
            let handler_box = Box::from_raw(handler_ptr);
            let handler_tuple = *handler_box;
            let (n_handler, p_handler, _client_ptr) = handler_tuple;
            Ok((Client::from_raw(client_ptr), n_handler, p_handler))
        }
    }
}

impl<N, P> Deref for AsyncClient<N, P>
where N: NotificationHandler, P: ProcessHandler{
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

/// Closes the client.
impl<N, P> Drop for AsyncClient<N, P>
    where N: NotificationHandler, P: ProcessHandler {
    fn drop(&mut self) {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();
        unsafe {
            // Deactivate the handler
            sleep_on_test();
            j::jack_deactivate(self.client.as_ptr()); // result doesn't matter
            sleep_on_test();

            // Drop the handler
            drop(Box::from_raw(self.handler));

            // The client will close itself on drop
        }
    }
}
