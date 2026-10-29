//! The safe API must report bad input as an error instead of panicking.

use crate::{AudioIn, AudioOut, Client, ClientOptions, Error, PortFlags, Property};

fn is_invalid_string<T: std::fmt::Debug>(res: Result<T, Error>) -> bool {
    matches!(res, Err(Error::InvalidString(_)))
}

#[test]
fn strings_with_nul_bytes_are_rejected() {
    let (client, _) = Client::new("", ClientOptions::default()).unwrap();
    let mut port = client.register_port("ok", AudioOut::default()).unwrap();
    let port_name = port.name().unwrap();

    assert!(is_invalid_string(
        client.register_port("a\0b", AudioIn::default())
    ));
    assert!(is_invalid_string(
        client.connect_ports_by_name("a\0b", &port_name)
    ));
    assert!(is_invalid_string(
        client.connect_ports_by_name(&port_name, "a\0b")
    ));
    assert!(is_invalid_string(
        client.disconnect_ports_by_name("a\0b", &port_name)
    ));
    assert!(is_invalid_string(
        client.request_monitor_by_name("a\0b", true)
    ));
    assert!(is_invalid_string(
        client.load_internal_client("a\0b", "x", "")
    ));
    assert!(is_invalid_string(port.is_connected_to("a\0b")));
    assert!(is_invalid_string(port.set_name("a\0b")));
    assert!(is_invalid_string(port.set_alias("a\0b")));
    assert!(is_invalid_string(port.unset_alias("a\0b")));
    assert!(is_invalid_string(client.property_set(
        client.uuid(),
        "a\0b",
        &Property::new("v", None)
    )));
    assert!(is_invalid_string(client.property_set(
        client.uuid(),
        "k",
        &Property::new("a\0b", None)
    )));
    assert!(is_invalid_string(client.property_set(
        client.uuid(),
        "k",
        &Property::new("v", Some("a\0b".into()))
    )));
    assert!(is_invalid_string(
        client.property_remove(client.uuid(), "a\0b")
    ));

    assert!(client.port_by_name("a\0b").is_none());
    assert!(client
        .ports(Some("a\0b"), None, PortFlags::empty())
        .is_empty());
    assert!(client
        .ports(None, Some("a\0b"), PortFlags::empty())
        .is_empty());
    assert!(client.uuid_of_client_by_name("a\0b").is_none());
    assert!(client.name_by_uuid_str("a\0b").is_none());
    assert!(client.property_get(client.uuid(), "a\0b").is_none());
}

#[test]
fn registering_a_second_property_change_handler_is_an_error() {
    let (mut client, _) = Client::new("", ClientOptions::default()).unwrap();
    let handler = || crate::ClosurePropertyChangeHandler::new(|_| {});
    assert_eq!(client.register_property_change_handler(handler()), Ok(()));
    assert_eq!(
        client.register_property_change_handler(handler()),
        Err(Error::CallbackRegistrationError)
    );
}
