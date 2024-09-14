#[test]
fn client_can_open() {
    let (client, status) =
        crate::Client::new("my new client", crate::ClientOptions::NO_START_SERVER).unwrap();
    assert_eq!(status, crate::ClientStatus::empty());
    assert_eq!(client.name(), "my new client");
    assert_ne!(client.sample_rate(), 0);
    assert_ne!(client.buffer_size(), 0);
    assert_ne!(client.uuid_string(), "");
    let cpu_load = client.cpu_load();
    assert!(cpu_load > 0.0, "client.cpu_load() = {}", cpu_load);
}

#[test]
fn time_is_montonically_increasing() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::empty()).unwrap();

    let t0 = client.time();
    let frames0 = client.frames_since_cycle_start();
    let frame_time0 = client.frame_time();

    std::thread::sleep(std::time::Duration::from_millis(50));
    assert_ne!(client.time(), t0);
    assert_ne!(client.frames_since_cycle_start(), frames0);
    assert_ne!(client.frame_time(), frame_time0);
}

#[test]
fn maybe_client_can_set_buffer_size() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::empty()).unwrap();
    let initial_buffer_size = client.buffer_size();
    if let Err(err) = client.set_buffer_size(initial_buffer_size * 2) {
        eprintln!("client does not support setting buffer size: {err}");
        return;
    }
    assert_eq!(client.buffer_size(), 2 * initial_buffer_size);
    client.set_buffer_size(initial_buffer_size).unwrap();
}

#[test]
fn client_uuid_are_unique() {
    let (client1, _) = crate::Client::new("", crate::ClientOptions::NO_START_SERVER).unwrap();
    let (client2, _) = crate::Client::new("", crate::ClientOptions::NO_START_SERVER).unwrap();
    assert_ne!(client1.uuid_string(), "");
    assert_ne!(client2.uuid_string(), "");
    assert_ne!(client1.uuid_string(), client2.uuid_string());
    #[cfg(feature = "metadata")]
    {
        assert_ne!(client1.uuid(), 0);
        assert_ne!(client2.uuid(), 0);
        assert_ne!(client1.uuid(), client2.uuid());
    }
}

#[test]
fn uuid_can_map_to_client_name() {
    let (client1, _) =
        crate::Client::new("uuid-client-1", crate::ClientOptions::NO_START_SERVER).unwrap();
    let (client2, _) =
        crate::Client::new("uuid-client-2", crate::ClientOptions::NO_START_SERVER).unwrap();

    assert_eq!(
        client1.name_by_uuid_str(&client1.uuid_string()).unwrap(),
        "uuid-client-1"
    );
    assert_eq!(
        client1.name_by_uuid_str(&client2.uuid_string()).unwrap(),
        "uuid-client-2"
    );
    #[cfg(feature = "metadata")]
    {
        assert_eq!(
            client1.name_by_uuid(client1.uuid()).unwrap(),
            "uuid-client-1"
        );
        assert_eq!(
            client1.name_by_uuid(client2.uuid()).unwrap(),
            "uuid-client-2"
        );
    }
}

#[test]
fn nonexistant_uuid_to_client_name_returns_none() {
    let (client1, _) = crate::Client::new("", crate::ClientOptions::NO_START_SERVER).unwrap();
    let (client2, _) =
        crate::Client::new("dropped-client", crate::ClientOptions::NO_START_SERVER).unwrap();
    let uuid_string = client2.uuid_string();
    #[cfg(feature = "metadata")]
    let uuid = client2.uuid();
    drop(client2);
    assert_eq!(client1.name_by_uuid_str(&uuid_string), None);
    #[cfg(feature = "metadata")]
    assert_eq!(client1.name_by_uuid(uuid), None);
}
