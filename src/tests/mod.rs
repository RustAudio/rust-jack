use std::time::{Duration, Instant};

use ctor::ctor;

use crate::{AsyncClient, Client, ClientOptions};

mod client;
mod invalid_input;
mod log;
mod port;
mod processing;
mod ringbuffer;
mod time;
mod transport;

// There is deliberately no shared static test client. A client that is still open when the test
// binary exits races with PipeWire's library destructors and can abort the process.

/// Open a new client and activate it with handlers that do nothing.
///
/// Some servers, such as PipeWire, only report timing information for active clients.
pub fn active_client() -> AsyncClient<(), ()> {
    let (client, _) = Client::new("", ClientOptions::default()).unwrap();
    client.activate_async((), ()).unwrap()
}

/// Poll `f` until it returns `true` or `timeout` passes. Returns the last result.
///
/// Servers apply some changes asynchronously, so tests wait for them instead of checking once.
pub fn wait_for(timeout: Duration, mut f: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if f() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[ctor]
fn log_to_stdio() {
    crate::set_logger(crate::LoggerType::Stdio);
}
