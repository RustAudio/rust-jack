use std::sync::LazyLock;

use ctor::ctor;

use crate::{Client, ClientOptions};

mod client;
mod log;
mod processing;
mod ringbuffer;
mod time;
mod transport;

pub static DEFAULT_TEST_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::new("default-test-client", ClientOptions::default())
        .unwrap()
        .0
});

#[ctor]
fn log_to_stdio() {
    crate::set_logger(crate::LoggerType::Stdio);
}
