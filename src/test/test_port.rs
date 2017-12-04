use prelude::*;

fn open_test_client(name: &str) -> Client {
    Client::new(name, ClientOptions::NO_START_SERVER).unwrap().0
}

fn open_client_with_port(client: &str, port: &str) -> (Client, Port<AudioInSpec>) {
    let c = open_test_client(client);
    let p = c.register_port(port, AudioInSpec::default()).unwrap();
    (c, p)
}

#[test]
fn port_can_be_cast_to_unowned() {
    let (_c, p) = open_client_with_port("port_cwpn", "the_port_name");
    let p_alt: UnownedPort = p.clone_unowned();
    assert_eq!(p.short_name(), p_alt.short_name());
    assert_eq!(p.name(), p_alt.name());
}

#[test]
fn port_created_with_proper_names() {
    let (_c, p) = open_client_with_port("port_cwpn", "the_port_name");
    assert_eq!(p.short_name(), "the_port_name");
    assert_eq!(p.name(), "port_cwpn:the_port_name");
}

#[test]
fn port_can_rename() {
    let client_name = "port_rename";
    let original_name = "port_to_rename";
    let new_name = "port_that_was_renamed";

    // initial port
    let (_c, mut p) = open_client_with_port(client_name, original_name);
    assert_eq!(p.name(), format!("{}:{}", client_name, original_name));
    assert_eq!(p.short_name(), original_name);

    // renamed port
    p.set_name(new_name).unwrap();
    assert_eq!(p.name(), format!("{}:{}", client_name, new_name));
    assert_eq!(p.short_name(), new_name);
}

#[test]
fn port_connected_count() {
    let c = open_test_client("port_connected_count");
    let pa = c.register_port("pa", AudioInSpec::default()).unwrap();
    let pb = c.register_port("pb", AudioOutSpec::default()).unwrap();
    let pc = c.register_port("pc", AudioOutSpec::default()).unwrap();
    let pd = c.register_port("pd", AudioOutSpec::default()).unwrap();
    let c = AsyncClient::new(c, (), ()).unwrap();
    c.connect_ports(&pb, &pa).unwrap();
    c.connect_ports(&pc, &pa).unwrap();
    assert_eq!(pa.connected_count(), 2);
    assert_eq!(pb.connected_count(), 1);
    assert_eq!(pc.connected_count(), 1);
    assert_eq!(pd.connected_count(), 0);
}

#[test]
fn port_knows_connections() {
    let c = open_test_client("port_knows_connections");
    let pa = c.register_port("pa", AudioInSpec::default()).unwrap();
    let pb = c.register_port("pb", AudioOutSpec::default()).unwrap();
    let pc = c.register_port("pc", AudioOutSpec::default()).unwrap();
    let pd = c.register_port("pd", AudioOutSpec::default()).unwrap();
    let c = AsyncClient::new(c, (), ()).unwrap();
    c.connect_ports(&pb, &pa).unwrap();
    c.connect_ports(&pc, &pa).unwrap();

    // pa
    assert!(pa.is_connected_to(pb.name()));
    assert!(pa.is_connected_to(pc.name()));
    assert!(!pa.is_connected_to(pd.name()));

    // pb
    assert!(pb.is_connected_to(pa.name()));
    assert!(!pb.is_connected_to(pc.name()));
    assert!(!pb.is_connected_to(pd.name()));

    // pc
    assert!(pc.is_connected_to(pa.name()));
    assert!(!pc.is_connected_to(pb.name()));
    assert!(!pc.is_connected_to(pd.name()));

    // pd
    assert!(!pd.is_connected_to(pa.name()));
    assert!(!pd.is_connected_to(pb.name()));
    assert!(!pd.is_connected_to(pc.name()));
}

#[test]
fn port_can_ensure_monitor() {
    let (_c, p) = open_client_with_port("port_can_ensure_monitor", "maybe_monitor");

    for should_monitor in [true, false].into_iter().cycle().take(10) {
        p.ensure_monitor(should_monitor.clone()).unwrap();
        assert_eq!(p.is_monitoring_input(), should_monitor.clone());
    }
}

#[test]
fn port_can_request_monitor() {
    let (_c, p) = open_client_with_port("port_can_ensure_monitor", "maybe_monitor");

    for should_monitor in [true, false].into_iter().cycle().take(10) {
        p.request_monitor(should_monitor.clone()).unwrap();
        assert_eq!(p.is_monitoring_input(), should_monitor.clone());
    }
}


#[test]
fn port_can_set_alias() {
    let (_c, mut p) = open_client_with_port("port_can_set_alias", "will_get_alias");

    // no alias
    assert!(p.aliases().is_empty());

    // 1 alias
    p.set_alias("first_alias").unwrap();
    assert_eq!(p.aliases(), vec!["first_alias".to_string()]);

    // 2 alias
    p.set_alias("second_alias").unwrap();
    assert_eq!(
        p.aliases(),
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
        p.aliases(),
        vec!["first_alias".to_string(), "second_alias".to_string()]
    );

    // unset alias
    p.unset_alias("first_alias").unwrap();
    assert_eq!(p.aliases(), vec!["second_alias".to_string()]);
}

#[test]
#[should_panic]
fn port_unowned_no_port_type() {
    Unowned::default().jack_port_type();
}

#[test]
#[should_panic]
fn port_unowned_no_port_flags() {
    Unowned::default().jack_flags();
}

#[test]
#[should_panic]
fn port_unowned_no_port_size() {
    Unowned::default().jack_buffer_size();
}
