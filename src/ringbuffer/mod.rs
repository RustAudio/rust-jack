use jack_sys as j;
use libc;
use std;

/// A lock-free ringbuffer. The key attribute of a ringbuffer is that it can be
/// safely accessed by
/// two threads simultaneously, one reading from the buffer and the other
/// writing to it - without
/// using any synchronization or mutual exclusion primitives.  For this to work
/// correctly, there can
/// only be a single reader and a single writer thread. Their identities cannot
/// be interchanged.
///
/// # Example
/// ```
/// use jack::prelude as j;
/// let ringbuf = j::RingBuffer::new(1024).unwrap();
/// let (mut reader, mut writer) = ringbuf.into_reader_writer();
///
/// let buf = [0u8, 1, 2, 3];
/// let num = writer.write_buffer(&buf);
/// assert_eq!(num, buf.len());
///
/// // Potentially in a another thread:
/// let mut outbuf = [0u8; 8];
/// let num = reader.read_buffer(&mut outbuf);
/// ```
pub struct RingBuffer(*mut j::jack_ringbuffer_t);

impl RingBuffer {
    /// Allocates a ringbuffer of a specified size.
    pub fn new(size: usize) -> Result<Self, ()> {

        let insize = size as libc::size_t;
        let handle = unsafe { j::jack_ringbuffer_create(insize) };

        if handle.is_null() {
            return Err(());
        }

        Ok(RingBuffer(handle))
    }

    /// Lock a ringbuffer data block into memory.
    pub fn mlock(&mut self) {
        unsafe { j::jack_ringbuffer_mlock(self.0) };
    }

    /// Resets the ring buffer, making an empty buffer. Not thread safe.
    pub fn reset(&mut self) {
        unsafe { j::jack_ringbuffer_reset(self.0) };
    }

    /// Create a reader and writer, to use the ring buffer.
    pub fn into_reader_writer(self) -> (RingBufferReader, RingBufferWriter) {
        let handle = std::sync::Arc::new(self);

        unsafe {
            (
                RingBufferReader::new(handle.clone()),
                RingBufferWriter::new(handle),
            )
        }
    }

    /// Re-create the ring buffer object from reader and writer. useful if you
    /// need to call reset.
    /// The reader and the writer pair must have been created from the same
    /// RingBuffer object.  Not
    /// needed for deallocation, disposing of both reader and writer will
    /// deallocate buffer
    /// resources automatically.
    ///
    /// panics if the reader and the writer were created from different
    /// RingBuffer objects.
    pub fn from_reader_writer(r: RingBufferReader, w: RingBufferWriter) -> Self {
        let mut handle_r = r.into_handle();
        let handle_w = w.into_handle();

        if handle_r.0 != handle_w.0 {
            panic!("mismatching read and write handles!")
        }

        let handle = RingBuffer(handle_r.0);

        // make sure that get_mut works
        std::mem::drop(handle_w);
        let muthandle = std::sync::Arc::get_mut(&mut handle_r).unwrap();
        // make sure they don't close when they are dropped.
        muthandle.0 = std::ptr::null_mut();
        handle
    }
}


impl Drop for RingBuffer {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { j::jack_ringbuffer_free(self.0) };
        }
        self.0 = 0 as *mut j::jack_ringbuffer_t;
    }
}

/// Read end of the ring buffer. Can only be used from one thread (can be
/// different from the write thread).
pub struct RingBufferReader {
    ringbuffer_handle: std::sync::Arc<RingBuffer>,
}

unsafe impl Send for RingBufferReader {}
// impl !Sync for RingBufferReader{ }

/// Write end of the ring buffer. Can only be used from one thread (can be a
/// different from the read thread).
pub struct RingBufferWriter {
    ringbuffer_handle: std::sync::Arc<RingBuffer>,
}

unsafe impl Send for RingBufferWriter {}
// impl !Sync for RingBufferWriter{ }


impl RingBufferReader {
    unsafe fn new(handle: std::sync::Arc<RingBuffer>) -> Self {
        RingBufferReader { ringbuffer_handle: handle }
    }

    fn into_handle(self) -> std::sync::Arc<RingBuffer> {
        self.ringbuffer_handle
    }


    /// Fill a data structure with a description of the current readable data
    /// held in the
    /// ringbuffer. This description is returned in a two slices. Two slices
    /// are needed because the
    /// data to be read may be split across the end of the ringbuffer. The
    /// first slice represents
    /// the bytes ready to be read. If the second slice is not empty, it is the
    /// continuation of the
    /// data that ended in the first slices. For convenience, consider using
    /// peek_iter instead.
    pub fn get_vector<'a>(&'a self) -> (&'a [u8], &'a [u8]) {
        let mut vec = [
            j::jack_ringbuffer_data_t::default(),
            j::jack_ringbuffer_data_t::default(),
        ];
        let vecstart = &mut vec[0] as *mut j::jack_ringbuffer_data_t;

        unsafe { j::jack_ringbuffer_get_read_vector(self.ringbuffer_handle.0, vecstart) };


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

    /// Read data from the ringbuffer.
    /// Returns: the number of bytes read, which may range from 0 to buf.len().
    pub fn read_buffer(&mut self, buf: &mut [u8]) -> usize {
        if buf.len() == 0 {
            return 0;
        }

        let insize: libc::size_t = buf.len() as libc::size_t;
        let bufstart = &mut buf[0] as *mut _ as *mut libc::c_char;

        let read = unsafe { j::jack_ringbuffer_read(self.ringbuffer_handle.0, bufstart, insize) };
        read as usize
    }

    /// Read data from the ringbuffer. Opposed to read_buffer() this function
    /// does not move the read pointer.
    /// Thus it's a convenient way to inspect data in the ringbuffer in a
    /// continous fashion.
    /// The price is that the data is copied into a user provided buffer.
    /// For "raw" non-copy inspection of the data in the ringbuffer use
    /// get_vector() or peek_iter.
    /// Returns: the number of bytes read, which may range from 0 to buf.len()
    pub fn peek(&self, buf: &mut [u8]) -> usize {
        if buf.len() == 0 {
            return 0;
        }

        let insize: libc::size_t = buf.len() as libc::size_t;
        let bufstart = &mut buf[0] as *mut _ as *mut libc::c_char;

        let read = unsafe { j::jack_ringbuffer_peek(self.ringbuffer_handle.0, bufstart, insize) };
        read as usize
    }

    /// Advance the read pointer. use this after peek/peek_iter or get_vector
    /// to advance the buffer pointer.
    pub fn advance(&mut self, cnt: usize) {
        let incnt = cnt as libc::size_t;
        unsafe { j::jack_ringbuffer_read_advance(self.ringbuffer_handle.0, incnt) };
    }


    /// Return the number of bytes available for reading.
    pub fn space(&self) -> usize {
        unsafe { j::jack_ringbuffer_read_space(self.ringbuffer_handle.0) as usize }
    }

    /// Iterator that goes over all the data available to read.
    pub fn peek_iter<'a>(
        &'a self,
    ) -> std::iter::Chain<std::slice::Iter<'a, u8>, std::slice::Iter<'a, u8>> {

        let (view1, view2) = self.get_vector();

        view1.iter().chain(view2.iter())
    }
}

impl std::io::Read for RingBufferReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(self.read_buffer(buf))
    }
}

impl RingBufferWriter {
    unsafe fn new(handle: std::sync::Arc<RingBuffer>) -> Self {
        RingBufferWriter { ringbuffer_handle: handle }
    }

    fn into_handle(self) -> std::sync::Arc<RingBuffer> {
        self.ringbuffer_handle
    }

    /// Write data into the ringbuffer.
    /// Returns: The number of bytes written, which may range from 0 to
    /// buf.len()
    pub fn write_buffer(&mut self, buf: &[u8]) -> usize {
        if buf.len() == 0 {
            return 0;
        }

        let insize: libc::size_t = buf.len() as libc::size_t;
        let bufstart = &buf[0] as *const _ as *const libc::c_char;

        let read = unsafe { j::jack_ringbuffer_write(self.ringbuffer_handle.0, bufstart, insize) };
        read as usize
    }

    /// Advance the write pointer. use this after peek_iter or get_vector to
    /// advance the buffer pointer.
    pub fn advance(&mut self, cnt: usize) {
        let incnt = cnt as libc::size_t;
        unsafe { j::jack_ringbuffer_write_advance(self.ringbuffer_handle.0, incnt) };

    }

    /// Return the number of bytes available for writing.
    pub fn space(&mut self) -> usize {
        unsafe { j::jack_ringbuffer_write_space(self.ringbuffer_handle.0) as usize }
    }

    /// Return a pair of slices of the current writable space in the
    /// ringbuffer. two slices are
    /// needed because the space available for writing may be split across the
    /// end of the
    /// ringbuffer.  consider using peek_iter for convenience.
    pub fn get_vector<'a>(&'a mut self) -> (&'a mut [u8], &'a mut [u8]) {
        let mut vec = [
            j::jack_ringbuffer_data_t::default(),
            j::jack_ringbuffer_data_t::default(),
        ];
        let vecstart = &mut vec[0] as *mut j::jack_ringbuffer_data_t;

        unsafe { j::jack_ringbuffer_get_write_vector(self.ringbuffer_handle.0, vecstart) };


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
    pub fn peek_iter<'a>(
        &'a mut self,
    ) -> std::iter::Chain<std::slice::IterMut<'a, u8>, std::slice::IterMut<'a, u8>> {

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
