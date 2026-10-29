use jack_sys as j;
use lazy_static::lazy_static;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::Error;

/// The maximum length of the JACK client name string. Unlike the "C" JACK API, this does not take
/// into account the final `NULL` character and instead corresponds directly to `.len()`. This value
/// is constant.
fn client_name_size() -> usize {
    let s = unsafe { j::jack_client_name_size() - 1 };
    s as usize
}

lazy_static! {
    /// The maximum string length for port names.
    pub static ref CLIENT_NAME_SIZE: usize = client_name_size();
}

lazy_static! {
    pub static ref CREATE_OR_DESTROY_CLIENT_MUTEX: Mutex<()> = Mutex::new(());
}

/// Tracks whether a client is still open, so that objects derived from it (such as `Port` and
/// `Transport`) can refuse to call into a closed client.
///
/// Callers enter with `ClientLife::enter` and hold the returned guard for the whole FFI call.
/// `ClientLife::close` marks the client as closed and then waits for every outstanding guard to be
/// dropped, so the client is never closed while it is in use. Entering is lock-free, so it is
/// realtime safe.
pub(crate) struct ClientLife {
    closed: AtomicBool,
    users: AtomicUsize,
}

/// Keeps a client open while it is held. See `ClientLife`.
pub(crate) struct ClientLifeGuard<'a>(&'a ClientLife);

impl ClientLife {
    pub(crate) fn new() -> Arc<ClientLife> {
        Arc::new(ClientLife {
            closed: AtomicBool::new(false),
            users: AtomicUsize::new(0),
        })
    }

    /// Returns a guard that keeps the client open, or `Err(Error::ClientIsNoLongerAlive)` if the
    /// client has been closed.
    pub(crate) fn enter(&self) -> Result<ClientLifeGuard<'_>, Error> {
        // Increment before checking `closed`. Paired with `close`, which sets `closed` before
        // reading `users`, this guarantees that either we see the client as closed or `close`
        // waits for us.
        self.users.fetch_add(1, Ordering::SeqCst);
        let guard = ClientLifeGuard(self);
        if self.closed.load(Ordering::SeqCst) {
            return Err(Error::ClientIsNoLongerAlive);
        }
        Ok(guard)
    }

    /// Mark the client as closed and wait until no guards are outstanding.
    pub(crate) fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
        while self.users.load(Ordering::SeqCst) != 0 {
            std::thread::yield_now();
        }
    }
}

impl Drop for ClientLifeGuard<'_> {
    fn drop(&mut self) {
        self.0.users.fetch_sub(1, Ordering::SeqCst);
    }
}
