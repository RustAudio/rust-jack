#[cfg(feature = "dlopen")]
use crate::LIB;
use dlib::ffi_dispatch;
#[cfg(not(feature = "dlopen"))]
use jack_sys::*;
use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};

/// A lock-free ringbuffer. The key attribute of a ringbuffer is that it can be safely accessed by
/// two threads simultaneously, one reading from the buffer and the other writing to it - without
/// using any synchronization or mutual exclusion primitives.  For this to work correctly, there can
/// only be a single reader and a single writer thread. Their identities cannot be interchanged.
///
/// # Example
/// ```
/// let ringbuf = jack::RingBuffer::new(1024).unwrap();
/// let (mut reader, mut writer) = ringbuf.into_reader_writer();
///
/// let buf = [0_u8, 1, 2, 3];
/// let num = writer.write_buffer(&buf);
/// assert_eq!(num, buf.len());
///
/// // Potentially in a another thread:
/// let mut outbuf = [0_u8; 8];
/// let num = reader.read_buffer(&mut outbuf);
/// ```
pub struct RingBuffer(*mut jack_sys::jack_ringbuffer_t);

impl RingBuffer {
    /// Allocates a ringbuffer of a specified size.
    pub fn new(size: usize) -> Result<Self, crate::Error> {
        let insize = size as libc::size_t;
        let handle = unsafe { ffi_dispatch!(LIB, jack_ringbuffer_create, insize) };

        if handle.is_null() {
            return Err(crate::Error::RingbufferCreateFailed);
        }

        Ok(RingBuffer(handle))
    }

    /// Lock a ringbuffer data block into memory.
    pub fn mlock(&mut self) {
        unsafe { ffi_dispatch!(LIB, jack_ringbuffer_mlock, self.0) };
    }

    /// Resets the ring buffer, making an empty buffer.
    pub fn reset(&mut self) {
        unsafe { ffi_dispatch!(LIB, jack_ringbuffer_reset, self.0) };
    }

    /// Create a reader and writer, to use the ring buffer.
    pub fn into_reader_writer(self) -> (RingBufferReader, RingBufferWriter) {
        let out = unsafe { (RingBufferReader::new(self.0), RingBufferWriter::new(self.0)) };
        mem::forget(self);
        out
    }

    /// Re-create the ring buffer object from reader and writer. useful if you need to call reset.
    /// The reader and the writer pair must have been created from the same RingBuffer object.  Not
    /// needed for deallocation, disposing of both reader and writer will deallocate buffer
    /// resources automatically.
    ///
    /// panics if the reader and the writer were created from different RingBuffer objects.
    pub fn from_reader_writer(r: RingBufferReader, w: RingBufferWriter) -> Self {
        if r.ringbuffer_handle != w.ringbuffer_handle {
            // drops will be valid during unwinding - assuming that all reader/writer pairs are
            // consisitent.
            panic!("mismatching read and write handles!")
        }

        // The next 3 lines transfer ownership of the ringbuffer from both reader and writer to the
        // new buffer object.
        let handle = RingBuffer(r.ringbuffer_handle);
        mem::forget(r);
        mem::forget(w);

        handle
    }
}

impl Drop for RingBuffer {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { ffi_dispatch!(LIB, jack_ringbuffer_free, self.0) };
        }
        self.0 = std::ptr::null_mut();
    }
}

unsafe impl Send for RingBuffer {}
unsafe impl Sync for RingBuffer {}

/// Read end of the ring buffer. Can only be used from one thread (can be different from the write
/// thread).
pub struct RingBufferReader {
    ringbuffer_handle: *mut jack_sys::jack_ringbuffer_t,
    /// A marker to check if both halves of the ringbuffer are live. Destroying a ringbuffer is not
    /// a realtime operation.
    both_live: AtomicBool,
}

unsafe impl Send for RingBufferReader {}
unsafe impl Sync for RingBufferReader {}

/// Write end of the ring buffer. Can only be used from one thread (can be a different from the read
/// thread).
pub struct RingBufferWriter {
    ringbuffer_handle: *mut jack_sys::jack_ringbuffer_t,
    both_live: AtomicBool,
}

unsafe impl Send for RingBufferWriter {}
unsafe impl Sync for RingBufferWriter {}

impl RingBufferReader {
    // safety: this method must be called as part of the splitting of the ringbuffer into 2
    // channels.
    unsafe fn new(raw: *mut jack_sys::jack_ringbuffer_t) -> Self {
        RingBufferReader {
            ringbuffer_handle: raw,
            both_live: AtomicBool::new(true),
        }
    }

    /// Fill a data structure with a description of the current readable data held in the
    /// ringbuffer. This description is returned in a two slices. Two slices are needed because the
    /// data to be read may be split across the end of the ringbuffer. The first slice represents
    /// the bytes ready to be read. If the second slice is not empty, it is the continuation of the
    /// data that ended in the first slices. For convenience, consider using peek_iter instead.
    pub fn get_vector(&self) -> (&[u8], &[u8]) {
        let mut vec = [
            jack_sys::jack_ringbuffer_data_t::default(),
            jack_sys::jack_ringbuffer_data_t::default(),
        ];
        let vecstart = &mut vec[0] as *mut jack_sys::jack_ringbuffer_data_t;

        unsafe {
            ffi_dispatch!(
                LIB,
                jack_ringbuffer_get_read_vector,
                self.ringbuffer_handle,
                vecstart
            )
        };

        let view1 = vec[0];
        let view2 = vec[1];

        let buf1 = view1.buf as *mut u8;
        let len1 = view1.len as usize;

        let mut buf2 = view2.buf as *mut u8;
        let len2 = view2.len as usize;

        if len2 == 0 {
            // buf2 can't be null even if length is zero, so just use buf1
            buf2 = buf1;
        }

        let view1 = unsafe { std::slice::from_raw_parts(buf1, len1) };
        let view2 = unsafe { std::slice::from_raw_parts(buf2, len2) };
        (view1, view2)
    }

    /// Read data from the ringbuffer.  Returns: the number of bytes read, which may range from 0 to
    /// buf.len().
    pub fn read_buffer(&mut self, buf: &mut [u8]) -> usize {
        if buf.is_empty() {
            return 0;
        }

        let insize: libc::size_t = buf.len() as libc::size_t;
        let bufstart = &mut buf[0] as *mut _ as *mut libc::c_char;

        let read = unsafe {
            ffi_dispatch!(
                LIB,
                jack_ringbuffer_read,
                self.ringbuffer_handle,
                bufstart,
                insize
            )
        };
        read as usize
    }

    /// Read data from the ringbuffer. Opposed to read_buffer() this function does not move the read
    /// pointer.  Thus it's a convenient way to inspect data in the ringbuffer in a continous
    /// fashion.  The price is that the data is copied into a user provided buffer.  For "raw"
    /// non-copy inspection of the data in the ringbuffer use get_vector() or peek_iter.  Returns:
    /// the number of bytes read, which may range from 0 to buf.len()
    pub fn peek(&self, buf: &mut [u8]) -> usize {
        if buf.is_empty() {
            return 0;
        }

        let insize: libc::size_t = buf.len() as libc::size_t;
        let bufstart = &mut buf[0] as *mut _ as *mut libc::c_char;

        let read = unsafe {
            ffi_dispatch!(
                LIB,
                jack_ringbuffer_peek,
                self.ringbuffer_handle,
                bufstart,
                insize
            )
        };
        read as usize
    }

    /// Advance the read pointer. use this after peek/peek_iter or get_vector to advance the buffer
    /// pointer.
    pub fn advance(&mut self, cnt: usize) {
        let incnt = cnt as libc::size_t;
        unsafe {
            ffi_dispatch!(
                LIB,
                jack_ringbuffer_read_advance,
                self.ringbuffer_handle,
                incnt
            )
        };
    }

    /// Return the number of bytes available for reading.
    pub fn space(&self) -> usize {
        unsafe { ffi_dispatch!(LIB, jack_ringbuffer_read_space, self.ringbuffer_handle) as usize }
    }

    /// Iterator that goes over all the data available to read.
    pub fn peek_iter(
        &'_ self,
    ) -> std::iter::Chain<std::slice::Iter<'_, u8>, std::slice::Iter<'_, u8>> {
        let (view1, view2) = self.get_vector();

        view1.iter().chain(view2.iter())
    }
}

impl std::io::Read for RingBufferReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(self.read_buffer(buf))
    }
}

impl Drop for RingBufferReader {
    fn drop(&mut self) {
        match self
            .both_live
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        {
            Ok(false) | Err(false) => {
                drop(RingBuffer(self.ringbuffer_handle));
            }
            _ => (),
        }
    }
}

impl RingBufferWriter {
    // safety: this method must be called as part of the splitting of the ringbuffer into 2
    // channels.
    unsafe fn new(raw: *mut jack_sys::jack_ringbuffer_t) -> Self {
        RingBufferWriter {
            ringbuffer_handle: raw,
            both_live: AtomicBool::new(true),
        }
    }

    /// Write data into the ringbuffer.  Returns: The number of bytes written, which may range from
    /// 0 to buf.len()
    pub fn write_buffer(&mut self, buf: &[u8]) -> usize {
        if buf.is_empty() {
            return 0;
        }

        let insize: libc::size_t = buf.len() as libc::size_t;
        let bufstart = &buf[0] as *const _ as *const libc::c_char;

        let read = unsafe {
            ffi_dispatch!(
                LIB,
                jack_ringbuffer_write,
                self.ringbuffer_handle,
                bufstart,
                insize
            )
        };
        read as usize
    }

    /// Advance the write pointer. use this after peek_iter or get_vector to advance the buffer
    /// pointer.
    pub fn advance(&mut self, cnt: usize) {
        let incnt = cnt as libc::size_t;
        unsafe {
            ffi_dispatch!(
                LIB,
                jack_ringbuffer_write_advance,
                self.ringbuffer_handle,
                incnt
            )
        };
    }

    /// Return the number of bytes available for writing.
    pub fn space(&mut self) -> usize {
        unsafe { ffi_dispatch!(LIB, jack_ringbuffer_write_space, self.ringbuffer_handle) as usize }
    }

    /// Return a pair of slices of the current writable space in the ringbuffer. two slices are
    /// needed because the space available for writing may be split across the end of the
    /// ringbuffer.  consider using peek_iter for convenience.
    pub fn get_vector(&mut self) -> (&mut [u8], &mut [u8]) {
        let mut vec = [
            jack_sys::jack_ringbuffer_data_t::default(),
            jack_sys::jack_ringbuffer_data_t::default(),
        ];
        let vecstart = &mut vec[0] as *mut jack_sys::jack_ringbuffer_data_t;

        unsafe {
            ffi_dispatch!(
                LIB,
                jack_ringbuffer_get_write_vector,
                self.ringbuffer_handle,
                vecstart
            )
        };

        let view1 = vec[0];
        let view2 = vec[1];

        let buf1 = view1.buf as *mut u8;
        let len1 = view1.len as usize;

        let mut buf2 = view2.buf as *mut u8;
        let len2 = view2.len as usize;

        if len2 == 0 {
            // buf2 can't be null even if length is zero, so just use buf1
            buf2 = buf1;
        }

        let view1 = unsafe { std::slice::from_raw_parts_mut(buf1, len1) };
        let view2 = unsafe { std::slice::from_raw_parts_mut(buf2, len2) };
        (view1, view2)
    }

    /// Iterator that goes over all the data available to write.
    pub fn peek_iter(
        &'_ mut self,
    ) -> std::iter::Chain<std::slice::IterMut<'_, u8>, std::slice::IterMut<'_, u8>> {
        let (view1, view2) = self.get_vector();

        view1.iter_mut().chain(view2.iter_mut())
    }
}

impl std::io::Write for RingBufferWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(self.write_buffer(buf))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for RingBufferWriter {
    fn drop(&mut self) {
        match self
            .both_live
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        {
            Ok(false) | Err(false) => {
                drop(RingBuffer(self.ringbuffer_handle));
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ringbuffer_can_create() {
        let ringbuf = RingBuffer::new(1024);
        ringbuf.unwrap();
    }

    #[test]
    fn ringbuffer_can_space() {
        const SIZE: usize = 1024;
        const ADVANCE: usize = 5;
        let ringbuf = RingBuffer::new(SIZE).unwrap();
        let (mut reader, mut writer) = ringbuf.into_reader_writer();

        assert_eq!(writer.space(), SIZE - 1);
        assert_eq!(reader.space(), 0);

        writer.advance(ADVANCE);

        assert_eq!(writer.space(), SIZE - 1 - ADVANCE);
        assert_eq!(reader.space(), ADVANCE);

        reader.advance(ADVANCE);
        assert_eq!(writer.space(), SIZE - 1);
        assert_eq!(reader.space(), 0);
    }

    #[test]
    fn ringbuffer_write_read() {
        let ringbuf = RingBuffer::new(1024).unwrap();
        let (mut reader, mut writer) = ringbuf.into_reader_writer();

        let buf = [0_u8, 1, 2, 3];
        let num = writer.write_buffer(&buf);
        assert_eq!(num, buf.len());

        let mut outbuf = [0_u8; 8];
        let num = reader.read_buffer(&mut outbuf);
        assert_eq!(num, buf.len());

        assert_eq!(outbuf[..num], buf[..]);
    }

    #[test]
    fn ringbuffer_peek_write() {
        let ringbuf = RingBuffer::new(1024).unwrap();
        let (reader, mut writer) = ringbuf.into_reader_writer();

        let buf = [0_u8, 1, 2, 3];
        writer.write_buffer(&buf);

        let data: Vec<u8> = reader.peek_iter().copied().collect();

        assert_eq!(data.len(), buf.len());
        assert_eq!(data[..], buf[..]);
    }

    #[test]
    fn ringbuffer_write_read_split() {
        const BUFSIZE: usize = 10;
        let ringbuf = RingBuffer::new(BUFSIZE).unwrap();
        let (mut reader, mut writer) = ringbuf.into_reader_writer();

        let buf = [0_u8, 1, 2, 3];

        let advancedsize = BUFSIZE / (buf.len() / 2);
        writer.advance(advancedsize);
        reader.advance(advancedsize);
        {
            let (_, v2) = writer.get_vector();
            assert_ne!(v2.len(), 0);
        }

        writer.write_buffer(&buf);

        {
            let (v1, _) = reader.get_vector();
            assert_ne!(v1.len(), 0);
        }

        let data: Vec<u8> = reader.peek_iter().copied().collect();

        assert_eq!(data.len(), buf.len());
        assert_eq!(data[..], buf[..]);
    }

    #[test]
    fn ringbuffer_peek_read() {
        let ringbuf = RingBuffer::new(1024).unwrap();
        let (mut reader, mut writer) = ringbuf.into_reader_writer();

        let buf = [0_u8, 1, 2, 3];
        for (item, bufitem) in writer.peek_iter().zip(buf.iter()) {
            *item = *bufitem;
        }

        writer.advance(buf.len());

        let mut outbuf = [0_u8; 8];
        let num = reader.read_buffer(&mut outbuf);
        assert_eq!(num, buf.len());

        assert_eq!(outbuf[..num], buf[..]);
    }

    #[test]
    fn ringbuffer_threaded() {
        use std::thread;

        let ringbuf = RingBuffer::new(1024).unwrap();
        let (mut reader, mut writer) = ringbuf.into_reader_writer();

        let buf = [0_u8, 1, 2, 3];
        thread::spawn(move || {
            for (item, bufitem) in writer.peek_iter().zip(buf.iter()) {
                *item = *bufitem;
            }

            writer.advance(buf.len());
        })
        .join()
        .unwrap();

        let mut outbuf = [0_u8; 8];
        let num = reader.read_buffer(&mut outbuf);
        assert_eq!(num, buf.len());

        assert_eq!(outbuf[..num], buf[..]);
    }
}
