use std::sync::LazyLock;

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
