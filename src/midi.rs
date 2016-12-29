use std::mem;
use jack_sys as j;
use rimd;
use jack_flags::port_flags::{IS_INPUT, IS_OUTPUT, PortFlags};
use port::PortData;

pub trait MidiStream: Sized {
    unsafe fn ptr(&self) -> *mut ::libc::c_void;

    fn iter<'a>(&'a self) -> MidiIter<'a, Self> {
        let n = self.len();
        MidiIter {
            stream: self,
            len: n,
            index: 0,
        }
    }

    fn nth(&self, n: usize) -> Option<MidiEvent> {
        if n < self.len() {
            MidiEvent::from_ptr(unsafe { self.ptr() }, n as u32)
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        let n = unsafe { j::jack_midi_get_event_count(self.ptr()) };
        n as usize
    }
}

/// `MidiStreamReader` implements the `PortData` trait, which defines
/// an endpoint for JACK. In this case, it provides readable midi
/// messages.
#[derive(Debug)]
pub struct MidiStreamReader {
    buffer_ptr: *mut ::libc::c_void,
}

impl MidiStream for MidiStreamReader {
    unsafe fn ptr(&self) -> *mut ::libc::c_void {
        self.buffer_ptr
    }
}

unsafe impl PortData for MidiStreamReader {
    unsafe fn from_ptr(ptr: *mut ::libc::c_void, _: u32) -> Self {
        MidiStreamReader { buffer_ptr: ptr }
    }

    fn jack_port_type() -> &'static str {
        "8 bit raw midi"
    }

    fn jack_flags() -> PortFlags {
        IS_INPUT
    }

    fn jack_buffer_size() -> u64 {
        // Not needed for built in types according to jack api
        0
    }
}

#[derive(Debug)]
pub struct MidiStreamWriter {
    buffer_ptr: *mut ::libc::c_void,
}

impl MidiStreamWriter {
    pub fn write(&mut self, events: &[MidiEvent]) {
        let mut _events_vec = Vec::new();
        unsafe {
            let mut events = events;
            let mut is_sorted = true;
            for i in 0..events.len() - 1 {
                if events[i].time() > events[i + 1].time() {
                    is_sorted = false;
                }
            }
            if !is_sorted {
                _events_vec = events.to_vec();
                _events_vec.sort_by(|a, b| {
                    match (a.time() < b.time(), a.time() > b.time()) {
                        (true, _) => ::std::cmp::Ordering::Less,
                        (_, true) => ::std::cmp::Ordering::Greater,
                        (_, _) => ::std::cmp::Ordering::Equal,
                    }
                });
                events = &_events_vec;
            }
            for e in events.iter() {
                j::jack_midi_event_reserve(self.buffer_ptr, e.time(), e.message().data.len());
                j::jack_midi_event_write(self.buffer_ptr,
                                         e.time(),
                                         e.message().data.as_ptr(),
                                         e.message().data.len());
            }
        }
    }
}

impl MidiStream for MidiStreamWriter {
    unsafe fn ptr(&self) -> *mut ::libc::c_void {
        self.buffer_ptr
    }
}

unsafe impl PortData for MidiStreamWriter {
    unsafe fn from_ptr(ptr: *mut ::libc::c_void, _: u32) -> Self {
        j::jack_midi_clear_buffer(ptr);
        MidiStreamWriter { buffer_ptr: ptr }
    }

    fn jack_port_type() -> &'static str {
        "8 bit raw midi"
    }

    fn jack_flags() -> PortFlags {
        IS_OUTPUT
    }

    fn jack_buffer_size() -> u64 {
        // Not needed for built in types according to jack api
        0
    }
}


/// Iterate over `MidiEvent`.
#[derive(Debug)]
pub struct MidiIter<'a, S: MidiStream + 'a> {
    stream: &'a S,
    len: usize,
    index: usize,
}

impl<'a, S: MidiStream + 'a> Iterator for MidiIter<'a, S> {
    type Item = MidiEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.stream.nth(self.index);
        self.index += 1;
        ret
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let elements_left = self.len - self.index;
        (elements_left, Some(elements_left))
    }

    fn count(self) -> usize {
        self.len - self.index
    }

    fn last(self) -> Option<Self::Item> {
        self.stream.nth(self.len - 1)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.index += n;
        self.next()
    }
}


#[derive(Clone, Debug)]
pub struct MidiEvent {
    message: rimd::MidiMessage,
    time: u32,
}

impl MidiEvent {
    pub fn new(message: rimd::MidiMessage, time: u32) -> MidiEvent {
        MidiEvent {
            message: message,
            time: time,
        }
    }

    pub fn from_ptr(ptr: *mut ::libc::c_void, i: u32) -> Option<Self> {
        unsafe {
            let mut e: j::jack_midi_event_t = mem::uninitialized();
            let res = j::jack_midi_event_get(&mut e, ptr, i);
            if res != 0 {
                return None;
            }
            let bytes_slice: &[u8] = ::std::slice::from_raw_parts(e.buffer as *const u8, e.size);
            let bytes_vec: Vec<u8> = bytes_slice.to_vec();
            let message = rimd::MidiMessage::from_bytes(bytes_vec);
            Some(MidiEvent {
                message: message,
                time: e.time,
            })
        }
    }

    pub fn message(&self) -> &rimd::MidiMessage {
        &self.message
    }

    pub fn time(&self) -> u32 {
        self.time
    }
}
