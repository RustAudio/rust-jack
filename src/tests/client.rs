use crate::tests::DEFAULT_TEST_CLIENT;

#[test]
fn client_can_open() {
    let (client, status) =
        crate::Client::new("my new client", crate::ClientOptions::default()).unwrap();
    assert_eq!(status, crate::ClientStatus::empty());
    assert_eq!(client.name(), "my new client");
    assert_ne!(client.sample_rate(), 0);
    assert_ne!(client.buffer_size(), 0);
    assert_ne!(client.uuid_string(), "");
    assert_ne!(client.uuid(), 0);
    let cpu_load = client.cpu_load();
    assert!(cpu_load > 0.0, "client.cpu_load() = {}", cpu_load);
}

#[test]
fn time_is_montonically_increasing() {
    let t0 = DEFAULT_TEST_CLIENT.time();
    let frames0 = DEFAULT_TEST_CLIENT.frames_since_cycle_start();
    let frame_time0 = DEFAULT_TEST_CLIENT.frame_time();

    std::thread::sleep(std::time::Duration::from_millis(50));
    assert_ne!(DEFAULT_TEST_CLIENT.time(), t0);
    assert_ne!(DEFAULT_TEST_CLIENT.frames_since_cycle_start(), frames0);
    assert_ne!(DEFAULT_TEST_CLIENT.frame_time(), frame_time0);
}

#[test]
fn maybe_client_can_set_buffer_size() {
    let initial_buffer_size = DEFAULT_TEST_CLIENT.buffer_size();
    if let Err(err) = DEFAULT_TEST_CLIENT.set_buffer_size(initial_buffer_size * 2) {
        eprintln!("client does not support setting buffer size: {err}");
        return;
    }
    assert_eq!(DEFAULT_TEST_CLIENT.buffer_size(), 2 * initial_buffer_size);
    DEFAULT_TEST_CLIENT
        .set_buffer_size(initial_buffer_size)
        .unwrap();
}

#[test]
fn client_uuid_are_unique() {
    let (client1, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    let (client2, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    assert_ne!(client1.uuid_string(), "");
    assert_ne!(client2.uuid_string(), "");
    assert_ne!(client1.uuid_string(), client2.uuid_string());
    assert_ne!(client1.uuid(), 0);
    assert_ne!(client2.uuid(), 0);
    assert_ne!(client1.uuid(), client2.uuid());
}

#[test]
fn uuid_can_map_to_client_name() {
    let (client1, _) =
        crate::Client::new("uuid-client-1", crate::ClientOptions::default()).unwrap();
    let (client2, _) =
        crate::Client::new("uuid-client-2", crate::ClientOptions::default()).unwrap();

    assert_eq!(
        client1.name_by_uuid_str(&client1.uuid_string()).unwrap(),
        "uuid-client-1"
    );
    assert_eq!(
        client1.name_by_uuid_str(&client2.uuid_string()).unwrap(),
        "uuid-client-2"
    );
    assert_eq!(
        client1.name_by_uuid(client1.uuid()).unwrap(),
        "uuid-client-1"
    );
    assert_eq!(
        client1.name_by_uuid(client2.uuid()).unwrap(),
        "uuid-client-2"
    );
}

#[test]
fn nonexistant_uuid_to_client_name_returns_none() {
    let (client, _) =
        crate::Client::new("dropped-client", crate::ClientOptions::default()).unwrap();
    let uuid_string = client.uuid_string();
    let uuid = client.uuid();
    drop(client);
    assert_eq!(DEFAULT_TEST_CLIENT.name_by_uuid_str(&uuid_string), None);
    assert_eq!(DEFAULT_TEST_CLIENT.name_by_uuid(uuid), None);
}
