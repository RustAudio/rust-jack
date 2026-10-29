use crate::RingBuffer;

#[test]
fn ringbuffer_new_creates_new_ringbuffer() {
    RingBuffer::new(1024).unwrap();
}

#[test]
fn advancing_transfers_space_from_writer_to_reader() {
    let ringbuf = RingBuffer::new(1024).unwrap();
    let (reader, mut writer) = ringbuf.into_reader_writer();

    assert_eq!(writer.space(), 1023);
    assert_eq!(reader.space(), 0);

    writer.advance(23);
    assert_eq!(writer.space(), 1000);
    assert_eq!(reader.space(), 23);
}

#[test]
fn writing_to_writer_sends_to_reader() {
    let ringbuf = RingBuffer::new(1024).unwrap();
    let (mut reader, mut writer) = ringbuf.into_reader_writer();

    assert_eq!(writer.write_buffer(&[0, 1, 2, 3]), 4);

    let mut tmp_buffer = [0_u8; 8];
    assert_eq!(reader.read_slice(&mut tmp_buffer), &[0, 1, 2, 3]);
}

#[test]
fn written_bytes_can_be_peaked() {
    let ringbuf = RingBuffer::new(1024).unwrap();
    let (reader, mut writer) = ringbuf.into_reader_writer();

    writer.write_buffer(&[0, 1, 2, 3]);
    assert_eq!(
        reader.peek_iter().copied().collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
}

#[test]
fn advancing_and_writing_shifts_vector() {
    let ringbuf = RingBuffer::new(8).unwrap();
    let (mut reader, mut writer) = ringbuf.into_reader_writer();

    assert_eq!(writer.get_vector().0.len(), 7);
    assert_eq!(writer.get_vector().1.len(), 0);
    assert_eq!(reader.get_vector().0.len(), 0);
    assert_eq!(reader.get_vector().1.len(), 0);

    writer.advance(3);
    assert_eq!(writer.get_vector().0.len(), 4);
    assert_eq!(writer.get_vector().1.len(), 0);
    reader.advance(3);
    assert_eq!(reader.get_vector().0.len(), 0);
    assert_eq!(reader.get_vector().1.len(), 0);

    assert_eq!(writer.write_buffer(&[0, 1, 2]), 3);
    assert_eq!(reader.get_vector().0.len(), 3);
    assert_eq!(reader.get_vector().1.len(), 0);
    assert_eq!(reader.peek_iter().copied().collect::<Vec<_>>(), &[0, 1, 2]);
}

#[test]
fn writing_and_advancing_produces_data_on_reader() {
    let ringbuf = RingBuffer::new(1024).unwrap();
    let (mut reader, mut writer) = ringbuf.into_reader_writer();
    for (item, bufitem) in writer.peek_iter().zip([0, 1, 2, 3]) {
        *item = bufitem;
    }
    assert_eq!(reader.read_slice(&mut [0; 8]), &[]);
    writer.advance(4);
    assert_eq!(reader.read_slice(&mut [0; 8]), &[0, 1, 2, 3]);
}

#[test]
fn reading_and_writing_from_separate_threads_is_ok() {
    let ringbuf = RingBuffer::new(1024).unwrap();
    let (mut reader, mut writer) = ringbuf.into_reader_writer();

    std::thread::spawn(move || {
        for (item, bufitem) in writer.peek_iter().zip([0, 1, 2, 3]) {
            *item = bufitem;
        }
        writer.advance(4);
    })
    .join()
    .unwrap();
    assert_eq!(reader.read_slice(&mut [0; 8]), &[0, 1, 2, 3]);
}

fn freed_count() -> usize {
    crate::ringbuffer::FREED_COUNT.with(|c| c.get())
}

#[test]
fn dropping_reader_then_writer_frees_ringbuffer() {
    let before = freed_count();
    let (reader, writer) = RingBuffer::new(1024).unwrap().into_reader_writer();
    drop(reader);
    assert_eq!(freed_count(), before);
    drop(writer);
    assert_eq!(freed_count(), before + 1);
}

#[test]
fn dropping_writer_then_reader_frees_ringbuffer() {
    let before = freed_count();
    let (reader, writer) = RingBuffer::new(1024).unwrap().into_reader_writer();
    drop(writer);
    assert_eq!(freed_count(), before);
    drop(reader);
    assert_eq!(freed_count(), before + 1);
}

#[test]
fn dropping_halves_on_different_threads_frees_ringbuffer_once() {
    // The count is per thread, so check that exactly one of the two threads freed it.
    let (reader, writer) = RingBuffer::new(1024).unwrap().into_reader_writer();
    let a = std::thread::spawn(move || {
        drop(reader);
        freed_count()
    });
    let b = std::thread::spawn(move || {
        drop(writer);
        freed_count()
    });
    assert_eq!(a.join().unwrap() + b.join().unwrap(), 1);
}

#[test]
fn rejoined_ringbuffer_is_freed_once() {
    let before = freed_count();
    let (reader, writer) = RingBuffer::new(1024).unwrap().into_reader_writer();
    let ringbuf = RingBuffer::from_reader_writer(reader, writer);
    assert_eq!(freed_count(), before);
    drop(ringbuf);
    assert_eq!(freed_count(), before + 1);
}
