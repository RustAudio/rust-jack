use jack_sys as j;
use libc;
use std::ffi;
use std::io::{stderr, Write};
use std::sync::{Mutex, Once, ONCE_INIT};

lazy_static! {
    static ref INFO_FN: Mutex<Option<fn(&str)>> = Mutex::new(None);
    static ref ERROR_FN: Mutex<Option<fn(&str)>> = Mutex::new(None);
}

unsafe extern "C" fn error_wrapper(msg: *const libc::c_char) {
    let msg = ffi::CStr::from_ptr(msg)
        .to_str()
        .unwrap_or("rust failed to interpret error message");
    let f = ERROR_FN.lock().unwrap();
    match *f {
        Some(f) => f(msg),
        None => writeln!(&mut stderr(), "{}", msg).unwrap(),
    }
}

unsafe extern "C" fn info_wrapper(msg: *const libc::c_char) {
    let msg = ffi::CStr::from_ptr(msg)
        .to_str()
        .unwrap_or("rust failed to interpret info message");
    let f = INFO_FN.lock().unwrap();
    match *f {
        Some(f) => f(msg),
        None => println!("{}", msg),
    }
}

static IS_INFO_CALLBACK_SET: Once = ONCE_INIT;
/// Set the global JACK info callback. It is recommended to specify a callback that uses the [log
/// crate](https://cratse.io/crates/log).
pub fn set_info_callback(info: fn(&str)) {
    *INFO_FN.lock().unwrap() = Some(info);
    IS_INFO_CALLBACK_SET.call_once(|| unsafe { j::jack_set_info_function(Some(info_wrapper)) })
}

/// Resets the JACK info callback to use stdio.

/// Get the info callback that was set using `set_info_callback`. This corresponds to the one set
/// using rust-jack, not JACK itself. `None` is returned if rust-jack hasn't set a callback or has
/// reset it to use stdout.
pub fn info_callback() -> Option<fn(&str)> {
    *INFO_FN.lock().unwrap()
}

/// Restores the JACK info callback to the JACK default, which is to write to
/// stdout.
pub fn reset_info_callback() {
    *INFO_FN.lock().unwrap() = None;
}

static IS_ERROR_CALLBACK_SET: Once = ONCE_INIT;
/// Set the global JACK info callback. It is recommended to specify a callback that uses the [log
/// crate](https://cratse.io/crates/log).
pub fn set_error_callback(error: fn(&str)) {
    *ERROR_FN.lock().unwrap() = Some(error);
    IS_ERROR_CALLBACK_SET.call_once(|| unsafe {
        j::jack_set_error_function(Some(error_wrapper))
    })
}

/// Get the error callback that was set using `set_error_callback`. This corresponds to the one set
/// using rust-jack, not JACK itself. `None` is returned if rust-jack hasn't set a callback or has
/// reset it to use stderr.
pub fn error_callback() -> Option<fn(&str)> {
    *ERROR_FN.lock().unwrap()
}
/// Restores the JACK info callback to the JACK default, which is to write to
/// stderr.
pub fn reset_error_callback() {
    *ERROR_FN.lock().unwrap() = None;
}

#[cfg(test)]
mod test {
    use super::*;

    fn null_log_fn(_: &str) {}

    #[test]
    fn logging_can_set_info() {
        // initial state
        reset_info_callback();
        assert!(info_callback().is_none());

        // set
        set_info_callback(null_log_fn);
        assert!(info_callback().is_some());
        info_callback().unwrap()("Using info callback!.");

        // reset
        reset_info_callback();
        assert!(info_callback().is_none());
    }

    #[test]
    fn logging_can_set_error() {
        // initial state
        reset_error_callback();
        assert!(error_callback().is_none());

        // set
        set_error_callback(null_log_fn);
        assert!(error_callback().is_some());
        error_callback().unwrap()("Using error callback!.");

        // reset
        reset_error_callback();
        assert!(error_callback().is_none());
    }
}
