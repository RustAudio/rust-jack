use std::sync::atomic::{AtomicUsize, Ordering};
use std::{mem, ptr, thread, time};

use super::*;
use AudioIn;
use Client;
use Control;
use Frames;
use LatencyType;
use NotificationHandler;
use PortId;
use ProcessHandler;

#[derive(Debug, Default)]
pub struct Counter {
    pub process_return_val: Control,
    pub induce_xruns: bool,
    pub thread_init_count: AtomicUsize,
    pub frames_processed: usize,
    pub buffer_size_change_history: Vec<Frames>,
    pub registered_client_history: Vec<String>,
    pub unregistered_client_history: Vec<String>,
    pub port_register_history: Vec<PortId>,
    pub port_unregister_history: Vec<PortId>,
    pub xruns_count: usize,
    pub last_frame_time: Frames,
    pub frames_since_cycle_start: Frames,
}

impl NotificationHandler for Counter {
    fn thread_init(&self, _: &Client) {
        self.thread_init_count.fetch_add(1, Ordering::Relaxed);
    }

    fn buffer_size(&mut self, _: &Client, size: Frames) -> Control {
        self.buffer_size_change_history.push(size);
        Control::Continue
    }

    fn client_registration(&mut self, _: &Client, name: &str, is_registered: bool) {
        if is_registered {
            self.registered_client_history.push(name.to_string())
        } else {
            self.unregistered_client_history.push(name.to_string())
        }
    }

    fn port_registration(&mut self, _: &Client, pid: PortId, is_registered: bool) {
        if is_registered {
            self.port_register_history.push(pid)
        } else {
            self.port_unregister_history.push(pid)
        }
    }

    fn xrun(&mut self, _: &Client) -> Control {
        self.xruns_count += 1;
        Control::Continue
    }
}

impl ProcessHandler for Counter {
    fn process(&mut self, _: &Client, ps: &ProcessScope) -> Control {
        self.frames_processed += ps.n_frames() as usize;
        self.last_frame_time = ps.last_frame_time();
        self.frames_since_cycle_start = ps.frames_since_cycle_start();
        let _cycle_times = ps.cycle_times();
        if self.induce_xruns {
            thread::sleep(time::Duration::from_millis(400));
        }
        Control::Continue
    }
}

fn open_test_client(name: &str) -> Client {
    Client::new(name, ClientOptions::NO_START_SERVER).unwrap().0
}

fn active_test_client(name: &str) -> (AsyncClient<Counter, Counter>) {
    let c = open_test_client(name);
    c.activate_async(Counter::default(), Counter::default())
        .unwrap()
}

#[test]
fn client_cback_has_proper_default_callbacks() {
    // defaults shouldn't care about these params
    let wc = unsafe { Client::from_raw(ptr::null_mut()) };
    let ps = unsafe { ProcessScope::from_raw(0, ptr::null_mut()) };
    // check each callbacks
    assert_eq!(().thread_init(&wc), ());
    assert_eq!(
        ().shutdown(client_status::ClientStatus::empty(), "mock"),
        ()
    );
    assert_eq!(().process(&wc, &ps), Control::Continue);
    assert_eq!(().freewheel(&wc, true), ());
    assert_eq!(().freewheel(&wc, false), ());
    assert_eq!(().buffer_size(&wc, 0), Control::Continue);
    assert_eq!(().sample_rate(&wc, 0), Control::Continue);
    assert_eq!(().client_registration(&wc, "mock", true), ());
    assert_eq!(().client_registration(&wc, "mock", false), ());
    assert_eq!(().port_registration(&wc, 0, true), ());
    assert_eq!(().port_registration(&wc, 0, false), ());
    assert_eq!(
        ().port_rename(&wc, 0, "old_mock", "new_mock"),
        Control::Continue
    );
    assert_eq!(().ports_connected(&wc, 0, 1, true), ());
    assert_eq!(().ports_connected(&wc, 2, 3, false), ());
    assert_eq!(().graph_reorder(&wc), Control::Continue);
    assert_eq!(().xrun(&wc), Control::Continue);
    assert_eq!(().latency(&wc, LatencyType::Capture), ());
    assert_eq!(().latency(&wc, LatencyType::Playback), ());

    mem::forget(wc);
    mem::forget(ps);
}

#[test]
fn client_cback_calls_thread_init() {
    let ac = active_test_client("client_cback_calls_thread_init");
    let counter = ac.deactivate().unwrap().1;
    // IDK why this isn't 1, even with a single thread.
    assert!(counter.thread_init_count.load(Ordering::Relaxed) > 0);
}

#[test]
fn client_cback_calls_process() {
    let ac = active_test_client("client_cback_calls_process");
    let counter = ac.deactivate().unwrap().2;
    assert!(counter.frames_processed > 0);
    assert!(counter.last_frame_time > 0);
    assert!(counter.frames_since_cycle_start > 0);
}

#[test]
fn client_cback_calls_buffer_size() {
    let ac = active_test_client("client_cback_calls_process");
    let initial = ac.as_client().buffer_size();
    let second = initial / 2;
    let third = second / 2;
    ac.as_client().set_buffer_size(second).unwrap();
    ac.as_client().set_buffer_size(third).unwrap();
    ac.as_client().set_buffer_size(initial).unwrap();
    let counter = ac.deactivate().unwrap().1;
    let mut history_iter = counter.buffer_size_change_history.iter().cloned();
    assert_eq!(history_iter.find(|&s| s == initial), Some(initial));
    assert_eq!(history_iter.find(|&s| s == second), Some(second));
    assert_eq!(history_iter.find(|&s| s == third), Some(third));
    assert_eq!(history_iter.find(|&s| s == initial), Some(initial));
}

#[test]
fn client_cback_calls_after_client_registered() {
    let ac = active_test_client("client_cback_cacr");
    let _other_client = open_test_client("client_cback_cacr_other");
    let counter = ac.deactivate().unwrap().1;
    assert!(counter
        .registered_client_history
        .contains(&"client_cback_cacr_other".to_string(),));
    assert!(!counter
        .unregistered_client_history
        .contains(&"client_cback_cacr_other".to_string(),));
}

#[test]
fn client_cback_calls_after_client_unregistered() {
    let ac = active_test_client("client_cback_cacu");
    let other_client = open_test_client("client_cback_cacu_other");
    drop(other_client);
    let counter = ac.deactivate().unwrap().1;
    assert!(counter
        .registered_client_history
        .contains(&"client_cback_cacu_other".to_string(),));
    assert!(counter
        .unregistered_client_history
        .contains(&"client_cback_cacu_other".to_string(),));
}

#[test]
fn client_cback_reports_xruns() {
    let c = open_test_client("client_cback_reports_xruns");
    let mut counter = Counter::default();
    counter.induce_xruns = true;
    let ac = c.activate_async(Counter::default(), counter).unwrap();
    let counter = ac.deactivate().unwrap().1;
    assert!(counter.xruns_count > 0, "No xruns encountered.");
}

#[test]
fn client_cback_calls_port_registered() {
    let ac = active_test_client("client_cback_cpr");
    let _pa = ac
        .as_client()
        .register_port("pa", AudioIn::default())
        .unwrap();
    let _pb = ac
        .as_client()
        .register_port("pb", AudioIn::default())
        .unwrap();
    let counter = ac.deactivate().unwrap().1;
    assert_eq!(
        counter.port_register_history.len(),
        2,
        "Did not detect port registrations."
    );
    assert!(
        counter.port_unregister_history.is_empty(),
        "Detected false port deregistrations."
    );
}

#[test]
fn client_cback_calls_port_unregistered() {
    let ac = active_test_client("client_cback_cpr");
    let pa = ac
        .as_client()
        .register_port("pa", AudioIn::default())
        .unwrap();
    let pb = ac
        .as_client()
        .register_port("pb", AudioIn::default())
        .unwrap();
    ac.as_client().unregister_port(pa).unwrap();
    ac.as_client().unregister_port(pb).unwrap();
    let counter = ac.deactivate().unwrap().1;
    assert!(
        counter.port_register_history.len() >= 2,
        "Did not detect port registrations."
    );
    assert!(
        counter.port_unregister_history.len() >= 2,
        "Did not detect port deregistrations."
    );
}
