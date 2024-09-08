use jack_sys as j;
use std::marker::PhantomData;
use std::{mem, slice};

use crate::{Error, Frames, Port, PortFlags, PortSpec, ProcessScope};

/// Contains 8bit raw midi information along with a timestamp relative to the
/// process cycle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RawMidi<'a> {
    /// The amount of time passed, in frames, relative to the start of the
    /// process cycle
    pub time: Frames,

    /// Midi data
    pub bytes: &'a [u8],
}

/// `MidiIn` implements the `PortSpec` trait, which defines an endpoint for JACK. In this case, it
/// defines midi input.
#[derive(Copy, Clone, Debug, Default)]
pub struct MidiIn;

/// `MidiOut` implements the `PortSpec` trait, which defines an endpoint for JACK. In this case, it
/// defines a midi output.
#[derive(Copy, Clone, Debug, Default)]
pub struct MidiOut;

unsafe impl PortSpec for MidiIn {
    fn jack_port_type(&self) -> &'static str {
        j::RAW_MIDI_TYPE
    }

    fn jack_flags(&self) -> PortFlags {
        PortFlags::IS_INPUT
    }

    fn jack_buffer_size(&self) -> libc::c_ulong {
        // Not needed for built in types according to JACK api
        0
    }
}

impl Port<MidiIn> {
    /// Get an iterator over midi events.
    pub fn iter<'a>(&'a self, ps: &'a ProcessScope) -> MidiIter<'a> {
        assert_eq!(self.client_ptr(), ps.client_ptr());
        MidiIter {
            buffer: unsafe { self.buffer(ps.n_frames()) },
            index: 0,
            _phantom: PhantomData,
        }
    }
}

/// Iterate through Midi Messages within a `Port<MidiIn>`.
#[derive(Debug, Clone)]
pub struct MidiIter<'a> {
    buffer: *mut ::libc::c_void,
    index: usize,
    _phantom: PhantomData<&'a ()>,
}

unsafe impl<'a> Sync for MidiIter<'a> {}

impl<'a> MidiIter<'a> {
    /// Return the next element without advancing the iterator.
    pub fn peek(&self) -> Option<RawMidi<'a>> {
        self.absolute_nth(self.index as u32)
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

    fn absolute_nth(&self, n: u32) -> Option<RawMidi<'a>> {
        let mut ev = mem::MaybeUninit::<j::jack_midi_event_t>::uninit();
        let res = unsafe { j::jack_midi_event_get(ev.as_mut_ptr(), self.buffer, n) };
        if res != 0 {
            return None;
        }
        let ev = unsafe { ev.assume_init() };
        let time = ev.time;
        let bytes: &[u8] = unsafe { slice::from_raw_parts(ev.buffer as *const u8, ev.size) };
        Some(RawMidi { time, bytes })
    }

    fn absolute_len(&self) -> usize {
        if self.buffer.is_null() {
            0
        } else {
            unsafe { j::jack_midi_get_event_count(self.buffer) as usize }
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
        let len = self.absolute_len() - self.index;
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.absolute_len() - self.index
    }

    fn last(self) -> Option<Self::Item> {
        let len = self.absolute_len() as u32;
        if len == 0 || self.index >= len as usize {
            None
        } else {
            self.absolute_nth(len - 1)
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.index += n;
        self.next()
    }
}

unsafe impl PortSpec for MidiOut {
    fn jack_port_type(&self) -> &'static str {
        j::RAW_MIDI_TYPE
    }

    fn jack_flags(&self) -> PortFlags {
        PortFlags::IS_OUTPUT
    }

    fn jack_buffer_size(&self) -> libc::c_ulong {
        // Not needed for built in types according to JACK api
        0
    }
}

impl Port<MidiOut> {
    /// Create a writer that can write midi events to the specified midi port. Calling this function
    /// clears the midi buffer.
    pub fn writer<'a>(&'a mut self, ps: &'a ProcessScope) -> MidiWriter<'a> {
        assert_eq!(self.client_ptr(), ps.client_ptr());
        let buffer = unsafe { self.buffer(ps.n_frames()) };
        unsafe { j::jack_midi_clear_buffer(buffer) };
        MidiWriter {
            buffer,
            _phantom: PhantomData,
        }
    }
}

/// Write midi events to an output midi port.
#[derive(Debug)]
pub struct MidiWriter<'a> {
    buffer: *mut ::libc::c_void,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> MidiWriter<'a> {
    /// Write an event into an event port buffer.
    ///
    /// Clients must write normalised MIDI data to the port - no running status and no (1-byte)
    /// realtime messages interspersed with other messagse (realtime messages are fine when they
    /// occur on their own, like other messages).
    pub fn write(&mut self, message: &RawMidi) -> Result<(), Error> {
        let ev = j::jack_midi_event_t {
            time: message.time,
            size: message.bytes.len(),
            buffer: message.bytes.as_ptr() as *mut u8,
        };
        let res = unsafe { j::jack_midi_event_write(self.buffer, ev.time, ev.buffer, ev.size) };
        match res {
            0 => Ok(()),
            _ => Err(Error::NotEnoughSpace),
        }
    }

    /// Get the number of events that could not be written to port_buffer.
    ///
    /// If the return value is greater than 0, than the buffer is full.  Currently, the only way
    /// this can happen is if events are lost on port mixdown.
    pub fn lost_count(&self) -> usize {
        let n = unsafe { j::jack_midi_get_lost_event_count(self.buffer) };
        n as usize
    }

    /// Get the size of the largest event that can be stored by the port.
    ///
    /// This function returns the current space available, taking into account events already stored
    /// in the port.
    pub fn max_event_size(&self) -> usize {
        unsafe { j::jack_midi_max_event_size(self.buffer) }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::client::Client;
    use crate::client::ClosureProcessHandler;
    use crate::client::ProcessHandler;
    use crate::jack_enums::Control;
    use crate::primitive_types::Frames;
    use crate::ClientOptions;
    use crossbeam_channel::bounded;
    use lazy_static::lazy_static;
    use std::iter::Iterator;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use std::{thread, time};

    fn open_test_client(name: &str) -> Client {
        Client::new(name, ClientOptions::NO_START_SERVER).unwrap().0
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

        fn unowned(&self) -> RawMidi<'_> {
            RawMidi {
                time: self.time,
                bytes: &self.bytes,
            }
        }
    }

    struct IterTest<F: Send + Fn(MidiIter) -> Vec<OwnedRawMidi>> {
        stream: Vec<OwnedRawMidi>,
        collected: Vec<OwnedRawMidi>,
        collector: F,
        midi_in: Port<MidiIn>,
        midi_out: Port<MidiOut>,
    }

    impl<F: Send + Fn(MidiIter) -> Vec<OwnedRawMidi>> IterTest<F> {
        fn new(client: &Client, stream: Vec<OwnedRawMidi>, collector: F) -> IterTest<F> {
            IterTest {
                stream,
                collected: Vec::new(),
                collector,
                midi_in: client.register_port("in", MidiIn).unwrap(),
                midi_out: client.register_port("out", MidiOut).unwrap(),
            }
        }

        fn connector(&self) -> Connector {
            Connector {
                src: self.midi_out.name().unwrap(),
                dst: self.midi_in.name().unwrap(),
            }
        }
    }

    impl<F: Send + Fn(MidiIter) -> Vec<OwnedRawMidi>> ProcessHandler for IterTest<F> {
        fn process(&mut self, _: &Client, ps: &ProcessScope) -> Control {
            let (midi_in, mut midi_out) = (self.midi_in.iter(ps), self.midi_out.writer(ps));
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
        let in_a = c.register_port("ia", MidiIn).unwrap();
        let in_b = c.register_port("ib", MidiIn).unwrap();
        let mut out_a = c.register_port("oa", MidiOut).unwrap();
        let mut out_b = c.register_port("ob", MidiOut).unwrap();

        // set callback routine
        let (signal_succeed, did_succeed) = bounded(1_000);
        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
            let exp_a = RawMidi {
                time: 0,
                bytes: &[0b1001_0000, 0b0100_0000],
            };
            let exp_b = RawMidi {
                time: 64,
                bytes: &[0b1000_0000, 0b0100_0000],
            };
            let in_a = in_a.iter(ps);
            let in_b = in_b.iter(ps);
            let mut out_a = out_a.writer(ps);
            let mut out_b = out_b.writer(ps);
            out_a.write(&exp_a).unwrap();
            out_b.write(&exp_b).unwrap();
            if in_a.clone().next().is_some()
                && in_a.clone().all(|m| m == exp_a)
                && in_b.clone().all(|m| m == exp_b)
            {
                signal_succeed.send(true).unwrap();
            }
            Control::Continue
        };

        // activate
        let ac = c
            .activate_async((), ClosureProcessHandler::new(process_callback))
            .unwrap();

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

    static PMCGMES_MAX_EVENT_SIZE: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn port_midi_can_get_max_event_size() {
        // open clients and ports
        let c = open_test_client("port_midi_cglc");
        let mut out_p = c.register_port("op", MidiOut).unwrap();

        // set callback routine
        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
            let out_p = out_p.writer(ps);
            PMCGMES_MAX_EVENT_SIZE.fetch_add(out_p.max_event_size(), Ordering::Relaxed);
            Control::Continue
        };

        // activate
        let ac = c
            .activate_async((), ClosureProcessHandler::new(process_callback))
            .unwrap();

        // check correctness
        assert!(PMCGMES_MAX_EVENT_SIZE.load(Ordering::Relaxed) > 0);
        ac.deactivate().unwrap();
    }

    lazy_static! {
        static ref PMCEMES_WRITE_RESULT: Mutex<Result<(), Error>> = Mutex::new(Ok(()));
    }

    #[test]
    fn port_midi_cant_exceed_max_event_size() {
        // open clients and ports
        let c = open_test_client("port_midi_cglc");
        let mut out_p = c.register_port("op", MidiOut).unwrap();

        // set callback routine
        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
            let mut out_p = out_p.writer(ps);
            let event_size = out_p.max_event_size();
            PMCGMES_MAX_EVENT_SIZE.store(event_size, Ordering::Relaxed);

            let bytes: Vec<u8> = (0..=out_p.max_event_size()).map(|_| 0).collect();
            let msg = RawMidi {
                time: 0,
                bytes: &bytes,
            };

            *PMCEMES_WRITE_RESULT.lock().unwrap() = out_p.write(&msg);

            Control::Continue
        };

        // activate
        let ac = c
            .activate_async((), ClosureProcessHandler::new(process_callback))
            .unwrap();

        // check correctness
        assert_eq!(
            *PMCEMES_WRITE_RESULT.lock().unwrap(),
            Err(Error::NotEnoughSpace)
        );
        ac.deactivate().unwrap();
    }

    static PMI_COUNT: AtomicUsize = AtomicUsize::new(0);
    lazy_static! {
        static ref PMI_NEXT: Mutex<Option<(Frames, Vec<u8>)>> = Mutex::default();
        static ref PMI_SIZE_HINT: Mutex<(usize, Option<usize>)> = Mutex::new((0, None));
        static ref PMI_LAST: Mutex<Option<(Frames, Vec<u8>)>> = Mutex::default();
        static ref PMI_THIRD: Mutex<Option<(Frames, Vec<u8>)>> = Mutex::default();
    }

    #[test]
    fn port_midi_iter() {
        // open clients and ports
        let c = open_test_client("port_midi_iter");
        let in_p = c.register_port("ip", MidiIn).unwrap();
        let mut out_p = c.register_port("op", MidiOut).unwrap();

        // set callback routine
        let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
            let in_p = in_p.iter(ps);
            let mut out_p = out_p.writer(ps);

            for i in 10..14 {
                let msg = RawMidi {
                    time: i,
                    bytes: &[i as u8],
                };
                out_p.write(&msg).ok();
            }

            let rm_to_owned = |m: &RawMidi| (m.time, m.bytes.to_vec());
            *PMI_NEXT.lock().unwrap() = in_p.clone().next().map(|m| rm_to_owned(&m));
            *PMI_SIZE_HINT.lock().unwrap() = in_p.size_hint();
            PMI_COUNT.store(in_p.clone().count(), Ordering::Relaxed);
            *PMI_LAST.lock().unwrap() = in_p.clone().last().map(|m| rm_to_owned(&m));
            *PMI_THIRD.lock().unwrap() = in_p.clone().nth(2).map(|m| rm_to_owned(&m));

            Control::Continue
        };

        // run
        let ac = c
            .activate_async((), ClosureProcessHandler::new(process_callback))
            .unwrap();
        ac.as_client()
            .connect_ports_by_name("port_midi_iter:op", "port_midi_iter:ip")
            .unwrap();
        thread::sleep(time::Duration::from_millis(200));
        ac.deactivate().unwrap();

        // check correctness
        assert_eq!(*PMI_NEXT.lock().unwrap(), Some((10, [10].to_vec())));
        assert_eq!(*PMI_SIZE_HINT.lock().unwrap(), (4, Some(4)));
        assert_eq!(PMI_COUNT.load(Ordering::Relaxed), 4);
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
        let collect = |midi_in: MidiIter| {
            let mut collected = Vec::with_capacity(midi_in.clone().count());
            let mut iter = midi_in.clone();
            while let Some(m) = iter.next_if(|m| m.time < 11) {
                collected.push(OwnedRawMidi::new(&m));
            }
            collected
        };
        let processor = IterTest::new(&c, stream.clone(), collect);
        let connector = processor.connector();

        let ac = c.activate_async((), processor).unwrap();
        connector.connect(ac.as_client());
        thread::sleep(time::Duration::from_millis(200));

        let (_, _, processor) = ac.deactivate().unwrap();
        let expected: &[OwnedRawMidi] = &stream[0..2];
        let got: &[OwnedRawMidi] = &processor.collected;
        assert_eq!(expected, got);
    }
}
