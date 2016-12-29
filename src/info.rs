use std::ffi;
use std::sync::{Once, ONCE_INIT};
use jack_sys as j;

fn to_nothing(_: &str) {}

static mut info_fn: fn(&str) = to_nothing;
static mut error_fn: fn(&str) = to_nothing;

unsafe extern "C" fn error_wrapper(msg: *const i8) {
    let msg = ffi::CStr::from_ptr(msg).to_str().unwrap();
    error_fn(msg);
}

unsafe extern "C" fn info_wrapper(msg: *const i8) {
    let msg = ffi::CStr::from_ptr(msg).to_str().unwrap();
    info_fn(msg)
}

static IS_INFO_CALLBACK_SET: Once = ONCE_INIT;
/// Set the global Jack info callback. If `None` is passed, then no
/// logging will occur.
pub fn set_info_callback(info: Option<fn(&str)>) {
    unsafe {
        info_fn = info.unwrap_or(to_nothing);
    }
    IS_INFO_CALLBACK_SET.call_once(|| unsafe { j::jack_set_info_function(Some(info_wrapper)) })
}

static IS_ERROR_CALLBACK_SET: Once = ONCE_INIT;
/// Set the global Jack error callback. If `None` is passed, then no
/// logging will occur.
pub fn set_error_callback(error: Option<fn(&str)>) {
    unsafe {
        error_fn = error.unwrap_or(to_nothing);
    }
    IS_ERROR_CALLBACK_SET.call_once(|| unsafe { j::jack_set_error_function(Some(error_wrapper)) })
}
