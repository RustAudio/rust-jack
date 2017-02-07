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
use jack_sys as j;

/// Collects strings from an array of c-strings into a Rust vector of strings
/// and frees the memory pointed to by `ptr`. The end of the array is marked by
/// the value of the c-string being the null pointer. `ptr` may be `null`, in
/// which case nothing (deallocating) is done and an empty vector is returned.
pub unsafe fn collect_strs(ptr: *const *const i8) -> Vec<String> {
    if ptr.is_null() {
        return Vec::new();
    };
    let len = {
        let mut len = 0;
        while !(*ptr.offset(len)).is_null() {
            len += 1;
        }
        len
    };
    let mut strs = Vec::with_capacity(len as usize);
    for i in 0..len {
        let cstr_ptr = *ptr.offset(i);
        let s = ffi::CStr::from_ptr(cstr_ptr)
            .to_string_lossy()
            .into_owned();
        strs.push(s);
    }
    j::jack_free(ptr as *mut ::libc::c_void);
    strs
}

#[cfg(test)]
use client::JackHandler;
#[cfg(test)]
pub struct DummyHandler;
#[cfg(test)]
impl JackHandler for DummyHandler {}
