use std::mem;
use std::ops::Deref;

use jack_sys as j;

use super::callbacks::{clear_callbacks, register_callbacks};
use jack_enums::*;
use client::base::Client;
use client::common::{CREATE_OR_DESTROY_CLIENT_MUTEX, sleep_on_test};

pub use super::callbacks::{JackHandler, ProcessHandler};

/// A JACK client that is currently active. Active clients may contain `JackHandler`s that are
/// processing data in real-time.
#[derive(Debug)]
pub struct ActiveClient<JH: JackHandler> {
    client: Client,
    handler: *mut (JH, *mut j::jack_client_t),
}

impl<JH: JackHandler> ActiveClient<JH> {
    /// Tell the JACK server that the program is ready to start processing
    /// audio. JACK will call the methods specified by the `JackHandler` trait, from `handler`.
    ///
    /// On failure, either `Err(JackErr::CallbackRegistrationError)` or
    /// `Err(JackErr::ClientActivationError)` is returned.
    ///
    /// `handler` is consumed, but it is returned when `Client::deactivate` is
    /// called.
    pub fn new(client: Client, handler: JH) -> Result<Self, JackErr> {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();
        unsafe {
            sleep_on_test();
            let handler_ptr = try!(register_callbacks(handler, client.as_ptr()));
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
                        Ok(ActiveClient {
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
    pub fn deactivate(self) -> Result<(Client, JH), JackErr> {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();
        unsafe {
            // Collect contents, cleanup will be manual, instead of automatic as we don't want to
            // drop our inner client, since it may still be open.
            let (client_ptr, handler) = (self.client.as_ptr(), self.handler);
            let client = Client::from_raw(client_ptr);
            mem::forget(self);

            // Deactivate, but not close, the client
            sleep_on_test();
            let res = match j::jack_deactivate(client.as_ptr()) {
                // We own the handler post-deactivation
                0 => Ok(Box::from_raw(handler)),

                // We may still own the handler here, but it's not safe to say
                // without more information about the error condition
                _ => Err(JackErr::ClientDeactivationError),
            };

            // Clear the callbacks
            sleep_on_test();
            let callback_res = clear_callbacks(client.as_ptr());
            sleep_on_test();

            match (res, callback_res) {
                (Ok(handler_ptr), Ok(())) => {
                    let (handler, _) = *handler_ptr;
                    Ok((client, handler))
                }
                (Err(err), _) | (_, Err(err)) => {
                    // We've invalidated the client, so it must be closed
                    drop(client);
                    Err(err)
                }
            }
        }
    }
}

impl<JH: JackHandler> Deref for ActiveClient<JH> {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

/// Closes the client.
impl<JH: JackHandler> Drop for ActiveClient<JH> {
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
