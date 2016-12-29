use std::ffi;
use std::sync::{Once, ONCE_INIT};
use jack_sys as j;

fn to_nothing(_: &str) {}

static mut INFO_FN: fn(&str) = to_nothing;
static mut ERROR_FN: fn(&str) = to_nothing;

unsafe extern "C" fn error_wrapper(msg: *const i8) {
    let msg = ffi::CStr::from_ptr(msg).to_str().unwrap();
    ERROR_FN(msg);
}

unsafe extern "C" fn info_wrapper(msg: *const i8) {
    let msg = ffi::CStr::from_ptr(msg).to_str().unwrap();
    INFO_FN(msg)
}

static IS_INFO_CALLBACK_SET: Once = ONCE_INIT;
/// Set the global Jack info callback.
///
/// # Example
/// ```rust
/// fn info_log(msg: &str) {
///     println!("{}", msg);
/// }
/// jack::set_info_callback(info_log);
/// ```
pub fn set_info_callback(info: fn(&str)) {
    unsafe {
        INFO_FN = info;
    }
    IS_INFO_CALLBACK_SET.call_once(|| unsafe { j::jack_set_info_function(Some(info_wrapper)) })
}

static IS_ERROR_CALLBACK_SET: Once = ONCE_INIT;
/// Set the global Jack error callback.
///
/// # Example
/// ```rust
/// fn error_log(msg: &str) {
///     println!("{}", msg);
/// }
/// jack::set_error_callback(error_log);
/// ```
pub fn set_error_callback(error: fn(&str)) {
    unsafe {
        ERROR_FN = error;
    }
    IS_ERROR_CALLBACK_SET.call_once(|| unsafe { j::jack_set_error_function(Some(error_wrapper)) })
}
