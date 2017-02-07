// The MIT License (MIT)
//
// Copyright (c) 2017 Will Medrano (will.s.medrano@gmail.com)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use prelude::*;
use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::{thread, time};
use std::iter::Iterator;

fn open_test_client(name: &str) -> Client {
    Client::open(name, client_options::NO_START_SERVER).unwrap().0
}

#[test]
fn port_midi_can_read_write() {
    // open clients and ports
    let c = open_test_client("port_midi_crw");
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
    ac.connect_ports_by_name("port_midi_crw:oa", "port_midi_crw:ia")
        .unwrap();
    ac.connect_ports_by_name("port_midi_crw:ob", "port_midi_crw:ib")
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
    let c = open_test_client("port_midi_cglc");
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
    static ref PMCEMES_WRITE_RESULT: Mutex<Result<(), JackErr>> = Mutex::new(Ok(()));
}

#[test]
fn port_midi_cant_execeed_max_event_size() {
    // open clients and ports
    let c = open_test_client("port_midi_cglc");
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

        *PMCEMES_WRITE_RESULT.lock().unwrap() = out_p.write(&msg);

        JackControl::Continue
    };

    // activate
    let ac = c.activate(ProcessHandler::new(process_callback)).unwrap();

    // check correctness
    assert_eq!(*PMCEMES_WRITE_RESULT.lock().unwrap(),
               Err(JackErr::NotEnoughSpace));
    ac.deactivate().unwrap();
}

lazy_static! {
    static ref PMI_NEXT: Mutex<Option<(JackFrames, Vec<u8>)>> = Mutex::default();
    static ref PMI_SIZE_HINT: Mutex<(usize, Option<usize>)> = Mutex::new((0, None));
    static ref PMI_COUNT: Mutex<usize> = Mutex::default();
    static ref PMI_LAST: Mutex<Option<(JackFrames, Vec<u8>)>> = Mutex::default();
    static ref PMI_THIRD: Mutex<Option<(JackFrames, Vec<u8>)>> = Mutex::default();
}

#[test]
fn port_midi_has_good_iter() {
    // open clients and ports
    let c = open_test_client("port_midi_has_good_iter");
    let in_p = c.register_port("ip", MidiInSpec::default()).unwrap();
    let mut out_p = c.register_port("op", MidiOutSpec::default()).unwrap();

    // set callback routine
    let process_callback = move |_: &WeakClient, ps: &ProcessScope| -> JackControl {
        let in_p = MidiInPort::new(&in_p, ps);
        let mut out_p = MidiOutPort::new(&mut out_p, ps);

        for i in 10..14 {
            let msg = RawMidi {
                time: i,
                bytes: &[i as u8],
            };
            out_p.write(&msg).ok();
        }

        let rm_to_owned = |m: &RawMidi| (m.time, m.bytes.to_vec());
        *PMI_NEXT.lock().unwrap() = in_p.iter().next().map(|m| rm_to_owned(&m));
        *PMI_SIZE_HINT.lock().unwrap() = in_p.iter().size_hint();
        *PMI_COUNT.lock().unwrap() = in_p.iter().count();
        *PMI_LAST.lock().unwrap() = in_p.iter().last().map(|m| rm_to_owned(&m));
        *PMI_THIRD.lock().unwrap() = in_p.iter().nth(2).map(|m| rm_to_owned(&m));

        JackControl::Continue
    };

    // run
    let ac = c.activate(ProcessHandler::new(process_callback)).unwrap();
    ac.connect_ports_by_name("port_midi_has_good_iter:op", "port_midi_has_good_iter:ip")
        .unwrap();
    thread::sleep(time::Duration::from_millis(200));
    ac.deactivate().unwrap();

    // check correctness
    assert_eq!(*PMI_NEXT.lock().unwrap(), Some((10, [10].to_vec())));
    assert_eq!(*PMI_SIZE_HINT.lock().unwrap(), (4, Some(4)));
    assert_eq!(*PMI_COUNT.lock().unwrap(), 4);
    assert_eq!(*PMI_LAST.lock().unwrap(), Some((13, [13].to_vec())));
    assert_eq!(*PMI_THIRD.lock().unwrap(), Some((12, [12].to_vec())));
}
