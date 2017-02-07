use prelude::*;
use jack_utils::*;

use std::sync::Mutex;
use std::sync::mpsc;
use std::time;

fn open_test_client(name: &str) -> Client {
    Client::open(name, client_options::NO_START_SERVER).unwrap().0
}

#[test]
fn client_port_can_register_port() {
    let c = open_test_client("cp_can_register_port");
    c.register_port("cpcrp_a", AudioInSpec::default()).unwrap();
}

#[test]
fn client_port_register_port_enforces_unique_names() {
    let pname = "cprpeun_a";
    let c = open_test_client("cp_can_register_port");
    c.register_port(pname, AudioInSpec::default()).unwrap();
    assert_eq!(c.register_port(pname, AudioInSpec::default()).err(),
               Some(JackErr::PortRegistrationError(pname.to_string())));
}

#[test]
fn client_port_register_port_enforces_name_length() {
    let c = open_test_client("cp_can_register_port");
    let pname = (0..*PORT_NAME_SIZE + 1)
        .map(|_| "a")
        .collect::<Vec<&str>>()
        .join("_");
    assert_eq!(c.register_port(&pname, AudioInSpec::default()).err(),
               Some(JackErr::PortRegistrationError(pname.to_string())));
}

#[test]
fn client_port_can_request_monitor_by_name() {
    let c = open_test_client("cp_can_request_monitor_by_name");
    let p = c.register_port("cpcrmbn_a", AudioInSpec::default()).unwrap();
    c.request_monitor_by_name(p.name(), true).unwrap();
    c.request_monitor_by_name(p.name(), false).unwrap();
}

#[test]
fn client_port_can_get_port_by_name() {
    let c = open_test_client("cp_can_get_port_by_name");
    let p = c.register_port("named_port", AudioInSpec::default()).unwrap();
    let _p = c.port_by_name(p.name()).unwrap();
}

pub struct PortIdHandler {
    pub reg_tx: Mutex<mpsc::SyncSender<JackPortId>>,
}

impl JackHandler for PortIdHandler {
    fn port_registration(&self, _: &Client, pid: JackPortId, is_registered: bool) {
        match is_registered {
            true => self.reg_tx.lock().unwrap().send(pid).unwrap(),
            _ => (),
        }
    }
}

#[test]
fn client_port_can_get_port_by_id() {
    let (client_name, port_name) = ("cp_can_get_port_by_id", "cp_registered_port_name");

    // Create handler
    let (reg_tx, reg_rx) = mpsc::sync_channel(100);
    let h = PortIdHandler { reg_tx: Mutex::new(reg_tx) };

    // Open and activate client
    let c = open_test_client(client_name);
    let ac = ActiveClient::new(c, h).unwrap();

    // Register port
    let _pa = ac.register_port(port_name, AudioInSpec::default()).unwrap();

    // Get by id
    let pa_unowned = ac.port_by_id(reg_rx.recv_timeout(time::Duration::from_secs(1)).unwrap())
        .unwrap();
    assert_eq!(pa_unowned.name(), format!("{}:{}", client_name, port_name));
}

#[test]
fn client_port_fails_to_nonexistant_port() {
    let c = open_test_client("cp_can_request_monitor_by_name");
    let p = c.register_port("cpcrmbn_a", AudioInSpec::default()).unwrap();
    let _p = c.port_by_name(p.name()).unwrap();

}

#[test]
fn client_port_recognizes_my_ports() {
    let ca = open_test_client("cp_cprmp_ca");
    let cb = open_test_client("cp_cprmp_cb");
    let pa = ca.register_port("cpcprmp_pa", AudioInSpec::default()).unwrap();
    let pb = cb.register_port("cpcprmp_pb", AudioInSpec::default()).unwrap();
    let pa_alt = ca.port_by_name(pa.name()).unwrap();
    let pb_alt = ca.port_by_name(pb.name()).unwrap();
    assert!(ca.is_mine(&pa));
    assert!(ca.is_mine(&pa_alt));
    assert!(!ca.is_mine(&pb));
    assert!(!ca.is_mine(&pb_alt));
}

#[test]
fn client_port_can_connect_ports() {
    let client = open_test_client("client_port_ccp");

    // initialize ports
    let in_p = client.register_port("inp", AudioInSpec::default()).unwrap();
    let out_p = client.register_port("outp", AudioOutSpec::default()).unwrap();

    // start client
    let client = ActiveClient::new(client, DummyHandler).unwrap();

    // connect them
    client.connect_ports(&out_p, &in_p).unwrap();
}

#[test]
fn client_port_can_connect_ports_by_name() {
    let client = open_test_client("client_port_ccpbn");

    // initialize ports
    let _in_p = client.register_port("inp", AudioInSpec::default()).unwrap();
    let _out_p = client.register_port("outp", AudioOutSpec::default()).unwrap();

    // start client
    let client = ActiveClient::new(client, DummyHandler).unwrap();

    // connect them
    client.connect_ports_by_name("client_port_ccpbn:outp", "client_port_ccpbn:inp")
        .unwrap();
}

#[test]
fn client_port_can_connect_unowned_ports() {
    let client = open_test_client("client_port_ccup");
    let connector = open_test_client("client_port_ccup_conn");

    // initialize ports
    let _in_p = client.register_port("inp", AudioInSpec::default()).unwrap();
    let _out_p = client.register_port("outp", AudioOutSpec::default()).unwrap();

    // start client
    let _client = ActiveClient::new(client, DummyHandler).unwrap();

    // connect them
    connector.connect_ports_by_name("client_port_ccup:outp", "client_port_ccup:inp")
        .unwrap();
}


#[test]
fn client_port_cant_connect_inactive_client() {
    let client = open_test_client("client_port_ccic");
    let other = open_test_client("client_port_ccic_other");

    // initialize ports
    let in_p = other.register_port("inp", AudioInSpec::default()).unwrap().name().to_string();
    let out_p = other.register_port("outp", AudioOutSpec::default()).unwrap().name().to_string();

    // commented out to not start client
    // let client = ActiveClient::new(client, DummyHandler).unwrap();

    // connect them
    assert_eq!(client.connect_ports_by_name(&in_p, &out_p).err(),
               Some(JackErr::PortConnectionError(in_p, out_p)));
}


#[test]
fn client_port_recognizes_already_connected_ports() {
    let client = open_test_client("client_port_racp");

    // initialize ports
    let in_p = client.register_port("conna", AudioInSpec::default()).unwrap();
    let out_p = client.register_port("connb", AudioOutSpec::default()).unwrap();

    // start client
    let client = ActiveClient::new(client, DummyHandler).unwrap();

    // attempt to connect the ports twice
    client.connect_ports(&out_p, &in_p).unwrap();
    assert_eq!(client.connect_ports(&out_p, &in_p),
               Err(JackErr::PortAlreadyConnected(out_p.name().to_string(),
                                                 in_p.name().to_string())));
}

#[test]
fn client_port_fails_to_connect_nonexistant_ports() {
    let client = open_test_client("client_port_ftcnp");
    let client = ActiveClient::new(client, DummyHandler).unwrap();
    assert_eq!(client.connect_ports_by_name("doesnt_exist", "also_no_exist"),
               Err(JackErr::PortConnectionError("doesnt_exist".to_string(),
                                                "also_no_exist".to_string())));
}

#[test]
fn client_port_can_disconnect_port_from_all() {
    let client = open_test_client("client_port_cdpfa");

    // initialize ports
    let in_p = client.register_port("conna", AudioInSpec::default()).unwrap();
    let out_p = client.register_port("connb", AudioOutSpec::default()).unwrap();

    // start client
    let client = ActiveClient::new(client, DummyHandler).unwrap();

    // connect and disconnect
    client.connect_ports(&out_p, &in_p).unwrap();
    in_p.disconnect().unwrap();
}

#[test]
fn client_port_can_disconnect_ports() {
    let client = open_test_client("client_port_cdp");

    // initialize ports
    let in_p = client.register_port("conna", AudioInSpec::default()).unwrap();
    let out_p = client.register_port("connb", AudioOutSpec::default()).unwrap();

    // start client
    let client = ActiveClient::new(client, DummyHandler).unwrap();

    // connect and disconnect
    client.connect_ports(&out_p, &in_p).unwrap();
    client.disconnect_ports(&out_p, &in_p).unwrap();
}

#[test]
fn client_port_can_disconnect_ports_by_name() {
    let client = open_test_client("client_port_cdpbn");

    // initialize ports
    let in_p = client.register_port("conna", AudioInSpec::default()).unwrap();
    let out_p = client.register_port("connb", AudioOutSpec::default()).unwrap();

    // start client
    let client = ActiveClient::new(client, DummyHandler).unwrap();

    // connect and disconnect
    client.connect_ports_by_name(out_p.name(), in_p.name()).unwrap();
    client.disconnect_ports_by_name(out_p.name(), in_p.name()).unwrap();
}

#[test]
fn client_port_can_disconnect_unowned_ports() {
    let client = open_test_client("client_port_cdup");
    let disconnector = open_test_client("client_port_cdup_disc");

    // initialize ports
    let in_p = client.register_port("conna", AudioInSpec::default()).unwrap();
    let out_p = client.register_port("connb", AudioOutSpec::default()).unwrap();

    // start client
    let client = ActiveClient::new(client, DummyHandler).unwrap();

    // connect and disconnect
    client.connect_ports_by_name(out_p.name(), in_p.name()).unwrap();
    disconnector.disconnect_ports_by_name(out_p.name(), in_p.name()).unwrap();
}

#[test]
fn client_port_can_get_existing_ports() {
    let client = open_test_client("client_port_cgep");
    let port_getter = open_test_client("client_port_cgep_getter");

    // initialize ports
    let in_p = client.register_port("conna", AudioInSpec::default()).unwrap();
    let out_p = client.register_port("connb", AudioOutSpec::default()).unwrap();

    // retrieve
    use std::collections::HashSet;
    let known_ports = [in_p.name().to_string(),
                       out_p.name().to_string(),
                       "system:playback_2".to_string(),
                       "system:playback_1".to_string(),
                       "system:capture_1".to_string(),
                       "system:capture_2".to_string()];
    let exp: HashSet<String> = known_ports.into_iter().cloned().collect();
    let got: HashSet<String> = port_getter.ports(None, None, PortFlags::empty())
        .into_iter()
        .collect();
    let intersection: HashSet<String> = exp.intersection(&got).cloned().collect();
    assert_eq!(exp, intersection);
}

#[test]
fn client_port_can_get_port_by_name_pattern() {
    let client = open_test_client("client_port_cgpbnp");

    // retrieve
    use std::collections::HashSet;
    let known_ports = ["system:playback_2".to_string(), "system:capture_2".to_string()];
    let exp: HashSet<String> = known_ports.into_iter().cloned().collect();
    let got: HashSet<String> = client.ports(Some("2"), None, PortFlags::empty())
        .into_iter()
        .collect();
    assert_eq!(got, exp);
}

#[test]
fn client_port_can_get_port_by_type_pattern() {
    let cname = "client_port_cgpbtp";
    let pname = "midip";
    let full_name = format!("{}:{}", cname, pname);
    let client = open_test_client(cname);

    // register port with type name, like midi
    let _p = client.register_port(pname, MidiInSpec::default());
    use std::{thread, time};
    thread::sleep(time::Duration::from_millis(400));

    // retrieve
    let ports = client.ports(None, Some("midi"), PortFlags::empty());
    assert!(ports.contains(&full_name),
            "{:?} does not contain {}",
            &ports,
            &full_name);
}
