mod client_impls;
mod base;
mod callbacks;

/// Contains `ClientOptions` flags used when opening a client.
pub mod client_options;

/// Contains `ClientStatus` flags which describe the status of a Client.
pub mod client_status;

pub use self::client_options::ClientOptions;
pub use self::client_status::ClientStatus;
pub use self::client_impls::*;
pub use self::base::*;
