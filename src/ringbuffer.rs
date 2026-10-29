use jack_sys as j;
use std::mem::ManuallyDrop;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
pub struct RingBuffer(*mut j::jack_ringbuffer_t);

impl RingBuffer {
    /// Allocates a ringbuffer of a specified size.
    pub fn new(size: usize) -> Result<Self, crate::Error> {
        if let Err(err) = jack_sys::ensure_loaded() {
            return Err(crate::Error::LibraryError(err.to_string()));
        }
        let insize = size as libc::size_t;
        let handle = unsafe { j::jack_ringbuffer_create(insize) };

        if handle.is_null() {
            return Err(crate::Error::RingbufferCreateFailed);
        }

        Ok(RingBuffer(handle))
    }

    /// Lock a ringbuffer data block into memory.
    pub fn mlock(&mut self) {
        unsafe { j::jack_ringbuffer_mlock(self.0) };
    }

    /// Resets the ring buffer, making an empty buffer.
    pub fn reset(&mut self) {
        unsafe { j::jack_ringbuffer_reset(self.0) };
    }

    /// Create a reader and writer, to use the ring buffer.
    pub fn into_reader_writer(self) -> (RingBufferReader, RingBufferWriter) {
        let this = ManuallyDrop::new(self);
        let both_live = Arc::new(AtomicBool::new(true));
        unsafe {
            (
                RingBufferReader::new(this.0, both_live.clone()),
                RingBufferWriter::new(this.0, both_live),
            )
        }
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

        // Transfer ownership of the ringbuffer from both reader and writer to the new buffer
        // object. Their `Drop` must not run, but their shared liveness flag still has to be freed.
        let r = ManuallyDrop::new(r);
        let w = ManuallyDrop::new(w);
        unsafe {
            drop(ptr::read(&r.both_live));
            drop(ptr::read(&w.both_live));
        }
        RingBuffer(r.ringbuffer_handle)
    }
}

// Counts the ring buffers freed on the current thread, so tests can detect leaks.
#[cfg(test)]
thread_local! {
    pub(crate) static FREED_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

impl Drop for RingBuffer {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { j::jack_ringbuffer_free(self.0) };
            #[cfg(test)]
            FREED_COUNT.with(|c| c.set(c.get() + 1));
        }
        self.0 = std::ptr::null_mut();
    }
}

unsafe impl Send for RingBuffer {}
unsafe impl Sync for RingBuffer {}

/// Read end of the ring buffer. Can only be used from one thread (can be different from the write
/// thread).
pub struct RingBufferReader {
    ringbuffer_handle: *mut j::jack_ringbuffer_t,
    /// Shared with the writer. `true` while both halves are live. The half that is dropped last
    /// frees the ringbuffer. Destroying a ringbuffer is not a realtime operation.
    both_live: Arc<AtomicBool>,
}

unsafe impl Send for RingBufferReader {}
unsafe impl Sync for RingBufferReader {}

/// Write end of the ring buffer. Can only be used from one thread (can be a different from the read
/// thread).
pub struct RingBufferWriter {
    ringbuffer_handle: *mut j::jack_ringbuffer_t,
    /// Shared with the reader. See `RingBufferReader::both_live`.
    both_live: Arc<AtomicBool>,
}

unsafe impl Send for RingBufferWriter {}
unsafe impl Sync for RingBufferWriter {}

impl RingBufferReader {
    // safety: this method must be called as part of the splitting of the ringbuffer into 2
    // channels.
    unsafe fn new(raw: *mut j::jack_ringbuffer_t, both_live: Arc<AtomicBool>) -> Self {
        RingBufferReader {
            ringbuffer_handle: raw,
            both_live,
        }
    }

    /// Fill a data structure with a description of the current readable data held in the
    /// ringbuffer. This description is returned in a two slices. Two slices are needed because the
    /// data to be read may be split across the end of the ringbuffer. The first slice represents
    /// the bytes ready to be read. If the second slice is not empty, it is the continuation of the
    /// data that ended in the first slices. For convenience, consider using peek_iter instead.
    pub fn get_vector(&self) -> (&[u8], &[u8]) {
        let mut vec = [
            j::jack_ringbuffer_data_t::default(),
            j::jack_ringbuffer_data_t::default(),
        ];
        let vecstart = &mut vec[0] as *mut j::jack_ringbuffer_data_t;

        unsafe { j::jack_ringbuffer_get_read_vector(self.ringbuffer_handle, vecstart) };

        let view1 = vec[0];
        let view2 = vec[1];

        let buf1 = view1.buf as *mut u8;
        let len1 = view1.len;

        let mut buf2 = view2.buf as *mut u8;
        let len2 = view2.len;

        if len2 == 0 {
            // buf2 can't be null even if length is zero, so just use buf1
            buf2 = buf1;
        }

        let view1 = unsafe { std::slice::from_raw_parts(buf1, len1) };
        let view2 = unsafe { std::slice::from_raw_parts(buf2, len2) };
        (view1, view2)
    }

    /// Read data from the ringbuffer. Returns the number of bytes read, which may range from 0 to
    /// buf.len().
    pub fn read_buffer(&mut self, buf: &mut [u8]) -> usize {
        if buf.is_empty() {
            return 0;
        }

        let insize: libc::size_t = buf.len() as libc::size_t;
        let bufstart = &mut buf[0] as *mut _ as *mut libc::c_char;

        unsafe { j::jack_ringbuffer_read(self.ringbuffer_handle, bufstart, insize) }
    }

    /// Read data from the ringbuffer. Returns the slice that was read into. This is a subset of `buf`.
    pub fn read_slice<'a>(&mut self, buf: &'a mut [u8]) -> &'a [u8] {
        let len = self.read_buffer(buf);
        &buf[0..len]
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

        unsafe { j::jack_ringbuffer_peek(self.ringbuffer_handle, bufstart, insize) }
    }

    /// Advance the read pointer. use this after peek/peek_iter or get_vector to advance the buffer
    /// pointer.
    pub fn advance(&mut self, cnt: usize) {
        let incnt = cnt as libc::size_t;
        unsafe { j::jack_ringbuffer_read_advance(self.ringbuffer_handle, incnt) };
    }

    /// Return the number of bytes available for reading.
    pub fn space(&self) -> usize {
        unsafe { j::jack_ringbuffer_read_space(self.ringbuffer_handle) }
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
        // The first half to be dropped flips the flag. The second one sees it already cleared and
        // frees the ringbuffer.
        if !self.both_live.swap(false, Ordering::AcqRel) {
            drop(RingBuffer(self.ringbuffer_handle));
        }
    }
}

impl RingBufferWriter {
    // safety: this method must be called as part of the splitting of the ringbuffer into 2
    // channels.
    unsafe fn new(raw: *mut j::jack_ringbuffer_t, both_live: Arc<AtomicBool>) -> Self {
        RingBufferWriter {
            ringbuffer_handle: raw,
            both_live,
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

        unsafe { j::jack_ringbuffer_write(self.ringbuffer_handle, bufstart, insize) }
    }

    /// Advance the write pointer. use this after peek_iter or get_vector to advance the buffer
    /// pointer.
    pub fn advance(&mut self, cnt: usize) {
        let incnt = cnt as libc::size_t;
        unsafe { j::jack_ringbuffer_write_advance(self.ringbuffer_handle, incnt) };
    }

    /// Return the number of bytes available for writing.
    pub fn space(&mut self) -> usize {
        unsafe { j::jack_ringbuffer_write_space(self.ringbuffer_handle) }
    }

    /// Return a pair of slices of the current writable space in the ringbuffer. two slices are
    /// needed because the space available for writing may be split across the end of the
    /// ringbuffer. Consider using peek_iter for convenience.
    pub fn get_vector(&mut self) -> (&mut [u8], &mut [u8]) {
        let mut vec = [
            j::jack_ringbuffer_data_t::default(),
            j::jack_ringbuffer_data_t::default(),
        ];
        let vecstart = &mut vec[0] as *mut j::jack_ringbuffer_data_t;

        unsafe { j::jack_ringbuffer_get_write_vector(self.ringbuffer_handle, vecstart) };

        let view1 = vec[0];
        let view2 = vec[1];

        let buf1 = view1.buf as *mut u8;
        let len1 = view1.len;

        let mut buf2 = view2.buf as *mut u8;
        let len2 = view2.len;

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
        // The first half to be dropped flips the flag. The second one sees it already cleared and
        // frees the ringbuffer.
        if !self.both_live.swap(false, Ordering::AcqRel) {
            drop(RingBuffer(self.ringbuffer_handle));
        }
    }
}
