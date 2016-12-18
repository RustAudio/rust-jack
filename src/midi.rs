use std::mem;
use jack_sys as j;
use rimd;
use jack_flags::port_flags::{IS_INPUT, PortFlags};
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


#[derive(Debug)]
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
