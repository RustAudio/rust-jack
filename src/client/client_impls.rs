use std::{ffi, mem, ptr};
use std::sync::Mutex;

use jack_sys as j;

use super::callbacks::{clear_callbacks, register_callbacks};
use jack_enums::*;
use client::client_options::ClientOptions;
use client::client_status::ClientStatus;

use super::base::{JackClient, WeakClient};
pub use super::callbacks::{JackHandler, ProcessHandler};

lazy_static! {
    static ref CREATE_OR_DESTROY_CLIENT_MUTEX: Mutex<()> = Mutex::new(());
}

/// The maximum length of the JACK client name string. Unlike the "C" JACK
/// API, this does not take into account the final `NULL` character and
/// instead corresponds directly to `.len()`. This value is constant.
fn client_name_size() -> usize {
    let s = unsafe { j::jack_client_name_size() - 1 };
    s as usize
}

lazy_static! {
    /// The maximum string length for port names.
    pub static ref CLIENT_NAME_SIZE: usize = client_name_size();
}

/// A client to interact with a JACK server.
#[derive(Debug)]
pub struct Client(WeakClient);

/// A `JackClient` that is currently active. Active clients may
/// contain `JackHandler`s that are processing data in real-time.
#[derive(Debug)]
pub struct ActiveClient<JH: JackHandler> {
    client: Client,
    handler: *mut (JH, *mut j::jack_client_t),
}

unsafe impl JackClient for Client {
    fn as_ptr(&self) -> *mut j::jack_client_t {
        self.0.as_ptr()
    }
}

unsafe impl<JH: JackHandler> JackClient for ActiveClient<JH> {
    fn as_ptr(&self) -> *mut j::jack_client_t {
        self.client.as_ptr()
    }
}

impl Client {
    /// The maximum length of the JACK client name string. Unlike the "C" JACK
    /// API, this does not take into account the final `NULL` character and
    /// instead corresponds directly to `.len()`.

    /// Opens a JACK client with the given name and options. If the client is
    /// successfully opened, then `Ok(client)` is returned. If there is a
    /// failure, then `Err(JackErr::ClientError(status))` will be returned.
    ///
    /// Although the client may be successful in opening, there still may be
    /// some errors minor errors when attempting to opening. To access these,
    /// check the returned `ClientStatus`.
    pub fn open(client_name: &str,
                options: ClientOptions)
                -> Result<(Self, ClientStatus), JackErr> {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();
        sleep_on_test();
        let mut status_bits = 0;
        let client = unsafe {
            let client_name = ffi::CString::new(client_name).unwrap();
            j::jack_client_open(client_name.as_ptr(), options.bits(), &mut status_bits)
        };
        sleep_on_test();
        let status = ClientStatus::from_bits(status_bits).unwrap_or(ClientStatus::empty());
        if client.is_null() {
            Err(JackErr::ClientError(status))
        } else {
            Ok((Client(unsafe { WeakClient::from_raw(client) }), status))
        }
    }

    /// Tell the JACK server that the program is ready to start processing
    /// audio. JACK will call the methods specified by the `JackHandler` trait, from `handler`.
    ///
    /// On failure, either `Err(JackErr::CallbackRegistrationError)` or
    /// `Err(JackErr::ClientActivationError)` is returned.
    ///
    /// `handler` is consumed, but it is returned when `Client::deactivate` is
    /// called.
    pub fn activate<JH: JackHandler>(self, handler: JH) -> Result<ActiveClient<JH>, JackErr> {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();
        unsafe {
            sleep_on_test();
            let handler_ptr = try!(register_callbacks(handler, self.as_ptr()));
            sleep_on_test();
            if handler_ptr.is_null() {
                Err(JackErr::CallbackRegistrationError)
            } else {
                let res = j::jack_activate(self.as_ptr());
                for _ in 0..4 {
                    sleep_on_test();
                }
                match res {
                    0 => {
                        Ok(ActiveClient {
                            client: self,
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

    /// Toggle input monitoring for the port with name `port_name`.
    ///
    /// `Err(JackErr::PortMonitorError)` is returned on failure.
    ///
    /// Only works if the port has the `CAN_MONITOR` flag, or else nothing
    /// happens.
    pub fn request_monitor_by_name(&self,
                                   port_name: &str,
                                   enable_monitor: bool)
                                   -> Result<(), JackErr> {
        let port_name = ffi::CString::new(port_name).unwrap();
        let onoff = match enable_monitor {
            true => 1,
            false => 0,
        };
        let res = unsafe {
            j::jack_port_request_monitor_by_name(self.as_ptr(), port_name.as_ptr(), onoff)
        };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortMonitorError),
        }
    }


    // TODO implement
    // /// Start/Stop JACK's "freewheel" mode.
    // ///
    // /// When in "freewheel" mode, JACK no longer waits for any external event to
    // /// begin the start of the next process cycle. As a result, freewheel mode
    // /// causes "faster than real-time" execution of a JACK graph. If possessed,
    // /// real-time scheduling is dropped when entering freewheel mode, and if
    // /// appropriate it is reacquired when stopping.
    // ///
    // /// IMPORTANT: on systems using capabilities to provide real-time scheduling
    // /// (i.e. Linux Kernel 2.4), if enabling freewheel, this function must be
    // /// called from the thread that originally called `self.activate()`. This
    // /// restriction does not apply to other systems (e.g. Linux Kernel 2.6 or OS
    // /// X).
    // pub fn set_freewheel(&self, enable: bool) -> Result<(), JackErr> {
    //     let onoff = match enable {
    //         true => 0,
    //         false => 1,
    //     };
    //     match unsafe { j::jack_set_freewheel(self.as_ptr(), onoff) } {
    //         0 => Ok(()),
    //         _ => Err(JackErr::FreewheelError),
    //     }
    // }

    pub unsafe fn from_raw(p: *mut j::jack_client_t) -> Self {
        Client(WeakClient::from_raw(p))
    }
}

impl<JH: JackHandler> ActiveClient<JH> {
    /// Tell the JACK server to remove this client from the process graph. Also,
    /// disconnect all ports belonging to it since inactive clients have no port
    /// connections.
    ///
    /// The `handler` that was used for `Client::activate` is returned on
    /// success. Its state may have changed due to JACK calling its methods.
    ///
    /// In the case of error, the `Client` is destroyed because its state is
    /// unknown, and it is therefore unsafe to continue using.
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
                    sleep_on_test();
                    drop(client);
                    sleep_on_test();
                    Err(err)
                }
            }
        }
    }
}

/// Close the client, deactivating if necessary.
impl Drop for Client {
    fn drop(&mut self) {
        let _ = *CREATE_OR_DESTROY_CLIENT_MUTEX.lock().unwrap();

        debug_assert!(!self.as_ptr().is_null()); // Rep invariant

        // Client isn't active, so no need to deactivate

        // Close the client
        sleep_on_test();
        let res = unsafe { j::jack_client_close(self.as_ptr()) }; // close the client
        sleep_on_test();
        assert_eq!(res, 0);
        self.0 = unsafe { WeakClient::from_raw(ptr::null_mut()) };
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

#[inline(always)]
fn sleep_on_test() {
    #[cfg(test)]
    {
        use std::{thread, time};
        thread::sleep(time::Duration::from_millis(200));
    }
}
