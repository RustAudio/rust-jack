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
    assert!(cpu_load >= 0.0, "client.cpu_load() = {}", cpu_load);
}

#[test]
fn client_new_rejects_options_that_need_extra_arguments() {
    for flag in [
        crate::ClientOptions::SERVER_NAME,
        crate::ClientOptions::LOAD_NAME,
        crate::ClientOptions::LOAD_INIT,
        crate::ClientOptions::SESSION_ID,
    ] {
        let options = crate::ClientOptions::default() | flag;
        let err = crate::Client::new("invalid options client", options).err();
        assert_eq!(err, Some(crate::Error::InvalidClientOptions(options)));
    }
}

#[test]
fn client_can_open_with_server_name() {
    let (client, _) = crate::Client::new_with_server_name(
        "server name client",
        "default",
        crate::ClientOptions::default(),
    )
    .unwrap();
    assert_eq!(client.name(), "server name client");

    // Explicitly passing `SERVER_NAME` is redundant but allowed.
    crate::Client::new_with_server_name(
        "server name client 2",
        "default",
        crate::ClientOptions::default() | crate::ClientOptions::SERVER_NAME,
    )
    .unwrap();
}

#[test]
fn client_with_server_name_rejects_other_options_that_need_extra_arguments() {
    for flag in [
        crate::ClientOptions::LOAD_NAME,
        crate::ClientOptions::LOAD_INIT,
        crate::ClientOptions::SESSION_ID,
    ] {
        let options = crate::ClientOptions::default() | flag;
        let err =
            crate::Client::new_with_server_name("invalid options client", "default", options).err();
        assert_eq!(err, Some(crate::Error::InvalidClientOptions(options)));
    }
}

#[test]
fn time_is_montonically_increasing() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    let ac = client.activate_async((), ()).unwrap();
    let client = ac.as_client();
    let t0 = client.time();
    let frames0 = client.frames_since_cycle_start();
    let frame_time0 = client.frame_time();

    std::thread::sleep(std::time::Duration::from_millis(50));
    assert_ne!(client.time(), t0);
    assert_ne!(client.frames_since_cycle_start(), frames0);
    assert_ne!(client.frame_time(), frame_time0);
}

#[test]
#[ignore = "unsupported on pipewire-jack; CI runs ignored tests with JACK2"]
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

#[test]
fn unregistering_another_clients_port_is_an_unregistration_error() {
    let (owner, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    let (other, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    let port = owner.register_port("p", crate::AudioIn::default()).unwrap();
    // Some servers only publish ports of active clients.
    let _owner = owner.activate_async((), ()).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let foreign = other.port_by_name(&port.name().unwrap()).unwrap();
    assert_eq!(
        other.unregister_port(foreign),
        Err(crate::Error::PortUnregistrationError)
    );
}

#[test]
fn registering_an_unowned_port_is_an_error() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    assert_eq!(
        client.register_port("p", crate::Unowned).map(|_| ()),
        Err(crate::Error::PortRegistrationError("p".to_string()))
    );
}

#[test]
fn client_with_unknown_server_name_fails_to_open() {
    // Proves the server name actually reaches JACK, rather than being dropped.
    let err = crate::Client::new_with_server_name(
        "unknown server client",
        "rust-jack-nonexistent-server",
        crate::ClientOptions::NO_START_SERVER,
    )
    .err();
    assert!(
        matches!(err, Some(crate::Error::ClientError(_))),
        "expected ClientError, got {:?}",
        err
    );
}

/// Opens a client whose name is not valid UTF-8. `Client::new` only accepts `&str`, but other
/// processes on the server can use any bytes.
fn open_non_utf8_client(name: &[u8]) -> crate::Client {
    let name = std::ffi::CString::new(name).unwrap();
    let mut status = 0;
    let ptr = unsafe {
        jack_sys::jack_client_open(name.as_ptr(), jack_sys::JackNoStartServer, &mut status)
    };
    assert!(!ptr.is_null(), "jack_client_open failed: {status:#x}");
    unsafe { crate::Client::from_raw(ptr) }
}

#[test]
fn non_utf8_client_name_is_decoded_lossily() {
    let client = open_non_utf8_client(b"non-utf8-\xff");
    assert_eq!(client.name(), "non-utf8-\u{fffd}");
    assert_eq!(
        client.name_by_uuid(client.uuid()).as_deref(),
        Some("non-utf8-\u{fffd}")
    );
}

#[test]
fn non_utf8_client_registration_does_not_stop_processing() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    struct Registrations(Arc<Mutex<Vec<String>>>);
    impl crate::NotificationHandler for Registrations {
        fn client_registration(&mut self, _: &crate::Client, name: &str, is_registered: bool) {
            if is_registered {
                self.0.lock().unwrap().push(name.to_string());
            }
        }
    }

    let names = Arc::new(Mutex::new(Vec::new()));
    let cycles = Arc::new(AtomicUsize::new(0));
    let process = {
        let cycles = cycles.clone();
        crate::contrib::ClosureProcessHandler::new(move |_, _| {
            cycles.fetch_add(1, Ordering::Relaxed);
            crate::Control::Continue
        })
    };
    let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    let ac = client
        .activate_async(Registrations(names.clone()), process)
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));

    let _other = open_non_utf8_client(b"other-\xfe");
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert!(
        names.lock().unwrap().iter().any(|n| n == "other-\u{fffd}"),
        "{:?}",
        names.lock().unwrap()
    );

    let before = cycles.load(Ordering::Relaxed);
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert!(cycles.load(Ordering::Relaxed) > before);
    ac.deactivate().unwrap();
}
