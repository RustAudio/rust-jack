use super::super::*;
use std::sync::mpsc::channel;

fn open_test_client(name: &str) -> Client {
    default_sleep();
    Client::open(name, client_options::NO_START_SERVER).unwrap().0
}

#[test]
fn port_audio_can_read_write() {
    let mut c = open_test_client("port_audio_crw");
    let in_a = c.register_port("ia", AudioInSpec::default()).unwrap();
    let in_b = c.register_port("ib", AudioInSpec::default()).unwrap();
    let mut out_a = c.register_port("oa", AudioOutSpec::default()).unwrap();
    let mut out_b = c.register_port("ob", AudioOutSpec::default()).unwrap();
    let (signal_succeed, did_succeed) = channel();
    let process_callback = move |ps: &ProcessScope| -> JackControl {
        let exp_a = 0.31244352;
        let exp_b = -0.61212;
        let in_a = AudioInPort::new(&in_a, ps);
        let in_b = AudioInPort::new(&in_b, ps);
        let mut out_a = AudioOutPort::new(&mut out_a, ps);
        let mut out_b = AudioOutPort::new(&mut out_b, ps);
        for v in out_a.iter_mut() {
            *v = exp_a;
        }
        for v in out_b.iter_mut() {
            *v = exp_b;
        }
        if in_a.iter().all(|v| *v == exp_a) && in_b.iter().all(|v| *v == exp_b) {
            let s = signal_succeed.clone();
            let _ = s.send(true);
        }
        JackControl::Continue
    };
    let ac = c.activate(process_callback).unwrap();
    default_longer_sleep();
    ac.connect_ports_by_name("port_audio_crw:oa", "port_audio_crw:ia")
        .unwrap();
    ac.connect_ports_by_name("port_audio_crw:ob", "port_audio_crw:ib")
        .unwrap();
    default_longer_sleep();
    assert!(did_succeed.iter().any(|b| b),
            "input port does not have expected data");
    ac.deactivate().unwrap();
}
