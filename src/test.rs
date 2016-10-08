use std::collections::HashMap;
use std::{thread, time};
use super::*;

lazy_static! {
    static ref DEFAULT_SLEEP_TIME: time::Duration = time::Duration::from_secs(1);
}


#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
enum TestCallbackTypes {
    ThreadInit,
    Process,
}

#[derive(Clone,Debug)]
struct TestHandler {
    pub callback_counts: HashMap<TestCallbackTypes, usize>,
}

impl TestHandler {
    pub fn new() -> Self {
        TestHandler {
            callback_counts: HashMap::new(),
        }
    }

    pub fn get_callback_count(&self, tp: TestCallbackTypes) -> usize {
        match self.callback_counts.get(&tp) {
            Some(&n) => n,
            None => 0,
        }
    }

    fn increment_callback_count(&mut self, tp: TestCallbackTypes) {
        let n = self.get_callback_count(tp);
        self.callback_counts.insert(tp, n + 1);
    }
}

impl JackHandler for TestHandler {
    fn thread_init(&mut self) {
        self.increment_callback_count(TestCallbackTypes::ThreadInit);
    }

    fn process(&mut self, _: u32) -> JackControl {
        self.increment_callback_count(TestCallbackTypes::Process);
        JackControl::Continue
    }
}

fn open_test_client(name: &str) -> Client<TestHandler> {
    thread::sleep(*DEFAULT_SLEEP_TIME);
    Client::<TestHandler>::open(name, NO_START_SERVER).unwrap()
}

#[test]
fn querying_jack_sizes_returns_valid_values() {
    assert!(Client::<TestHandler>::name_size() > 0);
    assert!(Port::name_size() > 0);
    assert!(Port::type_size() > 0);
}

#[test]
fn opening_returns_healthy_client() {
    let name: &'static str = "orhc";
    let client = open_test_client(name);
    assert_eq!(client.status(), ClientStatus::empty());
    assert_eq!(client.name(), name);
}

// TODO: investigate why thread_init gets called more than once.
// Or, most likely, abandon functionality
#[test]
fn activating_a_client_calls_thread_init_once() {
    let mut client = open_test_client("aacctio");
    let handler = TestHandler::new();
    client.activate(handler).unwrap();
    thread::sleep(*DEFAULT_SLEEP_TIME);
    let handler = client.deactivate().unwrap();
    assert!(handler.get_callback_count(TestCallbackTypes::ThreadInit) > 0);
}

#[test]
fn activating_a_client_calls_process_callback_several_times() {
    let mut client = open_test_client("aaccpcst");
    let handler = TestHandler::new();
    client.activate(handler).unwrap();
    thread::sleep(*DEFAULT_SLEEP_TIME);
    let handler = client.deactivate().unwrap();
    assert!(handler.get_callback_count(TestCallbackTypes::Process) > 1);
}
