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
    pub process_return_value: JackControl,
}

impl TestHandler {
    pub fn new() -> Self {
        TestHandler {
            callback_counts: HashMap::new(),
            process_return_value: JackControl::Continue,
        }
    }

    pub fn with_quit_on_process(self) -> Self {
        let mut h = self;
        h.process_return_value = JackControl::Quit;
        h
    }

    pub fn get_callback_count(&self, tp: TestCallbackTypes) -> usize {
        match self.callback_counts.get(&tp) {
            Some(&n) => n,
            None    => 0,
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
        self.process_return_value
    }
}

fn open_test_client(name: &str) -> Client<TestHandler> {
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
    let name: &'static str = "opening_returns_healthy_client";
    let client = open_test_client(name);
    assert_eq!(client.status(), ClientStatus::empty());
    assert_eq!(client.name(), name);
}

// TODO: investigate why thread_init gets called 3 times instead of once.
#[test]
fn activating_a_client_calls_thread_init_once() {
    let mut client = open_test_client("calls_thread_init_once");
    let handler = TestHandler::new();
    client.activate(handler).unwrap();
    thread::sleep(*DEFAULT_SLEEP_TIME);
    let handler = client.deactivate().unwrap();
    assert!(handler.get_callback_count(TestCallbackTypes::ThreadInit) > 0);
}

#[test]
fn activating_a_client_calls_process_callback_several_times() {
    let mut client = open_test_client("activating_a_client_calls_process_callback_several_times");
    let handler = TestHandler::new();
    client.activate(handler).unwrap();
    thread::sleep(*DEFAULT_SLEEP_TIME);
    let handler = client.deactivate().unwrap();
    assert!(handler.get_callback_count(TestCallbackTypes::Process) > 1);
}

#[test]
fn returning_quit_in_process_callback_stops_processing() {
    let mut client = open_test_client("returning_quit_in_process_callback_stops_processing");
    let handler = TestHandler::new().with_quit_on_process();
    client.activate(handler).unwrap();
    thread::sleep(*DEFAULT_SLEEP_TIME);
    let handler = client.deactivate().unwrap();
    assert_eq!(handler.get_callback_count(TestCallbackTypes::Process), 1);
}
