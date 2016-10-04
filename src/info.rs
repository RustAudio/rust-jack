use std::io::{Write, stderr};
use std::ffi;
use std::sync::{Once, ONCE_INIT};
use jack_sys as j;

fn to_stdout(msg: &str) {
    println!("{}", msg);
}

fn to_stderr(msg: &str) {
    writeln!(&mut stderr(), "{}", msg).unwrap();
}

static mut info_fn: fn (&str) = to_stdout;
static mut error_fn: fn (&str) = to_stderr;

unsafe extern "C" fn error_wrapper(msg: *const i8) {
    let msg = ffi::CStr::from_ptr(msg).to_str().unwrap();
    error_fn(msg);
}

unsafe extern "C" fn info_wrapper(msg: *const i8) {
    let msg = ffi::CStr::from_ptr(msg).to_str().unwrap();
    info_fn(msg)
}

static ARE_CALLBACKS_SET: Once = ONCE_INIT;
/// TODO: Provide better API for this functionality
pub fn set_info_callbacks(info: Option<fn (&str)>, error: Option<fn (&str)>) {
    unsafe {
        info_fn = info.unwrap_or(to_stdout);
        error_fn = error.unwrap_or(to_stderr);
    };
    ARE_CALLBACKS_SET.call_once(|| unsafe {
        j::jack_set_error_function(Some(error_wrapper));
        j::jack_set_info_function(Some(info_wrapper));
    });
}
