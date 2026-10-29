use std::{
    thread::sleep,
    time::{Duration, Instant},
};

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
    let ac = client.activate_async((), ()).unwrap();
    let transport = ac.as_client().transport();

    // Transport requests take effect asynchronously over subsequent process cycles.
    let wait_for_state = |expected| {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let state = transport.query().unwrap().state;
            if state == expected {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for transport state {:?}; last state: {:?}",
                expected,
                state
            );
            sleep(Duration::from_millis(5));
        }
    };

    transport.stop().unwrap();
    wait_for_state(TransportState::Stopped);

    transport.start().unwrap();
    wait_for_state(TransportState::Rolling);

    transport.stop().unwrap();
    wait_for_state(TransportState::Stopped);
}
