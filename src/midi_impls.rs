use std::cell::Cell;
use std::slice;
use std::mem;

use jack_sys as j;

use jack_flags::port_flags::{IS_INPUT, PortFlags};
use port::{Port, PortSpec};
use callbacks::ProcessScope;

#[derive(Clone, Copy, Debug)]
pub struct RawMidi<'a> {
    pub time: u32,
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

/// `MidiInSpec` implements the `PortSpec` trait which, defines an
/// endpoint for JACK.
#[derive(Debug, Default)]
pub struct MidiInSpec;

unsafe impl PortSpec for MidiInSpec {
    /// # Arguments
    ///
    /// * `ptr` - buffer pointer to underlying data.
    ///
    /// * `nframes` - the size of the buffer.

    fn jack_port_type(&self) -> &'static str {
        "8 bit raw midi"
    }

    fn jack_flags(&self) -> PortFlags {
        IS_INPUT
    }

    fn jack_buffer_size(&self) -> u64 {
        // Not needed for built in types according to JACK api
        0
    }
}

/// Safetly wrap a `Port<MidiInPort>`. Derefs into a `&[f32]`.
pub struct MidiInPort<'a> {
    _port: &'a Port<MidiInSpec>,
    buffer_ptr: *mut ::libc::c_void,
    message: Cell<RawMidi<'a>>,
}

impl<'a> MidiInPort<'a> {
    /// Wrap a `Port<MidiInSpec>` within a process scope of a client
    /// that registered the port. Panics if the port does not belong
    /// to the client that created the process.
    pub fn new(port: &'a Port<MidiInSpec>, ps: &'a ProcessScope) -> Self {
        unsafe { assert_eq!(port.client_ptr(), ps.client_ptr()) };
        let buffer_ptr = unsafe { port.buffer(ps.n_frames()) };
        MidiInPort {
            _port: port,
            buffer_ptr: buffer_ptr,
            message: Cell::new(RawMidi::default()),
        }
    }

    pub fn nth(&self, n: usize) -> Option<RawMidi> {
        let mut ev: j::jack_midi_event_t = unsafe { mem::uninitialized() };
        let res = unsafe { j::jack_midi_event_get(&mut ev, self.buffer_ptr, n as u32) };
        if res != 0 {
            return None;
        }
        let bytes_slice: &[u8] = unsafe { slice::from_raw_parts(ev.buffer as *const u8, ev.size) };
        self.message.set(RawMidi {
            time: ev.time,
            bytes: bytes_slice,
        });
        Some(self.message.get())
    }

    pub fn len(&self) -> usize {
        let n = unsafe { j::jack_midi_get_event_count(self.buffer_ptr) };
        n as usize
    }
}

pub struct MidiIter<'a> {
    port: &'a MidiInPort<'a>,
    index: usize,
}

impl<'a> Iterator for MidiIter<'a> {
    type Item = RawMidi<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.port.nth(self.index);
        self.index += 1;
        ret
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let elements_left = self.port.len() - self.index;
        (elements_left, Some(elements_left))
    }

    fn count(self) -> usize {
        self.size_hint().0
    }

    fn last(self) -> Option<Self::Item> {
        self.port.nth(self.port.len() - 1)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.index += n;
        self.next()
    }
}
