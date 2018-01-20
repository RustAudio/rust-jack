use std::{mem, slice};

use jack_sys as j;
use libc;

use client::ProcessScope;
use jack_enums::Error;
use port::{Port, PortSpec};
use port::port_flags::{PortFlags, IS_INPUT, IS_OUTPUT};
use primitive_types as pt;

/// Contains 8bit raw midi information along with a timestamp relative to the
/// process cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawMidi<'a> {
    /// The amount of time passed, in frames, relative to the start of the
    /// process cycle
    pub time: pt::Frames,

    /// Midi data
    pub bytes: &'a [u8],
}

impl<'a> Default for RawMidi<'a> {
    fn default() -> Self {
        RawMidi {
            time: 0,
            bytes: &[],
        }
    }
}

/// `MidiInSpec` implements the `PortSpec` trait, which defines an endpoint for
/// JACK. In this case,
/// it defines midi input.
#[derive(Copy, Clone, Debug, Default)]
pub struct MidiInSpec;

/// `MidiOutSpec` implements the `PortSpec` trait, which defines an endpoint
/// for JACK. In this case,
/// it defines a midi output.
#[derive(Copy, Clone, Debug, Default)]
pub struct MidiOutSpec;

unsafe impl PortSpec for MidiInSpec {
    fn jack_port_type(&self) -> &'static str {
        j::RAW_MIDI_TYPE
    }

    fn jack_flags(&self) -> PortFlags {
        IS_INPUT
    }

    fn jack_buffer_size(&self) -> libc::c_ulong {
        // Not needed for built in types according to JACK api
        0
    }
}

unsafe impl PortSpec for MidiOutSpec {
    fn jack_port_type(&self) -> &'static str {
        j::RAW_MIDI_TYPE
    }

    fn jack_flags(&self) -> PortFlags {
        IS_OUTPUT
    }

    fn jack_buffer_size(&self) -> libc::c_ulong {
        // Not needed for built in types according to JACK api
        0
    }
}

/// Safely and thinly wrap a `Port<MidiInPort>`.
#[derive(Debug, Clone)]
pub struct MidiInPort<'a> {
    _port: &'a Port<MidiInSpec>,
    buffer_ptr: *mut ::libc::c_void,
}

impl<'a> MidiInPort<'a> {
    /// Wrap a `Port<MidiInSpec>` within a process scope of a client that
    /// registered the
    /// port. Panics if the port does not belong to the client that created the
    /// process.
    pub fn new(port: &'a Port<MidiInSpec>, ps: &'a ProcessScope) -> Self {
        assert_eq!(port.client_ptr(), ps.client_ptr());
        let buffer_ptr = unsafe { port.buffer(ps.n_frames()) };
        MidiInPort {
            _port: port,
            buffer_ptr: buffer_ptr,
        }
    }

    pub fn nth(&self, n: usize) -> Option<RawMidi> {
        let mut ev: j::jack_midi_event_t = unsafe { mem::uninitialized() };
        let res = unsafe { j::jack_midi_event_get(&mut ev, self.buffer_ptr, n as libc::uint32_t) };
        if res != 0 {
            return None;
        }
        let bytes_slice: &[u8] = unsafe { slice::from_raw_parts(ev.buffer as *const u8, ev.size) };
        Some(RawMidi {
            time: ev.time,
            bytes: bytes_slice,
        })
    }

    pub fn len(&self) -> usize {
        if self.buffer_ptr.is_null() {
            return 0;
        };
        let n = unsafe { j::jack_midi_get_event_count(self.buffer_ptr) };
        n as usize
    }

    pub fn iter(&'a self) -> MidiIter {
        MidiIter {
            port: &self,
            index: 0,
        }
    }
}

/// Safely and thinly wrap a `Port<MidiInPort>`.
#[derive(Debug)]
pub struct MidiOutPort<'a> {
    _port: &'a mut Port<MidiOutSpec>,
    buffer_ptr: *mut ::libc::c_void,
}

impl<'a> MidiOutPort<'a> {
    /// Wrap a `Port<MidiInSpec>` within a process scope of a client that
    /// registered the
    /// port. Panics if the port does not belong to the client that created the
    /// process.
    ///
    /// The data in the port is cleared.
    pub fn new(port: &'a mut Port<MidiOutSpec>, ps: &'a ProcessScope) -> Self {
        assert_eq!(port.client_ptr(), ps.client_ptr());
        let buffer_ptr = unsafe { port.buffer(ps.n_frames()) };
        unsafe { j::jack_midi_clear_buffer(buffer_ptr) };
        MidiOutPort {
            _port: port,
            buffer_ptr: buffer_ptr,
        }
    }

    /// Write an event into an event port buffer.
    ///
    /// Clients must write normalised MIDI data to the port - no running status
    /// and no (1-byte)
    /// realtime messages interspersed with other messagse (realtime messages
    /// are fine when they
    /// occur on their own, like other messages).
    pub fn write(&mut self, message: &RawMidi) -> Result<(), Error> {
        let ev = j::jack_midi_event_t {
            time: message.time,
            size: message.bytes.len(),
            buffer: message.bytes.as_ptr() as *mut u8,
        };
        let res = unsafe { j::jack_midi_event_write(self.buffer_ptr, ev.time, ev.buffer, ev.size) };
        match res {
            0 => Ok(()),
            _ => Err(Error::NotEnoughSpace),
        }
    }

    /// Get the number of events that could not be written to port_buffer.
    ///
    /// If the return value is greater than 0, than the buffer is full.
    /// Currently, the only way this
    /// can happen is if events are lost on port mixdown.
    pub fn lost_count(&self) -> usize {
        let n = unsafe { j::jack_midi_get_lost_event_count(self.buffer_ptr) };
        n as usize
    }

    /// Get the size of the largest event that can be stored by the port.
    ///
    /// This function returns the current space available, taking into account
    /// events already stored
    /// in the port.
    pub fn max_event_size(&self) -> usize {
        let n = unsafe { j::jack_midi_max_event_size(self.buffer_ptr) };
        n as usize
    }
}

/// Iterate through Midi Messages within a `MidiInPort`.
#[derive(Debug, Clone)]
pub struct MidiIter<'a> {
    port: &'a MidiInPort<'a>,
    index: usize,
}

impl<'a> MidiIter<'a> {
    /// Return the next element without advancing the iterator.
    pub fn peek(&self) -> Option<RawMidi<'a>> {
        self.port.nth(self.index)
    }

    /// Return the next element only if the message passes the predicate.
    pub fn next_if<P>(&mut self, predicate: P) -> Option<RawMidi<'a>>
    where
        P: FnOnce(RawMidi) -> bool,
    {
        if self.peek().map(predicate).unwrap_or(false) {
            self.next()
        } else {
            None
        }
    }
}

impl<'a> Iterator for MidiIter<'a> {
    type Item = RawMidi<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.peek();
        self.index += 1;
        ret
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.port.len() - self.index;
        (count, Some(count))
    }

    fn count(self) -> usize {
        self.size_hint().0
    }

    fn last(self) -> Option<Self::Item> {
        let n = self.port.len();
        if n == 0 {
            None
        } else {
            self.port.nth(n - 1)
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.index += n;
        self.next()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use client::AsyncClient;
    use client::Client;
    use client::ClosureProcessHandler;
    use client::ProcessHandler;
    use client::client_options;
    use jack_enums::Control;
    use primitive_types::Frames;
    use std::{thread, time};
    use std::iter::Iterator;
    use std::sync::Mutex;
    use std::sync::mpsc::channel;

    fn open_test_client(name: &str) -> Client {
        Client::new(name, client_options::NO_START_SERVER)
            .unwrap()
            .0
    }

    struct Connector {
        src: String,
        dst: String,
    }

    impl Connector {
        fn connect(&self, c: &Client) {
            c.connect_ports_by_name(&self.src, &self.dst).unwrap();
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct OwnedRawMidi {
        time: Frames,
        bytes: Vec<u8>,
    }

    impl OwnedRawMidi {
        fn new(m: &RawMidi) -> OwnedRawMidi {
            OwnedRawMidi {
                time: m.time,
                bytes: m.bytes.to_vec(),
            }
        }

        fn unowned<'a>(&'a self) -> RawMidi<'a> {
            RawMidi {
                time: self.time,
                bytes: &self.bytes,
            }
        }
    }

    struct IterTest<F: Send + Fn(MidiInPort) -> Vec<OwnedRawMidi>> {
        stream: Vec<OwnedRawMidi>,
        collected: Vec<OwnedRawMidi>,
        collector: F,
        midi_in: Port<MidiInSpec>,
        midi_out: Port<MidiOutSpec>,
    }

    impl<F: Send + Fn(MidiInPort) -> Vec<OwnedRawMidi>> IterTest<F> {
        fn new(client: &Client, stream: Vec<OwnedRawMidi>, collector: F) -> IterTest<F> {
            IterTest {
                stream: stream,
                collected: Vec::new(),
                collector: collector,
                midi_in: client.register_port("in", MidiInSpec::default()).unwrap(),
                midi_out: client.register_port("out", MidiOutSpec::default()).unwrap(),
            }
        }

        fn connector(&self) -> Connector {
            Connector {
                src: self.midi_out.name().to_string(),
                dst: self.midi_in.name().to_string(),
            }
        }
    }

    impl<F: Send + Fn(MidiInPort) -> Vec<OwnedRawMidi>> ProcessHandler for IterTest<F> {
        fn process(&mut self, _: &Client, ps: &ProcessScope) -> Control {
            let (midi_in, mut midi_out) = (
                MidiInPort::new(&self.midi_in, ps),
                MidiOutPort::new(&mut self.midi_out, ps),
            );
            // Write to output.
            for m in self.stream.iter() {
                midi_out.write(&m.unowned()).unwrap();
            }
            // Collect in input.
            if self.collected.is_empty() {
                self.collected = (self.collector)(midi_in);
            }
            Control::Continue
        }
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
        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
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
            if in_a.len() == 1 && in_a.iter().all(|m| m == exp_a) && in_b.iter().all(|m| m == exp_b)
            {
                let _ = signal_succeed.send(true).unwrap();
            }
            Control::Continue
        };

        // activate
        let ac = AsyncClient::new(c, (), ClosureProcessHandler::new(process_callback)).unwrap();

        // connect ports to each other
        ac.as_client()
            .connect_ports_by_name("port_midi_crw:oa", "port_midi_crw:ia")
            .unwrap();
        ac.as_client()
            .connect_ports_by_name("port_midi_crw:ob", "port_midi_crw:ib")
            .unwrap();

        // check correctness
        thread::sleep(time::Duration::from_millis(400));
        assert!(
            did_succeed.iter().any(|b| b),
            "input port does not have expected data"
        );
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
        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
            let out_p = MidiOutPort::new(&mut out_p, ps);
            *PMCGMES_MAX_EVENT_SIZE.lock().unwrap() = out_p.max_event_size();
            Control::Continue
        };

        // activate
        let ac = AsyncClient::new(c, (), ClosureProcessHandler::new(process_callback)).unwrap();

        // check correctness
        assert!(*PMCGMES_MAX_EVENT_SIZE.lock().unwrap() > 0);
        ac.deactivate().unwrap();
    }

    lazy_static! {
        static ref PMCEMES_WRITE_RESULT: Mutex<Result<(), Error>> = Mutex::new(Ok(()));
    }

    #[test]
    fn port_midi_cant_exceed_max_event_size() {
        // open clients and ports
        let c = open_test_client("port_midi_cglc");
        let mut out_p = c.register_port("op", MidiOutSpec::default()).unwrap();

        // set callback routine
        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
            let mut out_p = MidiOutPort::new(&mut out_p, ps);
            *PMCGMES_MAX_EVENT_SIZE.lock().unwrap() = out_p.max_event_size();

            let bytes: Vec<u8> = (0..out_p.max_event_size() + 1).map(|_| 0).collect();
            let msg = RawMidi {
                time: 0,
                bytes: &bytes,
            };

            *PMCEMES_WRITE_RESULT.lock().unwrap() = out_p.write(&msg);

            Control::Continue
        };

        // activate
        let ac = AsyncClient::new(c, (), ClosureProcessHandler::new(process_callback)).unwrap();

        // check correctness
        assert_eq!(
            *PMCEMES_WRITE_RESULT.lock().unwrap(),
            Err(Error::NotEnoughSpace)
        );
        ac.deactivate().unwrap();
    }

    lazy_static! {
        static ref PMI_NEXT: Mutex<Option<(Frames, Vec<u8>)>> = Mutex::default();
        static ref PMI_SIZE_HINT: Mutex<(usize, Option<usize>)> = Mutex::new((0, None));
        static ref PMI_COUNT: Mutex<usize> = Mutex::default();
        static ref PMI_LAST: Mutex<Option<(Frames, Vec<u8>)>> = Mutex::default();
        static ref PMI_THIRD: Mutex<Option<(Frames, Vec<u8>)>> = Mutex::default();
    }

    #[test]
    fn port_midi_iter() {
        // open clients and ports
        let c = open_test_client("port_midi_iter");
        let in_p = c.register_port("ip", MidiInSpec::default()).unwrap();
        let mut out_p = c.register_port("op", MidiOutSpec::default()).unwrap();

        // set callback routine
        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
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

            Control::Continue
        };

        // run
        let ac = AsyncClient::new(c, (), ClosureProcessHandler::new(process_callback)).unwrap();
        ac.as_client()
            .connect_ports_by_name("port_midi_iter:op", "port_midi_iter:ip")
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

    #[test]
    fn port_midi_iter_next_if() {
        let c = open_test_client("pmi_nib");
        let stream = vec![
            OwnedRawMidi {
                time: 0,
                bytes: vec![1],
            },
            OwnedRawMidi {
                time: 10,
                bytes: vec![3, 4, 5],
            },
            OwnedRawMidi {
                time: 11,
                bytes: vec![6],
            },
            OwnedRawMidi {
                time: 12,
                bytes: vec![7, 8],
            },
        ];
        let collect = |midi_in: MidiInPort| {
            let mut collected = Vec::with_capacity(midi_in.len());
            let mut iter = midi_in.iter();
            while let Some(m) = iter.next_if(|m| m.time < 11) {
                collected.push(OwnedRawMidi::new(&m));
            }
            collected
        };
        let processor = IterTest::new(&c, stream.clone(), collect);
        let connector = processor.connector();

        let ac = AsyncClient::new(c, (), processor).unwrap();
        connector.connect(ac.as_client());
        thread::sleep(time::Duration::from_millis(200));

        let (_, _, processor) = ac.deactivate().unwrap();
        let expected: &[OwnedRawMidi] = &stream[0..2];
        let got: &[OwnedRawMidi] = &processor.collected;
        assert_eq!(expected, got);
    }
}
