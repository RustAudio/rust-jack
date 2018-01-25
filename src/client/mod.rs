mod async_client;
mod client;
mod callbacks;
mod common;
mod handler_impls;

/// Contains `ClientOptions` flags used when opening a client.
mod client_options;

/// Contains `ClientStatus` flags which describe the status of a Client.
mod client_status;

pub use self::async_client::AsyncClient;
pub use self::callbacks::{NotificationHandler, ProcessHandler};
pub use self::client::{Client, CycleTimes, ProcessScope};
pub use self::client_options::ClientOptions;
pub use self::client_status::ClientStatus;
pub use self::common::CLIENT_NAME_SIZE;

pub use self::handler_impls::ClosureProcessHandler;

// client.rs excluding functionality that involves ports or callbacks
#[cfg(test)]
mod test;

#[cfg(test)]
mod test_callback;
