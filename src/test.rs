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

use std::collections::HashSet;
use std::{thread, time};
use super::*;

fn info_handler(msg: &str) {
    panic!("Info: {}", msg);
}

fn error_handler(msg: &str) {
    panic!("Error Occurred!: {}", msg);
}

struct TestHandler {
    pub callbacks_used: HashSet<&'static str>,
}

impl TestHandler {
    pub fn new() -> Self {
        TestHandler {
            callbacks_used: HashSet::new(),
        }
    }
}

impl JackHandler for TestHandler {
    fn thread_init(&mut self) {
        self.callbacks_used.insert("thread_init");
    }

    fn shutdown(&mut self, _: ClientStatus, _: &str) {
        self.callbacks_used.insert("shutdown");
    }

    fn process(&mut self, _: u32) -> JackControl {
        self.callbacks_used.insert("process");
        JackControl::Continue
    }

    fn freewheel(&mut self, _: bool) {
        self.callbacks_used.insert("freewheel");
    }
}

#[test]
fn static_fns() {
    Client::<TestHandler>::name_size();
    Port::name_size();
    Port::type_size();
}

#[test]
fn test() {
    // info/error handling
    set_info_callbacks(Some(info_handler), Some(error_handler));

    // create client
    let mut client = Client::open("rj-test", NO_START_SERVER).unwrap();
    assert_eq!(client.status(), ClientStatus::empty());
    assert_eq!(client.name(), "rj-test");

    // query parameters
    let _audio_type_buffer_size = unsafe { client.type_buffer_size(DEFAULT_AUDIO_TYPE) };
    let _midi_type_buffer_size = unsafe { client.type_buffer_size(DEFAULT_MIDI_TYPE) };

    // test run
    client.activate(TestHandler::new()).unwrap();
    thread::sleep(time::Duration::from_secs(1));
    let tested_handler = client.deactivate().unwrap();
    let expected_called = ["thread_init", "process"];
    for s in expected_called.iter() {
        assert!(tested_handler.callbacks_used.contains(s));
    };

    // close
    client.close();
}
