use super::*;
use crate::Client;
use crate::ClientOptions;
use crate::Error;
use crate::NotificationHandler;
use crate::PortId;
use crate::PORT_NAME_SIZE;
use std::collections::HashSet;
use std::sync::mpsc;
use std::sync::Mutex;
use std::{thread, time};

fn open_test_client(name: &str) -> Client {
    Client::new(name, ClientOptions::NO_START_SERVER).unwrap().0
}

#[test]
fn client_port_can_register_port() {
    let c = open_test_client("cp_can_register_port");
    c.register_port("cpcrp_a", AudioIn::default()).unwrap();
}

#[test]
fn client_port_register_port_enforces_unique_names() {
    let pname = "cprpeun_a";
    let c = open_test_client("cp_can_register_port");
    c.register_port(pname, AudioIn::default()).unwrap();
    assert_eq!(
        c.register_port(pname, AudioIn::default()).err(),
        Some(Error::PortRegistrationError(pname.to_string()))
    );
}

#[test]
fn client_port_register_port_enforces_name_length() {
    let c = open_test_client("cp_can_register_port");
    let pname = (0..=*PORT_NAME_SIZE)
        .map(|_| "a")
        .collect::<Vec<&str>>()
        .join("_");
    assert_eq!(
        c.register_port(&pname, AudioIn::default()).err(),
        Some(Error::PortRegistrationError(pname.to_string()))
    );
}

#[test]
fn client_port_can_request_monitor_by_name() {
    let c = open_test_client("cp_can_request_monitor_by_name");
    let p = c.register_port("cpcrmbn_a", AudioIn::default()).unwrap();
    c.request_monitor_by_name(&p.name().unwrap(), true).unwrap();
    c.request_monitor_by_name(&p.name().unwrap(), false)
        .unwrap();
}

#[test]
fn client_port_can_get_port_by_name() {
    let c = open_test_client("cp_can_get_port_by_name");
    let p = c.register_port("named_port", AudioIn::default()).unwrap();
    let _p = c.port_by_name(&p.name().unwrap()).unwrap();
}

pub struct PortIdHandler {
    pub reg_tx: Mutex<mpsc::SyncSender<PortId>>,
}

impl NotificationHandler for PortIdHandler {
    fn port_registration(&mut self, _: &Client, pid: PortId, is_registered: bool) {
        if is_registered {
            self.reg_tx.lock().unwrap().send(pid).unwrap()
        }
    }
}

#[test]
fn client_port_can_get_port_by_id() {
    let (client_name, port_name) = ("cp_can_get_port_by_id", "cp_registered_port_name");

    // Create handler
    let (reg_tx, reg_rx) = mpsc::sync_channel(200);
    let h = PortIdHandler {
        reg_tx: Mutex::new(reg_tx),
    };

    // Open and activate client
    let c = open_test_client(client_name);
    let ac = c.activate_async(h, ()).unwrap();

    // Register port
    let _pa = ac
        .as_client()
        .register_port(port_name, AudioIn::default())
        .unwrap();

    // Get by id
    let c = ac.deactivate().unwrap().0;
    let mut registered_ports = reg_rx
        .iter()
        .flat_map(|i| c.port_by_id(i))
        .map(|p| p.name().unwrap());
    let port_name = format!("{client_name}:{port_name}");
    assert!(registered_ports.any(|n| n == port_name));

    // Port that doesn't exist
    // TODO: Restore when JACK doesn't exit when this happens.
    // let nonexistant_port = c.port_by_id(10000);
    // assert!(
    //     nonexistant_port.is_none(),
    //     format!("Expected None but got: {:?}", nonexistant_port)
    // );
}

#[test]
fn client_port_fails_to_nonexistant_port() {
    let c = open_test_client("cp_can_request_monitor_by_name");
    let p = c.register_port("cpcrmbn_a", AudioIn::default()).unwrap();
    let _p = c.port_by_name(&p.name().unwrap()).unwrap();
}

#[test]
fn client_port_recognizes_my_ports() {
    let ca = open_test_client("cp_cprmp_ca");
    let cb = open_test_client("cp_cprmp_cb");
    let first = ca.register_port("cpcprmp_pa", AudioIn::default()).unwrap();
    let second = cb.register_port("cpcprmp_pb", AudioIn::default()).unwrap();
    let first_alt = ca.port_by_name(&first.name().unwrap()).unwrap();
    let second_alt = ca.port_by_name(&second.name().unwrap()).unwrap();
    assert!(ca.is_mine(&first));
    assert!(ca.is_mine(&first_alt));
    assert!(!ca.is_mine(&second));
    assert!(!ca.is_mine(&second_alt));
}

#[test]
fn client_port_can_connect_ports() {
    let client = open_test_client("client_port_ccp");

    // initialize ports
    let in_p = client.register_port("inp", AudioIn::default()).unwrap();
    let out_p = client.register_port("outp", AudioOut::default()).unwrap();

    // start client
    let client = client.activate_async((), ()).unwrap();

    // connect them
    client.as_client().connect_ports(&out_p, &in_p).unwrap();
}

#[test]
fn client_port_can_connect_ports_by_name() {
    let client = open_test_client("client_port_ccpbn");

    // initialize ports
    let _in_p = client.register_port("inp", AudioIn::default()).unwrap();
    let _out_p = client.register_port("outp", AudioOut::default()).unwrap();

    // start client
    let client = client.activate_async((), ()).unwrap();

    // connect them
    client
        .as_client()
        .connect_ports_by_name("client_port_ccpbn:outp", "client_port_ccpbn:inp")
        .unwrap();
}

#[test]
fn client_port_can_connect_unowned_ports() {
    let client = open_test_client("client_port_ccup");
    let connector = open_test_client("client_port_ccup_conn");

    // initialize ports
    let _in_p = client.register_port("inp", AudioIn::default()).unwrap();
    let _out_p = client.register_port("outp", AudioOut::default()).unwrap();

    // start client
    let _client = client.activate_async((), ()).unwrap();

    // connect them
    connector
        .connect_ports_by_name("client_port_ccup:outp", "client_port_ccup:inp")
        .unwrap();
}

#[test]
fn client_port_cant_connect_inactive_client() {
    let client = open_test_client("client_port_ccic");
    let other = open_test_client("client_port_ccic_other");

    // initialize ports
    let in_p = other
        .register_port("inp", AudioIn::default())
        .unwrap()
        .name()
        .unwrap();
    let out_p = other
        .register_port("outp", AudioOut::default())
        .unwrap()
        .name()
        .unwrap();

    // Normally we start a client before we begin connecting, but in this case
    // we're checking for errors that happen when we connect before activating.
    //
    // let client = client.activate_async((), ()).unwrap();

    // connect them
    assert_eq!(
        client.connect_ports_by_name(&in_p, &out_p).err(),
        Some(Error::PortConnectionError(in_p, out_p))
    );
}

#[test]
fn client_port_recognizes_already_connected_ports() {
    let client = open_test_client("client_port_racp");

    // initialize ports
    let in_p = client.register_port("conna", AudioIn::default()).unwrap();
    let out_p = client.register_port("connb", AudioOut::default()).unwrap();

    // start client
    let client = client.activate_async((), ()).unwrap();

    // attempt to connect the ports twice
    client.as_client().connect_ports(&out_p, &in_p).unwrap();
    assert_eq!(
        client.as_client().connect_ports(&out_p, &in_p),
        Err(Error::PortAlreadyConnected(
            out_p.name().unwrap(),
            in_p.name().unwrap(),
        ))
    );
}

#[test]
fn client_port_fails_to_connect_nonexistant_ports() {
    let client = open_test_client("client_port_ftcnp")
        .activate_async((), ())
        .unwrap();
    assert_eq!(
        client
            .as_client()
            .connect_ports_by_name("doesnt_exist", "also_no_exist"),
        Err(Error::PortConnectionError(
            "doesnt_exist".to_string(),
            "also_no_exist".to_string(),
        ))
    );
}

#[test]
fn client_port_can_disconnect_port_from_all() {
    let client = open_test_client("client_port_cdpfa");

    // initialize ports
    let in_p = client.register_port("conna", AudioIn::default()).unwrap();
    let out_p = client.register_port("connb", AudioOut::default()).unwrap();

    // start client
    let client = client.activate_async((), ()).unwrap();

    // connect and disconnect
    client.as_client().connect_ports(&out_p, &in_p).unwrap();
    client.as_client().disconnect(&in_p).unwrap();
}

#[test]
fn client_port_can_disconnect_ports() {
    let client = open_test_client("client_port_cdp");

    // initialize ports
    let in_p = client.register_port("conna", AudioIn::default()).unwrap();
    let out_p = client.register_port("connb", AudioOut::default()).unwrap();

    // start client
    let client = client.activate_async((), ()).unwrap();

    // connect and disconnect
    client.as_client().connect_ports(&out_p, &in_p).unwrap();
    client.as_client().disconnect_ports(&out_p, &in_p).unwrap();
}

#[test]
fn client_port_can_disconnect_ports_by_name() {
    let client = open_test_client("client_port_cdpbn");

    // initialize ports
    let in_p = client.register_port("conna", AudioIn::default()).unwrap();
    let out_p = client.register_port("connb", AudioOut::default()).unwrap();

    // start client
    let client = client.activate_async((), ()).unwrap();

    // connect and disconnect
    client
        .as_client()
        .connect_ports_by_name(&out_p.name().unwrap(), &in_p.name().unwrap())
        .unwrap();
    client
        .as_client()
        .disconnect_ports_by_name(&out_p.name().unwrap(), &in_p.name().unwrap())
        .unwrap();
}

#[test]
fn client_port_can_disconnect_unowned_ports() {
    let client = open_test_client("client_port_cdup");
    let disconnector = open_test_client("client_port_cdup_disc");

    // initialize ports
    let in_p = client.register_port("conna", AudioIn::default()).unwrap();
    let out_p = client.register_port("connb", AudioOut::default()).unwrap();

    // start client
    let client = client.activate_async((), ()).unwrap();

    // connect and disconnect
    client
        .as_client()
        .connect_ports_by_name(&out_p.name().unwrap(), &in_p.name().unwrap())
        .unwrap();
    disconnector
        .disconnect_ports_by_name(&out_p.name().unwrap(), &in_p.name().unwrap())
        .unwrap();
}

#[test]
fn client_port_can_get_existing_ports() {
    let client = open_test_client("client_port_cgep");
    let port_getter = open_test_client("client_port_cgep_getter");

    // initialize ports
    let in_p = client.register_port("conna", AudioIn::default()).unwrap();
    let out_p = client.register_port("connb", AudioOut::default()).unwrap();

    // retrieve
    let known_ports = [
        in_p.name().unwrap(),
        out_p.name().unwrap(),
        "system:playback_2".to_string(),
        "system:playback_1".to_string(),
        "system:capture_1".to_string(),
        "system:capture_2".to_string(),
    ];
    let exp: HashSet<String> = known_ports.iter().cloned().collect();
    let got: HashSet<String> = port_getter
        .ports(None, None, PortFlags::empty())
        .into_iter()
        .collect();
    let intersection: HashSet<String> = exp.intersection(&got).cloned().collect();
    assert_eq!(exp, intersection);
}

#[test]
fn client_port_can_get_port_by_name_pattern() {
    let client = open_test_client("client_port_cgpbnp");

    // retrieve
    let known_ports = [
        "system:playback_2".to_string(),
        "system:capture_2".to_string(),
    ];
    let exp: HashSet<String> = known_ports.iter().cloned().collect();
    let got: HashSet<String> = client
        .ports(Some("2"), None, PortFlags::empty())
        .into_iter()
        .collect();
    assert_eq!(got, exp);
}

#[test]
fn client_port_can_get_port_by_type_pattern() {
    let c_name = "client_port_cgpbtp";
    let p_name = "midip";
    let full_name = format!("{c_name}:{p_name}");
    let client = open_test_client(c_name);

    // register port with type name, like midi
    let _p = client.register_port(p_name, MidiIn::default());
    thread::sleep(time::Duration::from_millis(400));

    // retrieve
    let ports = client.ports(None, Some("midi"), PortFlags::empty());
    assert!(
        ports.contains(&full_name),
        "{:?} does not contain {}",
        &ports,
        &full_name
    );
}
