use std::{thread::sleep, time::Duration};

use crate::{Client, TransportPosition, TransportState};

#[test]
fn new_transport_is_not_valid() {
    assert!(!TransportPosition::default().valid_bbt());
    assert!(!TransportPosition::default().valid_bbt_frame_offset());
    assert_eq!(TransportPosition::default().frame(), 0);
    assert_eq!(TransportPosition::default().bbt(), None);
    assert_eq!(TransportPosition::default().bbt_offset(), None);
    assert_eq!(TransportPosition::default().frame_rate(), None);
    assert_eq!(TransportPosition::default().usecs(), None);
}

#[test]
fn starting_transport_sets_state_to_started() {
    let (client, _) = Client::new("", Default::default()).unwrap();
    let transport = client.transport();

    transport.stop().unwrap();
    sleep(Duration::from_millis(50));
    assert_eq!(transport.query().unwrap().state, TransportState::Stopped);

    transport.start().unwrap();
    sleep(Duration::from_millis(50));
    assert_eq!(transport.query().unwrap().state, TransportState::Rolling);

    transport.stop().unwrap();
}
