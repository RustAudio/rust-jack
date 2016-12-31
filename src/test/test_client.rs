use super::super::*;

fn open_test_client(name: &str) -> (Client, ClientStatus) {
    default_sleep();
    Client::open(name, client_options::NO_START_SERVER).unwrap()
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
fn client_fails_to_open_with_large_name() {
    default_sleep();
    let name = (0..*CLIENT_NAME_SIZE + 1)
        .map(|_| "a")
        .collect::<Vec<&str>>()
        .join("_");
    assert_eq!(Client::open(&name, client_options::NO_START_SERVER).err(),
               Some(JackErr::ClientError(client_status::FAILURE | client_status::SERVER_ERROR)));
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
    c.activate(DummyHandler).unwrap();
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
fn client_can_deactivate() {
    let (c, _) = open_test_client("client_can_deactivate");
    let a = c.activate(DummyHandler).unwrap();
    a.deactivate().unwrap();
}

#[test]
fn client_knows_sample_rate() {
    let (c, _) = open_test_client("client_knows_sample_rate");
    // 44100 - As started by dummy_jack_server.sh
    assert_eq!(c.sample_rate(), 44100);
}

// TODO - improve test
#[test]
fn client_knows_cpu_load() {
    let (c, _) = open_test_client("client_knows_cpu_load");
    let _load = c.cpu_load();
}

// TODO - improve test
#[test]
fn client_can_estimate_frame_times() {
    let (c, _) = open_test_client("client_knows_cpu_load");
    c.frames_to_time(44100);
    c.time_to_frames(1000000);
}
