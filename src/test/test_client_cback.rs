use super::super::*;

#[derive(Debug, Default)]
pub struct Counter {
    pub thread_init_count: usize,
    pub frames_processed: usize,
    pub buffer_size_change_history: Vec<JackFrames>,
    pub registered_client_history: Vec<String>,
    pub unregistered_client_history: Vec<String>,
    pub port_register_history: Vec<JackPortId>,
    pub port_unregister_history: Vec<JackPortId>,
}

impl JackHandler for Counter {
    fn thread_init(&mut self) {
        self.thread_init_count += 1;
    }

    fn process(&mut self, ps: &ProcessScope) -> JackControl {
        self.frames_processed += ps.n_frames() as usize;
        JackControl::Continue
    }

    fn buffer_size(&mut self, size: JackFrames) -> JackControl {
        self.buffer_size_change_history.push(size);
        JackControl::Continue
    }

    fn client_registration(&mut self, name: &str, is_registered: bool) {
        match is_registered {
            true => self.registered_client_history.push(name.to_string()),
            false => self.unregistered_client_history.push(name.to_string()),
        }
    }

    fn port_registration(&mut self, pid: JackPortId, is_registered: bool) {
        match is_registered {
            true => self.port_register_history.push(pid),
            false => self.port_unregister_history.push(pid),
        }
    }
}

fn open_test_client(name: &str) -> Client {
    default_sleep();
    Client::open(name, client_options::NO_START_SERVER).unwrap().0
}

fn active_test_client(name: &str) -> (ActiveClient<Counter>) {
    let c = open_test_client(name);
    let ac = c.activate(Counter::default()).unwrap();
    default_longer_sleep();
    ac
}

#[test]
fn client_cback_calls_thread_init() {
    let ac = active_test_client("client_cback_calls_thread_init");
    let counter = ac.deactivate().unwrap().1;
    // IDK why this isn't 1.
    assert!(counter.thread_init_count > 0);
}

#[test]
fn client_cback_calls_process() {
    let ac = active_test_client("client_cback_calls_process");
    let counter = ac.deactivate().unwrap().1;
    assert!(counter.frames_processed > 0);
}

#[test]
fn client_cback_calls_buffer_size() {
    let ac = active_test_client("client_cback_calls_process");
    let initial = ac.buffer_size();
    let second = 2 * initial;
    let third = 2 * second;
    ac.set_buffer_size(second).unwrap();
    ac.set_buffer_size(third).unwrap();
    ac.set_buffer_size(initial).unwrap();
    let counter = ac.deactivate().unwrap().1;
    assert_eq!(counter.buffer_size_change_history,
               vec![initial, second, third, initial]);
}

#[test]
fn client_cback_calls_after_client_registered() {
    let ac = active_test_client("client_cback_cacr");
    let _other_client = open_test_client("client_cback_cacr_other");
    let counter = ac.deactivate().unwrap().1;
    assert_eq!(counter.registered_client_history,
               vec!["client_cback_cacr_other"]);
    assert_eq!(counter.unregistered_client_history, Vec::<String>::new());
}

#[test]
fn client_cback_calls_after_client_uregistered() {
    let ac = active_test_client("client_cback_cacu");
    let other_client = open_test_client("client_cback_cacu_other");
    drop(other_client);
    let counter = ac.deactivate().unwrap().1;
    assert_eq!(counter.registered_client_history,
               vec!["client_cback_cacu_other"]);
    assert_eq!(counter.unregistered_client_history,
               vec!["client_cback_cacu_other"]);
}

#[test]
fn client_cback_doesnt_call_port_registered_when_no_ports() {
    let ac = active_test_client("client_cback_dcprwnp");
    let counter = ac.deactivate().unwrap().1;
    assert_eq!(counter.port_register_history, Vec::<JackPortId>::new());
    assert_eq!(counter.port_unregister_history, Vec::<JackPortId>::new());
}

// #[test]
// fn client_cback_calls_port_registered() {
//     let ac = active_test_client("client_cback_cpr");
//     let mut other = open_test_client("client_cback_cpr_ports");
//     let _pa = other.register_port("pa", AudioInSpec).unwrap();
//     let _pb = other.register_port("pb", AudioInSpec).unwrap();
//     let counter = ac.deactivate().unwrap().1;
//     assert_eq!(counter.port_register_history.len(), 2);
//     assert_eq!(counter.port_unregister_history.len(), 0);
// }

// #[test]
// fn client_cback_calls_port_unregistered() {
//     let ac = active_test_client("client_cback_cpu");
//     let mut other = open_test_client("client_cback_cpu_ports");
//     other.register_port("pa", AudioInSpec).unwrap().unregister().unwrap();
//     other.register_port("pb", AudioInSpec).unwrap().unregister().unwrap();
//     default_longer_sleep();
//     let counter = ac.deactivate().unwrap().1;
//     assert_eq!(counter.port_register_history.len(), 2);
//     assert_eq!(counter.port_unregister_history.len(), 2);
// }
