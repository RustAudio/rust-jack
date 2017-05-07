use libc;
use std;
use jack_sys as j;

pub struct RingBuffer(*mut j::jack_ringbuffer_t);

impl RingBuffer {
    pub fn new(size: usize) -> Result<Self, ()> {

        let insize = size as libc::size_t;
        let handle = unsafe { j::jack_ringbuffer_create(insize) };

        if handle.is_null() {
            return Err(());
        }

        Ok(RingBuffer(handle))
    }

    pub fn mlock(&mut self) {
        unsafe { j::jack_ringbuffer_mlock(self.0) };
    }

    pub fn reset(&mut self) {
        unsafe { j::jack_ringbuffer_reset(self.0) };
    }

    pub fn into_reader_writer(self) -> (RingBufferReader, RingBufferWriter) {
        let handle = std::sync::Arc::new(self);

        (RingBufferReader::new(handle.clone()), RingBufferWriter::new(handle))
    }

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

pub struct RingBufferReader {
    ringbuffer_handle: std::sync::Arc<RingBuffer>,
}

unsafe impl Send for RingBufferReader {}
// impl !Sync for RingBufferReader{ }

pub struct RingBufferWriter {
    ringbuffer_handle: std::sync::Arc<RingBuffer>,
}

unsafe impl Send for RingBufferWriter {}
// impl !Sync for RingBufferWriter{ }


impl RingBufferReader {
    fn new(handle: std::sync::Arc<RingBuffer>) -> Self {
        RingBufferReader { ringbuffer_handle: handle }
    }

    fn into_handle(self) -> std::sync::Arc<RingBuffer> {
        self.ringbuffer_handle
    }

    fn get_read_vector(&self) -> [j::jack_ringbuffer_data_t; 2] {
        let mut vec = [j::jack_ringbuffer_data_t::default(), j::jack_ringbuffer_data_t::default()];
        let vecstart = &mut vec[0] as *mut j::jack_ringbuffer_data_t;

        unsafe { j::jack_ringbuffer_get_read_vector(self.ringbuffer_handle.0, vecstart) };

        vec
    }

    pub fn read_buffer(&mut self, buf: &mut [u8]) -> usize {
        if buf.len() == 0 {
            return 0;
        }

        let insize: libc::size_t = buf.len() as libc::size_t;
        let bufstart = &mut buf[0] as *mut _ as *mut libc::c_char;

        let read = unsafe { j::jack_ringbuffer_read(self.ringbuffer_handle.0, bufstart, insize) };
        read as usize
    }

    pub fn peek(&self, buf: &mut [u8]) -> usize {
        if buf.len() == 0 {
            return 0;
        }

        let insize: libc::size_t = buf.len() as libc::size_t;
        let bufstart = &mut buf[0] as *mut _ as *mut libc::c_char;

        let read = unsafe { j::jack_ringbuffer_peek(self.ringbuffer_handle.0, bufstart, insize) };
        read as usize
    }

    pub fn advance(&mut self, cnt: usize) {
        let incnt = cnt as libc::size_t;
        unsafe { j::jack_ringbuffer_read_advance(self.ringbuffer_handle.0, incnt) };
    }


    pub fn space(&self) -> usize {
        unsafe { j::jack_ringbuffer_read_space(self.ringbuffer_handle.0) as usize }
    }

    // TODO: change this impl Iter when it is stable.
    pub fn peek_iter<'a>
        (&'a self)
         -> std::iter::Chain<std::slice::Iter<'a, u8>, std::slice::Iter<'a, u8>> {

        let views = self.get_read_vector();

        let view1 = views[0];
        let view2 = views[1];

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

        view1.iter().chain(view2.iter())
    }
}

impl std::io::Read for RingBufferReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(self.read_buffer(buf))
    }
}

impl RingBufferWriter {
    fn new(handle: std::sync::Arc<RingBuffer>) -> Self {
        RingBufferWriter { ringbuffer_handle: handle }
    }

    fn into_handle(self) -> std::sync::Arc<RingBuffer> {
        self.ringbuffer_handle
    }

    pub fn write_buffer(&mut self, buf: &[u8]) -> usize {
        if buf.len() == 0 {
            return 0;
        }

        let insize: libc::size_t = buf.len() as libc::size_t;
        let bufstart = &buf[0] as *const _ as *const libc::c_char;

        let read = unsafe { j::jack_ringbuffer_write(self.ringbuffer_handle.0, bufstart, insize) };
        read as usize
    }

    pub fn advance(&mut self, cnt: usize) {
        let incnt = cnt as libc::size_t;
        unsafe { j::jack_ringbuffer_write_advance(self.ringbuffer_handle.0, incnt) };

    }
    pub fn space(&mut self) -> usize {
        unsafe { j::jack_ringbuffer_write_space(self.ringbuffer_handle.0) as usize }
    }


    fn get_write_vector(&self) -> [j::jack_ringbuffer_data_t; 2] {
        let mut vec = [j::jack_ringbuffer_data_t::default(), j::jack_ringbuffer_data_t::default()];
        let vecstart = &mut vec[0] as *mut j::jack_ringbuffer_data_t;

        unsafe { j::jack_ringbuffer_get_write_vector(self.ringbuffer_handle.0, vecstart) };

        vec
    }

    pub fn peek_iter<'a>
        (&'a mut self)
         -> std::iter::Chain<std::slice::IterMut<'a, u8>, std::slice::IterMut<'a, u8>> {
        //        RingBufferWriterIter::new(self)

        let views = self.get_write_vector();

        let view1 = views[0];
        let view2 = views[1];

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
