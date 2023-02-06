use crate::AudioIn;
use crate::AudioOut;
use crate::Client;
use crate::ClientOptions;
use crate::Port;
use crate::PortFlags;
use crate::PortSpec;
use crate::Unowned;

fn open_test_client(name: &str) -> Client {
    Client::new(name, ClientOptions::NO_START_SERVER).unwrap().0
}

fn open_client_with_port(client: &str, port: &str) -> (Client, Port<AudioIn>) {
    let c = open_test_client(client);
    let p = c.register_port(port, AudioIn::default()).unwrap();
    (c, p)
}

#[test]
fn port_can_be_cast_to_unowned() {
    let (_c, p) = open_client_with_port("port_cwpn", "the_port_name");
    let p_alt: Port<Unowned> = p.clone_unowned();
    assert_eq!(p.short_name(), p_alt.short_name());
    assert_eq!(p.name(), p_alt.name());
}

#[test]
fn port_created_with_proper_names() {
    let (_c, p) = open_client_with_port("port_cwpn", "the_port_name");
    assert_eq!(p.short_name().unwrap(), "the_port_name");
    assert_eq!(p.name().unwrap(), "port_cwpn:the_port_name");
}

#[test]
fn port_can_rename() {
    let client_name = "port_rename";
    let original_name = "port_to_rename";
    let new_name = "port_that_was_renamed";

    // initial port
    let (_c, mut p) = open_client_with_port(client_name, original_name);
    assert_eq!(p.name().unwrap(), format!("{client_name}:{original_name}"));
    assert_eq!(p.short_name().unwrap(), original_name);

    // renamed port
    p.set_name(new_name).unwrap();
    assert_eq!(p.name().unwrap(), format!("{client_name}:{new_name}"));
    assert_eq!(p.short_name().unwrap(), new_name);
}

#[test]
fn port_connected_count() {
    let c = open_test_client("port_connected_count");
    let pa = c.register_port("pa", AudioIn::default()).unwrap();
    let pb = c.register_port("pb", AudioOut::default()).unwrap();
    let pc = c.register_port("pc", AudioOut::default()).unwrap();
    let pd = c.register_port("pd", AudioOut::default()).unwrap();
    let c = c.activate_async((), ()).unwrap();
    c.as_client().connect_ports(&pb, &pa).unwrap();
    c.as_client().connect_ports(&pc, &pa).unwrap();
    assert_eq!(pa.connected_count().unwrap(), 2);
    assert_eq!(pb.connected_count().unwrap(), 1);
    assert_eq!(pc.connected_count().unwrap(), 1);
    assert_eq!(pd.connected_count().unwrap(), 0);
}

#[test]
fn port_knows_connections() {
    let c = open_test_client("port_knows_connections");
    let pa = c.register_port("pa", AudioIn::default()).unwrap();
    let pb = c.register_port("pb", AudioOut::default()).unwrap();
    let pc = c.register_port("pc", AudioOut::default()).unwrap();
    let pd = c.register_port("pd", AudioOut::default()).unwrap();
    let c = c.activate_async((), ()).unwrap();
    c.as_client().connect_ports(&pb, &pa).unwrap();
    c.as_client().connect_ports(&pc, &pa).unwrap();

    // pa
    assert!(pa.is_connected_to(&pb.name().unwrap()).unwrap());
    assert!(pa.is_connected_to(&pc.name().unwrap()).unwrap());
    assert!(!pa.is_connected_to(&pd.name().unwrap()).unwrap());

    // pb
    assert!(pb.is_connected_to(&pa.name().unwrap()).unwrap());
    assert!(!pb.is_connected_to(&pc.name().unwrap()).unwrap());
    assert!(!pb.is_connected_to(&pd.name().unwrap()).unwrap());

    // pc
    assert!(pc.is_connected_to(&pa.name().unwrap()).unwrap());
    assert!(!pc.is_connected_to(&pb.name().unwrap()).unwrap());
    assert!(!pc.is_connected_to(&pd.name().unwrap()).unwrap());

    // pd
    assert!(!pd.is_connected_to(&pa.name().unwrap()).unwrap());
    assert!(!pd.is_connected_to(&pb.name().unwrap()).unwrap());
    assert!(!pd.is_connected_to(&pc.name().unwrap()).unwrap());
}

#[test]
fn port_can_ensure_monitor() {
    let (_c, p) = open_client_with_port("port_can_ensure_monitor", "maybe_monitor");

    for should_monitor in [true, false].iter().cycle().take(10) {
        p.ensure_monitor(*should_monitor).unwrap();
        assert_eq!(p.is_monitoring_input().unwrap(), *should_monitor);
    }
}

#[test]
fn port_can_request_monitor() {
    let (_c, p) = open_client_with_port("port_can_ensure_monitor", "maybe_monitor");

    for should_monitor in [true, false].iter().cycle().take(10) {
        p.request_monitor(*should_monitor).unwrap();
        assert_eq!(p.is_monitoring_input().unwrap(), *should_monitor);
    }
}

#[test]
fn port_can_set_alias() {
    let (_c, mut p) = open_client_with_port("port_can_set_alias", "will_get_alias");

    // no alias
    assert!(p.aliases().unwrap().is_empty());

    // 1 alias
    p.set_alias("first_alias").unwrap();
    assert_eq!(p.aliases().unwrap(), vec!["first_alias".to_string()]);

    // 2 alias
    p.set_alias("second_alias").unwrap();
    assert_eq!(
        p.aliases().unwrap(),
        vec!["first_alias".to_string(), "second_alias".to_string()]
    );
}

#[test]
fn port_can_unset_alias() {
    let (_c, mut p) = open_client_with_port("port_can_unset_alias", "will_unset_alias");

    // set aliases
    p.set_alias("first_alias").unwrap();
    p.set_alias("second_alias").unwrap();
    assert_eq!(
        p.aliases().unwrap(),
        vec!["first_alias".to_string(), "second_alias".to_string()]
    );

    // unset alias
    p.unset_alias("first_alias").unwrap();
    assert_eq!(p.aliases().unwrap(), vec!["second_alias".to_string()]);
}

#[test]
fn port_unowned_no_port_type() {
    assert_eq!("", Unowned::default().jack_port_type());
}

#[test]
fn port_unowned_no_port_flags() {
    assert_eq!(PortFlags::empty(), Unowned::default().jack_flags());
}

#[test]
#[should_panic]
fn port_unowned_no_port_size() {
    Unowned::default().jack_buffer_size();
}

#[test]
fn port_debug_printing() {
    let (_c, mut p) = open_client_with_port("port_has_debug_string", "debug_info");
    p.set_alias("this_port_alias").unwrap();
    let got = format!("{p:?}");
    let parts = [
        ("name", "Ok(\"port_has_debug_string:debug_info\")"),
        ("connections", "0"),
        ("port_type", "Ok(\"32 bit float mono audio\")"),
        ("port_flags", "IS_INPUT"),
        ("aliases", "[\"this_port_alias\""),
    ];
    for &(k, v) in parts.iter() {
        let p = format!("{k}: {v}");
        assert!(got.contains(&p), "Expected {got} to contain \"{}\".", p);
    }
}
