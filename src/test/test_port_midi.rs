use super::super::*;
use std::sync::mpsc::channel;

fn open_test_client(name: &str) -> Client {
    default_sleep();
    Client::open(name, client_options::NO_START_SERVER).unwrap().0
}

#[test]
fn port_midi_can_read_write() {
    let mut c = open_test_client("port_audio_crw");
    let in_a = c.register_port("ia", MidiInSpec::default()).unwrap();
    let in_b = c.register_port("ib", MidiInSpec::default()).unwrap();
    let mut out_a = c.register_port("oa", MidiOutSpec::default()).unwrap();
    let mut out_b = c.register_port("ob", MidiOutSpec::default()).unwrap();
    let (signal_succeed, did_succeed) = channel();
    let process_callback = move |ps: &ProcessScope| -> JackControl {
        let exp_a = RawMidi {
            time: 0,
            bytes: &[0b10010000, 0b01000000],
        };
        let exp_b = RawMidi {
            time: 64,
            bytes: &[0b10000000, 0b01000000],
        };
        let in_a = MidiInPort::new(&in_a, ps);
        let in_b = MidiInPort::new(&in_b, ps);
        let mut out_a = MidiOutPort::new(&mut out_a, ps);
        let mut out_b = MidiOutPort::new(&mut out_b, ps);
        out_a.write(&exp_a).unwrap();
        out_b.write(&exp_b).unwrap();
        if in_a.iter().all(|m| m == exp_a) && in_b.iter().all(|m| m == exp_b) {
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
