use prelude::*;
use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::{thread, time};

fn open_test_client(name: &str) -> Client {
    Client::open(name, client_options::NO_START_SERVER).unwrap().0
}

#[test]
fn port_midi_can_read_write() {
    // open clients and ports
    let mut c = open_test_client("port_audio_crw");
    let in_a = c.register_port("ia", MidiInSpec::default()).unwrap();
    let in_b = c.register_port("ib", MidiInSpec::default()).unwrap();
    let mut out_a = c.register_port("oa", MidiOutSpec::default()).unwrap();
    let mut out_b = c.register_port("ob", MidiOutSpec::default()).unwrap();

    // set callback routine
    let (signal_succeed, did_succeed) = channel();
    let process_callback = move |_: &WeakClient, ps: &ProcessScope| -> JackControl {
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
        if in_a.len() == 1 && in_a.iter().all(|m| m == exp_a) && in_b.iter().all(|m| m == exp_b) {
            let _ = signal_succeed.send(true).unwrap();
        }
        JackControl::Continue
    };

    // activate
    let ac = c.activate(ProcessHandler::new(process_callback)).unwrap();

    // connect ports to each other
    ac.connect_ports_by_name("port_audio_crw:oa", "port_audio_crw:ia")
        .unwrap();
    ac.connect_ports_by_name("port_audio_crw:ob", "port_audio_crw:ib")
        .unwrap();

    // check correctness
    thread::sleep(time::Duration::from_millis(400));
    assert!(did_succeed.iter().any(|b| b),
            "input port does not have expected data");
    ac.deactivate().unwrap();
}

lazy_static! {
    static ref PMCGMES_MAX_EVENT_SIZE: Mutex<usize> = Mutex::new(0);
}

#[test]
fn port_midi_can_get_max_event_size() {
    // open clients and ports
    let mut c = open_test_client("port_audio_cglc");
    let mut out_p = c.register_port("op", MidiOutSpec::default()).unwrap();

    // set callback routine
    let process_callback = move |_: &WeakClient, ps: &ProcessScope| -> JackControl {
        let out_p = MidiOutPort::new(&mut out_p, ps);
        *PMCGMES_MAX_EVENT_SIZE.lock().unwrap() = out_p.max_event_size();
        JackControl::Continue
    };

    // activate
    let ac = c.activate(ProcessHandler::new(process_callback)).unwrap();

    // check correctness
    assert!(*PMCGMES_MAX_EVENT_SIZE.lock().unwrap() > 0);
    ac.deactivate().unwrap();
}


lazy_static! {
    static ref PMCEMES_DID_EXCEED: Mutex<Option<JackErr>> = Mutex::new(None);
}

#[test]
fn port_midi_cant_execeed_max_event_size() {
    // open clients and ports
    let mut c = open_test_client("port_audio_cglc");
    let mut out_p = c.register_port("op", MidiOutSpec::default()).unwrap();

    // set callback routine
    let process_callback = move |_: &WeakClient, ps: &ProcessScope| -> JackControl {
        let mut out_p = MidiOutPort::new(&mut out_p, ps);
        *PMCGMES_MAX_EVENT_SIZE.lock().unwrap() = out_p.max_event_size();

        let bytes: Vec<u8> = (0..out_p.max_event_size() + 1).map(|_| 0).collect();
        let msg = RawMidi {
            time: 0,
            bytes: &bytes,
        };

        *PMCEMES_DID_EXCEED.lock().unwrap() = out_p.write(&msg).err();

        JackControl::Continue
    };

    // activate
    let ac = c.activate(ProcessHandler::new(process_callback)).unwrap();

    // check correctness
    assert_eq!(*PMCEMES_DID_EXCEED.lock().unwrap(),
               Some(JackErr::NotEnoughSpace));
    ac.deactivate().unwrap();
}
