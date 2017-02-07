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

use prelude::*;

fn null_log_fn(_: &str) {}

#[test]
fn logging_can_set_info() {
    // initial state
    reset_info_callback();
    assert!(get_info_callback().is_none());

    // set
    set_info_callback(null_log_fn);
    assert!(get_info_callback().is_some());
    get_info_callback().unwrap()("Using info callback!.");

    // reset
    reset_info_callback();
    assert!(get_info_callback().is_none());
}

#[test]
fn logging_can_set_error() {
    // initial state
    reset_error_callback();
    assert!(get_error_callback().is_none());

    // set
    set_error_callback(null_log_fn);
    assert!(get_error_callback().is_some());
    get_error_callback().unwrap()("Using error callback!.");

    // reset
    reset_error_callback();
    assert!(get_error_callback().is_none());
}
