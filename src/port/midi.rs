use jack_sys as j;
use std::marker::PhantomData;
use std::ptr::NonNull;
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
        let buffer = NonNull::new(unsafe { self.buffer(ps.n_frames()) });
        MidiIter {
            buffer,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

/// Iterate through Midi Messages within a [`Port<MidiIn>`].
#[derive(Debug, Clone)]
pub struct MidiIter<'a> {
    buffer: Option<NonNull<::libc::c_void>>,
    index: usize,
    _phantom: PhantomData<&'a ()>,
}

unsafe impl Sync for MidiIter<'_> {}

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
        let buffer = self.buffer?.as_ptr();
        let mut ev = mem::MaybeUninit::<j::jack_midi_event_t>::uninit();
        let res = unsafe { j::jack_midi_event_get(ev.as_mut_ptr(), buffer, n) };
        if res != 0 {
            return None;
        }
        let ev = unsafe { ev.assume_init() };
        let time = ev.time;
        // `slice::from_raw_parts` requires a non-null pointer, even for an empty slice.
        let bytes: &[u8] = if ev.buffer.is_null() || ev.size == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(ev.buffer as *const u8, ev.size) }
        };
        Some(RawMidi { time, bytes })
    }

    fn absolute_len(&self) -> usize {
        let Some(buffer) = self.buffer else {
            return 0;
        };

        unsafe { j::jack_midi_get_event_count(buffer.as_ptr()) as usize }
    }
}

impl<'a> Iterator for MidiIter<'a> {
    type Item = RawMidi<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.peek()?;
        self.index += 1;
        Some(ret)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.absolute_len().saturating_sub(self.index);
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.absolute_len().saturating_sub(self.index)
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
        self.index = self.index.saturating_add(n).min(self.absolute_len());
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

impl MidiWriter<'_> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_iter() -> MidiIter<'static> {
        MidiIter {
            buffer: None,
            index: 0,
            _phantom: PhantomData,
        }
    }

    #[test]
    fn empty_iter_has_no_events() {
        let mut iter = empty_iter();
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.peek(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.clone().count(), 0);
        assert_eq!(iter.last(), None);
    }

    #[test]
    fn nth_past_the_end_does_not_underflow() {
        let mut iter = empty_iter();
        assert_eq!(iter.nth(5), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.clone().count(), 0);
        assert_eq!(iter.nth(usize::MAX), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    #[test]
    fn next_past_the_end_does_not_underflow() {
        let mut iter = empty_iter();
        for _ in 0..3 {
            assert_eq!(iter.next(), None);
        }
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }
}
