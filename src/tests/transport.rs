use std::time::Duration;

use crate::{TransportPosition, TransportState};

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
    // The transport only moves while process cycles run, so the client must be active.
    let ac = crate::tests::active_client();
    let transport = ac.as_client().transport();
    // Transport requests take effect asynchronously over subsequent process cycles.
    let state_becomes = |state| {
        crate::tests::wait_for(Duration::from_secs(3), || {
            transport.query().unwrap().state == state
        })
    };

    transport.stop().unwrap();
    assert!(state_becomes(TransportState::Stopped));

    transport.start().unwrap();
    let rolled = state_becomes(TransportState::Rolling);
    transport.stop().unwrap();
    assert!(rolled, "{:?}", transport.query().unwrap().state);
    assert!(state_becomes(TransportState::Stopped));
}

#[test]
fn bbt_with_nan_is_invalid() {
    use crate::{TransportBBT, TransportBBTValidationError as E};
    let nan = |f: fn(&mut TransportBBT)| {
        let mut bbt = TransportBBT::default();
        f(&mut bbt);
        bbt.validated()
    };
    assert_eq!(nan(|b| b.sig_num = f32::NAN), Err(E::SigNumRange));
    assert_eq!(nan(|b| b.sig_denom = f32::NAN), Err(E::SigDenomRange));
    assert_eq!(
        nan(|b| b.ticks_per_beat = f64::NAN),
        Err(E::TicksPerBeatRange)
    );
    assert_eq!(nan(|b| b.bpm = f64::NAN), Err(E::BPMRange));
    assert_eq!(
        nan(|b| b.bar_start_tick = f64::NAN),
        Err(E::BarStartTickRange)
    );
    assert!(TransportBBT::default().validated().is_ok());
}

#[test]
fn bbt_offset_round_trips() {
    let mut pos = TransportPosition::default();
    let () = pos.set_bbt_offset(Some(5));
    assert_eq!(pos.bbt_offset(), Some(5));
    let () = pos.set_bbt_offset(None);
    assert_eq!(pos.bbt_offset(), None);
}
