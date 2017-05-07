use prelude::*;

#[test]
fn ringbuffer_can_create() {
    let ringbuf = RingBuffer::new(1024);
    ringbuf.unwrap();
}

#[test]
fn ringbuffer_can_space() {
    const SIZE : usize = 1024;
    let ringbuf = RingBuffer::new(SIZE).unwrap();
    let (mut reader, mut writer) = ringbuf.into_reader_writer();

    assert_eq!(writer.space(), SIZE-1);
    assert_eq!(reader.space(), 0);

    const ADVANCE : usize = 5;

    writer.advance(ADVANCE);

    assert_eq!(writer.space(), SIZE-1-ADVANCE);
    assert_eq!(reader.space(), ADVANCE);

    reader.advance(ADVANCE);
    assert_eq!(writer.space(), SIZE-1);
    assert_eq!(reader.space(), 0);

    
}



#[test]
fn ringbuffer_write_read() {
    let ringbuf = RingBuffer::new(1024).unwrap();
    let (mut reader, mut writer) = ringbuf.into_reader_writer();

    let buf = [0u8,1,2,3];
    let num = writer.write_buffer(&buf);
    assert_eq!(num, buf.len());


    let mut outbuf = [0u8;8];
    let num = reader.read_buffer(&mut outbuf);
    assert_eq!(num, buf.len());

    assert_eq!(outbuf[..num], buf[..]);

}

#[test]
fn ringbuffer_peak_write() {
    let ringbuf = RingBuffer::new(1024).unwrap();
    let (reader, mut writer) = ringbuf.into_reader_writer();

    let buf = [0u8,1,2,3];
    writer.write_buffer(&buf);

    let mut outbuf = [0u8;8];

    let mut num = 0;
    for (i, item) in reader.peek_iter().enumerate() {
        outbuf[i] = *item;
        num = i + 1;
    }
 
    assert_eq!(num, buf.len());
    assert_eq!(outbuf[..num], buf[..]);

}

#[test]
fn ringbuffer_peak_read() {
    let ringbuf = RingBuffer::new(1024).unwrap();
    let (mut reader, mut writer) = ringbuf.into_reader_writer();


    let buf = [0u8,1,2,3];
    for (i, item) in writer.peek_iter().take(buf.len()).enumerate() {
        *item = buf[i];
    }
  
    writer.advance(buf.len());
 
    let mut outbuf = [0u8;8];
    let num = reader.read_buffer(&mut outbuf);
    assert_eq!(num, buf.len());

    assert_eq!(outbuf[..num], buf[..]);

}

