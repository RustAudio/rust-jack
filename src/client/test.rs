use crate::client::*;
use crate::jack_enums::Error;
use crate::{ClosureProcessHandler, Control, RingBuffer};

fn open_test_client(name: &str) -> (Client, ClientStatus) {
    Client::new(name, ClientOptions::NO_START_SERVER).unwrap()
}

#[test]
fn client_valid_client_name_size() {
    assert!(*CLIENT_NAME_SIZE > 0);
}

#[test]
fn client_can_open() {
    open_test_client("client_can_open");
}

#[test]
#[should_panic]
fn client_fails_to_open_with_large_name() {
    let name = (0..=*CLIENT_NAME_SIZE)
        .map(|_| "a")
        .collect::<Vec<&str>>()
        .join("_");
    Client::new(&name, ClientOptions::NO_START_SERVER).unwrap();
    // fails on travis, switched to should_panic for a catch all
    // assert_eq!(Client::new(&name, ClientOptions::NO_START_SERVER).err(),
    // Some(Error::ClientError(client_status::FAILURE |
    // client_status::SERVER_ERROR)));
}

#[test]
fn client_can_be_named() {
    let name = "client_can_be_named";
    let (c, _) = open_test_client(name);
    assert_eq!(c.name(), name);
}

#[test]
fn client_can_activate() {
    let (c, _) = open_test_client("client_can_activate");
    let _ac = c.activate_async((), ()).unwrap();
}

#[test]
fn client_can_set_buffer_size() {
    let (c, _) = open_test_client("client_can_set_buffer_size");
    let initial_size = c.buffer_size();
    let new_size = 2 * initial_size;
    c.set_buffer_size(new_size).unwrap();
    assert_eq!(c.buffer_size(), new_size);
    c.set_buffer_size(initial_size).unwrap();
    assert_eq!(c.buffer_size(), initial_size);
}

#[test]
fn client_detects_bad_buffer_size() {
    let (c, _) = open_test_client("client_detects_bad_buffer_size");
    let initial_size = c.buffer_size();
    assert_eq!(c.set_buffer_size(0), Err(Error::SetBufferSizeError));
    c.set_buffer_size(initial_size).unwrap();
    assert_eq!(c.buffer_size(), initial_size);
}

#[test]
fn client_can_deactivate() {
    let (c, _) = open_test_client("client_can_deactivate");
    let a = c.activate_async((), ()).unwrap();
    a.deactivate().unwrap();
}

#[test]
fn client_knows_buffer_size() {
    let (c, _) = open_test_client("client_knows_buffer_size");
    // 1024 - As started by dummy_jack_server.sh
    assert_eq!(c.buffer_size(), 1024);
}

#[test]
fn client_knows_sample_rate() {
    let (c, _) = open_test_client("client_knows_sample_rate");
    // 44100 - As started by dummy_jack_server.sh
    assert_eq!(c.sample_rate(), 44100);
}

#[test]
fn client_knows_cpu_load() {
    let (c, _) = open_test_client("client_knows_cpu_load");
    let _load = c.cpu_load();
}

#[test]
fn client_can_estimate_frame_times() {
    let (c, _) = open_test_client("client_knows_frame_times");
    let current_frame_time = c.frame_time();
    let time = c.frames_to_time(44_100);
    let frames = c.time_to_frames(1_000_000);
    assert!(current_frame_time > 0);
    assert!(time > 0);
    assert!(frames > 0);
}

#[test]
fn client_debug_printing() {
    let (c, _) = open_test_client("client_has_debug_string");
    let got = format!("{c:?}");
    assert_ne!("", got);
}

#[test]
fn client_can_use_ringbuffer() {
    let (c, _) = open_test_client("client_can_use_ringbuffer");

    let ringbuf = RingBuffer::new(1024).unwrap();
    let (mut reader, mut writer) = ringbuf.into_reader_writer();

    let buf = [0_u8, 1, 2, 3];
    let mut sent = false;
    let _a = c
        .activate_async(
            (),
            ClosureProcessHandler::new(move |_, _| {
                if !sent {
                    for (item, bufitem) in writer.peek_iter().zip(buf.iter()) {
                        *item = *bufitem;
                    }

                    writer.advance(buf.len());
                    sent = true;
                }
                Control::Continue
            }),
        )
        .unwrap();

    // spin until realtime closure has been run
    while reader.space() == 0 {}

    let mut outbuf = [0_u8; 8];
    let num = reader.read_buffer(&mut outbuf);
    assert_eq!(num, buf.len());

    assert_eq!(outbuf[..num], buf[..]);
}

#[test]
fn client_uuid() {
    let (c1, _) = open_test_client("uuidtest-client1");
    let (c2, _) = open_test_client("uuidtest-client2");

    let uuid1s = c1.uuid_string();
    let uuid2s = c2.uuid_string();
    assert_ne!(uuid1s, uuid2s);

    assert_eq!(
        c1.name_by_uuid_str(&uuid1s),
        Some("uuidtest-client1".to_string())
    );
    assert_eq!(
        c2.name_by_uuid_str(&uuid1s),
        Some("uuidtest-client1".to_string())
    );

    assert_eq!(
        c1.name_by_uuid_str(&uuid2s),
        Some("uuidtest-client2".to_string())
    );
    assert_eq!(
        c2.name_by_uuid_str(&uuid2s),
        Some("uuidtest-client2".to_string())
    );

    //create and then dealloc a client, get the uuid.
    let uuid3s = {
        let (c3, _) = open_test_client("uuidtest-client3");
        c3.uuid_string()
    };
    assert_eq!(c1.name_by_uuid_str(&uuid3s), None);
    assert_eq!(c2.name_by_uuid_str(&uuid3s), None);
}

#[cfg(feature = "metadata")]
#[test]
fn client_numeric_uuid() {
    let (c1, _) = open_test_client("numeric-uuid-client1");
    let (c2, _) = open_test_client("numeric-uuid-client2");

    let ac1 = c1.activate_async((), ()).unwrap();
    let ac2 = c2.activate_async((), ()).unwrap();

    let c1 = ac1.as_client();
    let c2 = ac2.as_client();

    let uuid1 = c1.uuid();
    let uuid2 = c2.uuid();
    assert_ne!(uuid1, uuid2);
    assert_ne!(0, uuid1);
    assert_ne!(0, uuid2);

    let uuid1s = c1.uuid_string();
    let uuid2s = c2.uuid_string();
    assert_ne!(uuid1s, uuid2s);

    assert_eq!(c1.name_by_uuid(0), None);
    assert_eq!(c2.name_by_uuid(0), None);

    assert_eq!(
        c1.name_by_uuid(uuid1),
        Some("numeric-uuid-client1".to_string())
    );
    assert_eq!(
        c2.name_by_uuid(uuid1),
        Some("numeric-uuid-client1".to_string())
    );
    assert_eq!(
        c1.name_by_uuid_str(&uuid1s),
        Some("numeric-uuid-client1".to_string())
    );
    assert_eq!(
        c2.name_by_uuid_str(&uuid1s),
        Some("numeric-uuid-client1".to_string())
    );

    assert_eq!(
        c1.name_by_uuid(uuid2),
        Some("numeric-uuid-client2".to_string())
    );
    assert_eq!(
        c2.name_by_uuid(uuid2),
        Some("numeric-uuid-client2".to_string())
    );
    assert_eq!(
        c1.name_by_uuid_str(&uuid2s),
        Some("numeric-uuid-client2".to_string())
    );
    assert_eq!(
        c2.name_by_uuid_str(&uuid2s),
        Some("numeric-uuid-client2".to_string())
    );

    //create and then dealloc a client, get the uuid.
    let uuid3 = {
        let (c3, _) = open_test_client("numeric-uuid-client3");
        c3.uuid()
    };
    assert_eq!(c1.name_by_uuid(uuid3), None);
    assert_eq!(c2.name_by_uuid(uuid3), None);
}
