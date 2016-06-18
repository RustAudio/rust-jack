// The MIT License (MIT)

// Copyright (c) 2016 Will S Medrano

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

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

extern "C" fn error_wrapper(msg: *const i8) {
    unsafe {
        let msg = ffi::CStr::from_ptr(msg).to_str().unwrap();
        error_fn(msg);
    };
}

extern "C" fn info_wrapper(msg: *const i8) {
    unsafe {
        let msg = ffi::CStr::from_ptr(msg).to_str().unwrap();
        info_fn(msg)
    };
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
