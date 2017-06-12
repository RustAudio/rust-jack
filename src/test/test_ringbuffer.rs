use prelude::*;

#[test]
fn ringbuffer_can_create() {
    let ringbuf = RingBuffer::new(1024);
    ringbuf.unwrap();
}

#[test]
fn ringbuffer_can_space() {
    const SIZE: usize = 1024;
    let ringbuf = RingBuffer::new(SIZE).unwrap();
    let (mut reader, mut writer) = ringbuf.into_reader_writer();

    assert_eq!(writer.space(), SIZE - 1);
    assert_eq!(reader.space(), 0);

    const ADVANCE: usize = 5;

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

    let buf = [0u8, 1, 2, 3];
    let num = writer.write_buffer(&buf);
    assert_eq!(num, buf.len());


    let mut outbuf = [0u8; 8];
    let num = reader.read_buffer(&mut outbuf);
    assert_eq!(num, buf.len());

    assert_eq!(outbuf[..num], buf[..]);

}

#[test]
fn ringbuffer_peek_write() {
    let ringbuf = RingBuffer::new(1024).unwrap();
    let (reader, mut writer) = ringbuf.into_reader_writer();

    let buf = [0u8, 1, 2, 3];
    writer.write_buffer(&buf);

    let data: Vec<u8> = reader.peek_iter().map(|x| *x).collect();

    assert_eq!(data.len(), buf.len());
    assert_eq!(data[..], buf[..]);

}

#[test]
fn ringbuffer_write_read_split() {
    const BUFSIZE: usize = 10;
    let ringbuf = RingBuffer::new(BUFSIZE).unwrap();
    let (mut reader, mut writer) = ringbuf.into_reader_writer();

    let buf = [0u8, 1, 2, 3];

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


    let data: Vec<u8> = reader.peek_iter().map(|x| *x).collect();

    assert_eq!(data.len(), buf.len());
    assert_eq!(data[..], buf[..]);

}

#[test]
fn ringbuffer_peek_read() {
    let ringbuf = RingBuffer::new(1024).unwrap();
    let (mut reader, mut writer) = ringbuf.into_reader_writer();


    let buf = [0u8, 1, 2, 3];
    for (item, bufitem) in writer.peek_iter().zip(buf.iter()) {
        *item = *bufitem;
    }

    writer.advance(buf.len());

    let mut outbuf = [0u8; 8];
    let num = reader.read_buffer(&mut outbuf);
    assert_eq!(num, buf.len());

    assert_eq!(outbuf[..num], buf[..]);

}
