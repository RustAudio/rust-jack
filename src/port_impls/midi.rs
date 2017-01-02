use std::cell::Cell;
use std::{mem, slice};

use jack_sys as j;
use libc;

use callbacks::ProcessScope;
use jack_enums::JackErr;
use jack_flags::port_flags::{IS_INPUT, IS_OUTPUT, PortFlags};
use port::{Port, PortSpec};
use primitive_types as pt;

/// Contains 8bit raw midi information along with a timestamp relative to the process cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawMidi<'a> {
    /// The amount of time, in frames, relative to the start of the process cycle
    pub time: pt::JackFrames,
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

/// `MidiInSpec` implements the `PortSpec` trait, which defines an
/// endpoint for JACK.
#[derive(Debug, Default)]
pub struct MidiInSpec;

/// `MidiOutSpec` implements the `PortSpec` trait, which defines an
/// endpoint for JACK.
#[derive(Debug, Default)]
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

/// Safetly wrap a `Port<MidiInPort>`.
#[derive(Debug)]
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
        let res = unsafe { j::jack_midi_event_get(&mut ev, self.buffer_ptr, n as libc::uint32_t) };
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

    pub fn iter(&'a self) -> MidiIter {
        MidiIter {
            port: &self,
            index: 0,
        }
    }
}

/// Safetly wrap a `Port<MidiInPort>`.
#[derive(Debug)]
pub struct MidiOutPort<'a> {
    _port: &'a mut Port<MidiOutSpec>,
    buffer_ptr: *mut ::libc::c_void,
}

impl<'a> MidiOutPort<'a> {
    /// Wrap a `Port<MidiInSpec>` within a process scope of a client
    /// that registered the port. Panics if the port does not belong
    /// to the client that created the process.
    ///
    /// The data in the port is cleared.
    pub fn new(port: &'a mut Port<MidiOutSpec>, ps: &'a ProcessScope) -> Self {
        unsafe { assert_eq!(port.client_ptr(), ps.client_ptr()) };
        let buffer_ptr = unsafe { port.buffer(ps.n_frames()) };
        unsafe { j::jack_midi_clear_buffer(buffer_ptr) };
        MidiOutPort {
            _port: port,
            buffer_ptr: buffer_ptr,
        }
    }

    /// Write an event into an event port buffer.
    ///
    /// Clients must write normalised MIDI data to the port - no running status and no (1-byte)
    /// realtime messages intersperesed with other messagse (realtime messages are fine when they
    /// occur on their own, like other messages).
    pub fn write(&mut self, message: &RawMidi) -> Result<(), JackErr> {
        let ev = j::jack_midi_event_t {
            time: message.time,
            size: message.bytes.len(),
            buffer: message.bytes.as_ptr() as *mut u8,
        };
        let res = unsafe { j::jack_midi_event_write(self.buffer_ptr, ev.time, ev.buffer, ev.size) };
        match res {
            ::libc::ENOBUFS => Err(JackErr::NotEnoughSpace),
            0 => Ok(()),
            _ => Err(JackErr::UnknownError),
        }
    }

    /// Get the number of events that could not be written to port_buffer.
    ///
    /// If the return value is greater than 0, than the buffer is full. Currently, the only way this
    /// can happen is if events are lost on port mixdown.
    pub fn lost_count(&self) -> usize {
        let n = unsafe { j::jack_midi_get_lost_event_count(self.buffer_ptr) };
        n as usize
    }

    /// Get the size of the largest event that can be stored by the port.
    ///
    /// This function returns the current space available, taking into account events already stored
    /// in the port.
    pub fn max_event_size(&self) -> usize {
        let n = unsafe { j::jack_midi_max_event_size(self.buffer_ptr) };
        n as usize
    }
}

/// Iterate through Midi Messages within a `MidiInPort`.
#[derive(Debug)]
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
