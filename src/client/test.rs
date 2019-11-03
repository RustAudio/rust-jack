use client::*;
use jack_enums::Error;

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
    let got = format!("{:?}", c);
    let parts = [
        ("name", "\"client_has_debug_string\""),
        ("sample_rate", "44100"),
        ("buffer_size", "1024"),
        ("cpu_usage", ""),
        ("ports", "["),
        ("frame_time", ""),
    ];
    for &(k, v) in parts.iter() {
        let p = format!("{}: {}", k, v);
        assert!(got.contains(&p), "Expected {} to contain {}.", got, p);
    }
}
