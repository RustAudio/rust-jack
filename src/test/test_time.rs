use super::super::*;
use std::{thread, time};

#[test]
fn time_can_get_time() {
    get_time();
}

#[test]
fn time_is_monotonically_increasing() {
    let initial_t = get_time();
    thread::sleep(time::Duration::from_millis(100));
    let later_t = get_time();
    assert!(initial_t < later_t, "failed {} < {}", initial_t, later_t);
}
