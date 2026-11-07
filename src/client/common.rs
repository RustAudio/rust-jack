use jack_sys as j;
use std::sync::{LazyLock, Mutex};

/// The maximum length of the JACK client name string. Unlike the "C" JACK API, this does not take
/// into account the final `NULL` character and instead corresponds directly to `.len()`. This value
/// is constant.
fn client_name_size() -> usize {
    let s = unsafe { j::jack_client_name_size() - 1 };
    s as usize
}

/// The maximum string length for port names.
pub static CLIENT_NAME_SIZE: LazyLock<usize> = LazyLock::new(client_name_size);

pub static CREATE_OR_DESTROY_CLIENT_MUTEX: Mutex<()> = Mutex::new(());
