use std::sync::Mutex;

use prelude::*;
use jack_utils::*;

#[derive(Debug, Default)]
pub struct Counter {
    pub process_return_val: JackControl,
    pub induce_xruns: bool,
    pub thread_init_count: Mutex<usize>,
    pub frames_processed: Mutex<usize>,
    pub buffer_size_change_history: Mutex<Vec<JackFrames>>,
    pub registered_client_history: Mutex<Vec<String>>,
    pub unregistered_client_history: Mutex<Vec<String>>,
    pub port_register_history: Mutex<Vec<JackPortId>>,
    pub port_unregister_history: Mutex<Vec<JackPortId>>,
    pub xruns_count: Mutex<usize>,
}

impl JackHandler for Counter {
    fn thread_init(&self) {
        *self.thread_init_count.lock().unwrap() += 1;
    }

    fn process(&self, ps: &ProcessScope) -> JackControl {
        *self.frames_processed.lock().unwrap() += ps.n_frames() as usize;
        if self.induce_xruns {
            default_sleep();
        }
        JackControl::Continue
    }

    fn buffer_size(&self, size: JackFrames) -> JackControl {
        self.buffer_size_change_history.lock().unwrap().push(size);
        JackControl::Continue
    }

    fn client_registration(&self, name: &str, is_registered: bool) {
        match is_registered {
            true => self.registered_client_history.lock().unwrap().push(name.to_string()),
            false => self.unregistered_client_history.lock().unwrap().push(name.to_string()),
        }
    }

    fn port_registration(&self, pid: JackPortId, is_registered: bool) {
        match is_registered {
            true => self.port_register_history.lock().unwrap().push(pid),
            false => self.port_unregister_history.lock().unwrap().push(pid),
        }
    }

    fn xrun(&self) -> JackControl {
        *self.xruns_count.lock().unwrap() += 1;
        JackControl::Continue
    }
}

fn open_test_client(name: &str) -> Client {
    Client::open(name, client_options::NO_START_SERVER).unwrap().0
}

fn active_test_client(name: &str) -> (ActiveClient<Counter>) {
    let c = open_test_client(name);
    let ac = c.activate(Counter::default()).unwrap();
    ac
}

#[test]
fn client_cback_calls_thread_init() {
    let ac = active_test_client("client_cback_calls_thread_init");
    let counter = ac.deactivate().unwrap().1;
    // IDK why this isn't 1.
    assert!(*counter.thread_init_count.lock().unwrap() > 0);
}

#[test]
fn client_cback_calls_process() {
    let ac = active_test_client("client_cback_calls_process");
    let counter = ac.deactivate().unwrap().1;
    assert!(*counter.frames_processed.lock().unwrap() > 0);
}

#[test]
fn client_cback_calls_buffer_size() {
    let ac = active_test_client("client_cback_calls_process");
    let initial = ac.buffer_size();
    let second = initial / 2;
    let third = second / 2;
    ac.set_buffer_size(second).unwrap();
    ac.set_buffer_size(third).unwrap();
    ac.set_buffer_size(initial).unwrap();
    let counter = ac.deactivate().unwrap().1;
    assert_eq!(*counter.buffer_size_change_history.lock().unwrap(),
               vec![initial, second, third, initial]);
}

#[test]
fn client_cback_calls_after_client_registered() {
    let ac = active_test_client("client_cback_cacr");
    let _other_client = open_test_client("client_cback_cacr_other");
    let counter = ac.deactivate().unwrap().1;
    assert_eq!(*counter.registered_client_history.lock().unwrap(),
               vec!["client_cback_cacr_other"]);
    assert!(counter.unregistered_client_history.lock().unwrap().is_empty());
}

#[test]
fn client_cback_calls_after_client_unregistered() {
    let ac = active_test_client("client_cback_cacu");
    let other_client = open_test_client("client_cback_cacu_other");
    drop(other_client);
    let counter = ac.deactivate().unwrap().1;
    assert_eq!(*counter.registered_client_history.lock().unwrap(),
               vec!["client_cback_cacu_other"],
               "wrong clients detected as registered");
    assert_eq!(*counter.unregistered_client_history.lock().unwrap(),
               vec!["client_cback_cacu_other"],
               "wrong clients detected as unregistered");
}

#[test]
fn client_cback_doesnt_call_port_registered_when_no_ports() {
    let ac = active_test_client("client_cback_dcprwnp");
    let counter = ac.deactivate().unwrap().1;
    assert!(counter.port_register_history.lock().unwrap().is_empty());
    assert!(counter.port_unregister_history.lock().unwrap().is_empty());
}

#[test]
fn client_cback_reports_xruns() {
    let c = open_test_client("client_cback_reports_xruns");
    let mut counter = Counter::default();
    counter.induce_xruns = true;
    let ac = c.activate(counter).unwrap();
    let counter = ac.deactivate().unwrap().1;
    assert!(*counter.xruns_count.lock().unwrap() > 0,
            "No xruns encountered.");
}

#[test]
fn client_cback_calls_port_registered() {
    let mut ac = active_test_client("client_cback_cpr");
    let _pa = ac.register_port("pa", AudioInSpec::default()).unwrap();
    let _pb = ac.register_port("pb", AudioInSpec::default()).unwrap();
    let counter = ac.deactivate().unwrap().1;
    assert_eq!(counter.port_register_history.lock().unwrap().len(),
               2,
               "Did not detect port registrations.");
    assert!(counter.port_unregister_history.lock().unwrap().is_empty(),
            "Detected false port deregistrations.");
}

#[test]
fn client_cback_calls_port_unregistered() {
    let mut ac = active_test_client("client_cback_cpr");
    let _pa = ac.register_port("pa", AudioInSpec::default()).unwrap();
    let _pb = ac.register_port("pb", AudioInSpec::default()).unwrap();
    _pa.unregister().unwrap();
    _pb.unregister().unwrap();
    let counter = ac.deactivate().unwrap().1;
    assert_eq!(counter.port_register_history.lock().unwrap().len(),
               2,
               "Did not detect port registrations.");
    assert_eq!(counter.port_unregister_history.lock().unwrap().len(),
               2,
               "Did not detect port deregistrations.");
}
