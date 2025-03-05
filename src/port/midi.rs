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

/// [`MidiIn`] implements the [`PortSpec`] trait, which defines an endpoint for JACK. In this case, it
/// defines midi input.
#[derive(Copy, Clone, Debug, Default)]
pub struct MidiIn {
    _internal: (),
}

/// [`MidiOut`] implements the [`PortSpec`] trait, which defines an endpoint for JACK. In this case, it
/// defines a midi output.
#[derive(Copy, Clone, Debug, Default)]
pub struct MidiOut {
    _internal: (),
}

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

/// Iterate through Midi Messages within a [`Port<MidiIn>`].
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
        match -res {
            0 => Ok(()),
            libc::ENOBUFS => Err(Error::NotEnoughSpace),
            error_code => Err(Error::UnknownError { error_code }),
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
