use crate::LIB;
use lazy_static::lazy_static;
use std::sync::Mutex;

/// The maximum length of the JACK client name string. Unlike the "C" JACK API, this does not take
/// into account the final `NULL` character and instead corresponds directly to `.len()`. This value
/// is constant.
fn client_name_size() -> usize {
    let s = unsafe { (LIB.jack_client_name_size)() - 1 };
    s as usize
}

lazy_static! {
    /// The maximum string length for port names.
    pub static ref CLIENT_NAME_SIZE: usize = client_name_size();
}

lazy_static! {
    pub static ref CREATE_OR_DESTROY_CLIENT_MUTEX: Mutex<()> = Mutex::new(());
}

#[inline(always)]
pub fn sleep_on_test() {
    #[cfg(test)]
    {
        use std::{thread, time};
        thread::sleep(time::Duration::from_millis(150));
    }
}
