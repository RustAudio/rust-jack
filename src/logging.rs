// The MIT License (MIT)
//
// Copyright (c) 2017 Will Medrano (will.s.medrano@gmail.com)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use std::ffi;
use std::io::{Write, stderr};
use std::sync::{Mutex, ONCE_INIT, Once};

use jack_sys as j;

lazy_static! {
    static ref INFO_FN: Mutex<Option<fn(&str)>> = Mutex::new(None);
    static ref ERROR_FN: Mutex<Option<fn(&str)>> = Mutex::new(None);
}

unsafe extern "C" fn error_wrapper(msg: *const i8) {
    let msg = ffi::CStr::from_ptr(msg).to_str().unwrap_or("rust failed to interpret error message");
    let f = ERROR_FN.lock().unwrap();
    match *f {
        Some(f) => f(msg),
        None => writeln!(&mut stderr(), "{}", msg).unwrap(),
    }
}

unsafe extern "C" fn info_wrapper(msg: *const i8) {
    let msg = ffi::CStr::from_ptr(msg).to_str().unwrap_or("rust failed to interpret info message");
    let f = INFO_FN.lock().unwrap();
    match *f {
        Some(f) => f(msg),
        None => println!("{}", msg),
    }
}

static IS_INFO_CALLBACK_SET: Once = ONCE_INIT;
/// Set the global JACK info callback. It is recommended to use the [log
/// crate](https://cratse.io/crates/log).
pub fn set_info_callback(info: fn(&str)) {
    *INFO_FN.lock().unwrap() = Some(info);
    IS_INFO_CALLBACK_SET.call_once(|| unsafe { j::jack_set_info_function(Some(info_wrapper)) })
}

/// Resets the JACK info callback to use stdio.

/// Get the info callback that was set using `set_info_callback`. This corresponds to the one set
/// using rust-jack, not JACK itself. `None` is returned if rust-jack hasn't set a callback or has
/// reset it to use stdout.
pub fn get_info_callback() -> Option<fn(&str)> {
    *INFO_FN.lock().unwrap()
}

/// Restores the JACK info callback to the JACK default, which is to write to stdout.
pub fn reset_info_callback() {
    *INFO_FN.lock().unwrap() = None;
}

static IS_ERROR_CALLBACK_SET: Once = ONCE_INIT;
/// Set the global JACK error callback. It is recommended to use the [log
/// crate](https://cratse.io/crates/log).
pub fn set_error_callback(error: fn(&str)) {
    *ERROR_FN.lock().unwrap() = Some(error);
    IS_ERROR_CALLBACK_SET.call_once(|| unsafe { j::jack_set_error_function(Some(error_wrapper)) })
}

/// Get the error callback that was set using `set_error_callback`. This corresponds to the one set
/// using rust-jack, not JACK itself. `None` is returned if rust-jack hasn't set a callback or has
/// reset it to use stderr.
pub fn get_error_callback() -> Option<fn(&str)> {
    *ERROR_FN.lock().unwrap()
}
/// Restores the JACK info callback to the JACK default, which is to write to stderr.
pub fn reset_error_callback() {
    *ERROR_FN.lock().unwrap() = None;
}
