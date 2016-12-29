use super::super::*;

pub struct DummyClient;
impl JackHandler for DummyClient {}

fn open_test_client(name: &str) -> Client {
    Client::open(name, client_options::NO_START_SERVER).unwrap().0
}

#[test]
fn simple_connection_by_port() {
    let mut client = open_test_client("simple_connections");

    // initialize ports
    let in_p = client.register_port("valid_in_p", AudioInSpec).unwrap();
    let out_p = client.register_port("valid_out_p", AudioOutSpec).unwrap();
    assert_eq!(in_p.connected_count(), 0);
    assert_eq!(out_p.connected_count(), 0);

    // start client
    let client = client.activate(DummyClient).unwrap();

    // connect them
    client.connect_ports_by_name("simple_connections:valid_out_p",
                               "simple_connections:valid_in_p")
        .unwrap();
}

#[test]
fn unowned_connection() {
    let mut owns = open_test_client("owned_ports");
    let no_owns = open_test_client("unowned_ports");

    // initialize ports
    let _in_p = owns.register_port("owned_in_p", AudioInSpec).unwrap();
    let _out_p = owns.register_port("owned_out_p", AudioOutSpec).unwrap();
    let unowned_in = no_owns.port_by_name("owned_ports:owned_in_p").unwrap();
    let unowned_out = no_owns.port_by_name("owned_ports:owned_out_p").unwrap();

    // start client
    let _owns = owns.activate(DummyClient).unwrap();

    // connect them
    no_owns.connect_ports(&unowned_out, &unowned_in)
        .unwrap();
}

#[test]
fn simple_connection_by_name() {
    let mut client = open_test_client("simple_name_connections");

    // initialize ports
    let in_p = client.register_port("valid_in_p", AudioInSpec).unwrap();
    let out_p = client.register_port("valid_out_p", AudioOutSpec).unwrap();
    assert_eq!(in_p.connected_count(), 0);
    assert_eq!(out_p.connected_count(), 0);

    // start client
    let client = client.activate(DummyClient).unwrap();

    // connect them
    client.connect_ports_by_name("simple_name_connections:valid_out_p",
                               "simple_name_connections:valid_in_p")
        .unwrap();
}

#[test]
fn inactive_client_fail() {
    let mut client = open_test_client("simple_connections");

    // initialize ports
    let in_p = client.register_port("valid_in_p", AudioInSpec).unwrap();
    let out_p = client.register_port("valid_out_p", AudioOutSpec).unwrap();
    assert_eq!(in_p.connected_count(), 0);
    assert_eq!(out_p.connected_count(), 0);

    // connect them
    assert_eq!(client.connect_ports(&out_p, &in_p),
               Err(JackErr::PortConnectionError(out_p.name().to_string(),
                                                in_p.name().to_string())));
}


#[test]
fn already_connected() {
    let mut client = open_test_client("already_connected");

    // initialize ports
    let in_p = client.register_port("conna", AudioInSpec).unwrap();
    let out_p = client.register_port("connb", AudioOutSpec).unwrap();

    // start client
    let client = client.activate(DummyClient).unwrap();

    // connect them twice
    client.connect_ports(&out_p, &in_p).unwrap();
    assert_eq!(client.connect_ports(&out_p, &in_p),
               Err(JackErr::PortAlreadyConnected(out_p.name().to_string(),
                                                 in_p.name().to_string())));
}



#[test]
fn nonexistant_port_connections() {
    let client = open_test_client("nonexistants");
    let client = client.activate(DummyClient).unwrap();
    assert_eq!(client.connect_ports_by_name("doesnt_exist", "also_no_exist"),
               Err(JackErr::PortConnectionError("doesnt_exist".to_string(),
                                                "also_no_exist".to_string())));
}

#[test]
fn connect_from_other_client_when_inactive() {
    // set up first client
    let mut client_a = open_test_client("client_a");
    let in_p = client_a.register_port("client_a_in", AudioInSpec).unwrap();
    let out_p = client_a.register_port("client_a_out", AudioOutSpec).unwrap();
    let _client_a = client_a.activate(DummyClient).unwrap();

    // connect using another client
    let client_b = open_test_client("client_b");
    client_b.connect_ports(&out_p, &in_p).unwrap();
}

#[test]
fn connect_cross_client() {
    // set up first client
    let mut client_x = open_test_client("client_x");
    let in_p = client_x.register_port("client_x_in", AudioInSpec).unwrap();
    let _client_x = client_x.activate(DummyClient).unwrap();

    // set up second client
    let mut client_y = open_test_client("client_y");
    let out_p = client_y.register_port("client_y_out", AudioOutSpec).unwrap();
    let client_y = client_y.activate(DummyClient).unwrap();

    // connect ports
    client_y.connect_ports(&out_p, &in_p).unwrap();
}
