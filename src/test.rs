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

    fn process(&mut self, _: &mut ProcessScope) -> JackControl {
        self.callbacks_used.insert("process");
        JackControl::Continue
    }

    fn freewheel(&mut self, _: bool) {
        self.callbacks_used.insert("freewheel");
    }
}

#[test]
fn static_fns() {
    client_name_size();
    port_name_size();
    port_type_size();
}

#[test]
fn test() {
    // info/error handling
    set_info_callbacks(Some(info_handler), Some(error_handler));

    // create client
    let (client, status) = Client::open("rj-test", NO_START_SERVER).unwrap();
    assert_eq!(status, ClientStatus::empty());
    assert_eq!(client.name(), "rj-test");

    // query parameters
    let _audio_type_buffer_size = unsafe { client.type_buffer_size(DEFAULT_AUDIO_TYPE) };
    let _midi_type_buffer_size = unsafe { client.type_buffer_size(DEFAULT_MIDI_TYPE) };

    // test run
    let activated_client = client.activate(TestHandler::new()).unwrap();
    thread::sleep(time::Duration::from_secs(1));
    let (deactivated_client, tested_handler) = activated_client.deactivate().unwrap();
    let expected_called = ["thread_init", "process"];
    for s in expected_called.iter() {
        assert!(tested_handler.callbacks_used.contains(s));
    };

    // close
    deactivated_client.close();
}
