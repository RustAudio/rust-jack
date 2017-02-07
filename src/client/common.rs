use std::sync::Mutex;

#[inline(always)]
pub fn sleep_on_test() {
    #[cfg(test)]
    {
        use std::{thread, time};
        thread::sleep(time::Duration::from_millis(200));
    }
}

lazy_static! {
    pub static ref CREATE_OR_DESTROY_CLIENT_MUTEX: Mutex<()> = Mutex::new(());
}
