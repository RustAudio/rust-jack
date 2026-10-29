use std::time::Duration;

use crate::tests::{active_client, wait_for};

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
    // An idle server, such as the dummy backend, may report no load at all.
    let cpu_load = client.cpu_load();
    assert!(
        (0.0..=100.0).contains(&cpu_load),
        "client.cpu_load() = {}",
        cpu_load
    );
}

#[test]
fn time_is_montonically_increasing() {
    let ac = active_client();
    let client = ac.as_client();
    let t0 = client.time();
    let frame_time0 = client.frame_time();

    std::thread::sleep(Duration::from_millis(50));
    // `frames_since_cycle_start` is an offset within the current cycle, so it is not monotonic and
    // is not checked here.
    assert!(client.time() > t0);
    assert!(client.frame_time() > frame_time0);
}

#[test]
#[ignore = "unsupported on pipewire-jack; CI runs ignored tests with JACK2"]
fn maybe_client_can_set_buffer_size() {
    let ac = active_client();
    let client = ac.as_client();
    let initial_buffer_size = client.buffer_size();
    if let Err(err) = client.set_buffer_size(initial_buffer_size * 2) {
        eprintln!("client does not support setting buffer size: {err}");
        return;
    }
    // The server applies the change asynchronously.
    let changed = wait_for(Duration::from_secs(1), || {
        client.buffer_size() == 2 * initial_buffer_size
    });
    client.set_buffer_size(initial_buffer_size).unwrap();
    assert!(changed, "buffer size is still {}", client.buffer_size());
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
    let (observer, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    // The server may take a moment to forget the closed client.
    assert!(wait_for(Duration::from_secs(1), || {
        observer.name_by_uuid_str(&uuid_string).is_none() && observer.name_by_uuid(uuid).is_none()
    }));
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
fn error_messages_are_spelled_correctly() {
    assert_eq!(
        crate::Error::UnknownError { error_code: 3 }.to_string(),
        "unknown error with code 3"
    );
    assert_eq!(
        crate::Error::ClientPanicked.to_string(),
        "client notification or processor panicked"
    );
}

#[test]
fn client_options_support_common_traits() {
    let options = crate::ClientOptions::NO_START_SERVER | crate::ClientOptions::USE_EXACT_NAME;
    let copy = options;
    assert_eq!(copy.clone(), options);
    assert!(format!("{:?}", options).contains("USE_EXACT_NAME"));
}

#[test]
fn options_that_need_variadic_arguments_are_ignored() {
    // `jack_client_open` reads an extra variadic argument for each of these flags. `Client::new`
    // passes none, so they must be masked out instead of making JACK read garbage.
    for flag in [
        jack_sys::JackServerName,
        jack_sys::JackLoadName,
        jack_sys::JackLoadInit,
    ] {
        let options =
            crate::ClientOptions::from_bits_retain(crate::ClientOptions::default().bits() | flag);
        let (client, status) = crate::Client::new("variadic-options", options).unwrap();
        assert!(!status.contains(crate::ClientStatus::INVALID_OPTION));
        drop(client);
    }
}

#[test]
fn shutdown_can_be_implemented_as_a_safe_fn() {
    struct Handler;
    impl crate::NotificationHandler for Handler {
        fn shutdown(&mut self, _status: crate::ClientStatus, _reason: &str) {}
    }
    let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    let ac = client.activate_async(Handler, ()).unwrap();
    ac.deactivate().unwrap();
}

#[test]
fn set_name_renames_the_port() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    let mut port = client
        .register_port("before", crate::AudioIn::default())
        .unwrap();
    port.set_name("after").unwrap();
    assert!(
        port.name().unwrap().ends_with(":after"),
        "{:?}",
        port.name()
    );
}
