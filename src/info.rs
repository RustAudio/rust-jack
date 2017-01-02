use std::ffi;
use std::cell::Cell;
use std::sync::{Mutex, Once, ONCE_INIT};
use jack_sys as j;

fn to_nothing(_: &str) {}

lazy_static! {
    static ref INFO_FN: Mutex<Cell<fn(&str)>> = Mutex::new(Cell::new(to_nothing));
    static ref ERROR_FN: Mutex<Cell<fn(&str)>> = Mutex::new(Cell::new(to_nothing));
}

unsafe extern "C" fn error_wrapper(msg: *const i8) {
    let msg = ffi::CStr::from_ptr(msg).to_str().unwrap();
    ERROR_FN.lock().unwrap().get()(msg);
}

unsafe extern "C" fn info_wrapper(msg: *const i8) {
    let msg = ffi::CStr::from_ptr(msg).to_str().unwrap();
    INFO_FN.lock().unwrap().get()(msg)
}

static IS_INFO_CALLBACK_SET: Once = ONCE_INIT;
/// Set the global JACK info callback. It is recommended to use the [log
/// crate](https://cratse.io/crates/log).
pub fn set_info_callback(info: fn(&str)) {
    INFO_FN.lock().unwrap().set(info);
    IS_INFO_CALLBACK_SET.call_once(|| unsafe { j::jack_set_info_function(Some(info_wrapper)) })
}

static IS_ERROR_CALLBACK_SET: Once = ONCE_INIT;
/// Set the global JACK error callback. It is recommended to use the [log
/// crate](https://cratse.io/crates/log).
pub fn set_error_callback(error: fn(&str)) {
    ERROR_FN.lock().unwrap().set(error);
    IS_ERROR_CALLBACK_SET.call_once(|| unsafe { j::jack_set_error_function(Some(error_wrapper)) })
}
