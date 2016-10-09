use jack_sys as j;
use rimd;

type MidiBufferPtr = *mut ::libc::c_void;

pub struct MidiStream {
    port_buffer_ptr: MidiBufferPtr,
}

impl MidiStream {
    pub unsafe fn from_port_buffer(ptr: *mut ::libc::c_void) -> MidiStream {
        MidiStream {
            port_buffer_ptr: ptr,
        }
    }

    pub fn iter<'a>(&'a self) -> MidiIter<'a> {
        let n = self.len();
        MidiIter {
            stream: &self,
            len: n,
            index: 0,
        }
    }

    pub fn nth(&self, n: usize) -> Option<MidiEvent> {
        if n < self.len() {
            MidiEvent::from_ptr(self.port_buffer_ptr, n as u32)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        let n = unsafe { j::jack_midi_get_event_count(self.port_buffer_ptr) };
        n as usize
    }
}

pub struct MidiIter<'a> {
    stream: &'a MidiStream,
    len: usize,
    index: usize,
}

impl<'a> Iterator for MidiIter<'a> {
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

    pub fn from_ptr(ptr: MidiBufferPtr, i: u32) -> Option<Self> {
        unsafe {
            let mut e: j::jack_midi_event_t = ::std::mem::uninitialized();
            let res = j::jack_midi_event_get(&mut e, ptr, i);
            if res != 0 {
                return None;
            }
            let bytes_slice: &[u8] = ::std::slice::from_raw_parts(e.buffer as *const u8,
                                                                  e.size);
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
