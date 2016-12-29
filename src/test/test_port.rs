use super::super::*;

fn open_test_client(name: &str) -> Client {
    Client::open(name, client_options::NO_START_SERVER).unwrap().0
}

#[test]
fn rename() {
    let client_name = "port_rename";
    let original_name = "port_to_rename";
    let new_name = "port_that_was_renamed";

    // initial port
    let mut client = open_test_client(client_name);
    let mut p = client.register_port(original_name, AudioInSpec).unwrap();
    assert_eq!(p.name(), format!("{}:{}", client_name, original_name));
    assert_eq!(p.short_name(), original_name);

    // renamed port
    p.set_name(new_name).unwrap();
    assert_eq!(p.name(), format!("{}:{}", client_name, new_name));
    assert_eq!(p.short_name(), new_name);
}

#[test]
fn unregister() {
    let mut client = open_test_client("unregister_port");
    let p = client.register_port("to_unregister", AudioInSpec).unwrap();
    p.unregister().unwrap();
}

#[test]
#[should_panic(expected = "not yet implemented")]
fn aliases() {
    let mut client = open_test_client("client_uuid");
    let p = client.register_port("has_uuid", AudioInSpec).unwrap();
    assert!(p.aliases().is_empty());
}

#[test]
fn port_has_uuid() {
    let mut client = open_test_client("client_uuid");
    let a = client.register_port("has_uuid", AudioInSpec).unwrap();
    let b = client.register_port("also_has_uuid", AudioOutSpec).unwrap();
    assert!(a.uuid() != b.uuid());
}

#[test]
fn get_port_by_name() {
    let mut client = open_test_client("client_with_port_names");
    let a = client.register_port("has_name", AudioInSpec).unwrap();
    let b = client.port_by_name("client_with_port_names:has_name").unwrap();
    assert_eq!(a.name(), b.name());
}

#[test]
fn cannot_find_nonexistant_port() {
    let client = open_test_client("client_with_no_port_names");
    let p = client.port_by_name("client_with_no_port_names:dont_exist");
    assert!(p.is_none());
}

#[test]
fn can_classify_as_mine() {
    let mut mines = open_test_client("its_my_port");
    let not_mines = open_test_client("not_my_port");

    // initialize ports
    let p = mines.register_port("i_belong", AudioInSpec).unwrap();

    // classify
    assert!(mines.is_mine(&p));
    assert!(!not_mines.is_mine(&p));
}

#[test]
fn can_find_ports() {
    let mut client = open_test_client("will_find_ports");

    // initialize ports
    let _p = client.register_port("i_exist", AudioInSpec).unwrap();

    let found = client.ports(None, None, PortFlags::empty());

    assert!(found.contains(&"will_find_ports:i_exist".to_string()),
            "{:?}",
            &found);
}
