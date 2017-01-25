use prelude::*;

fn null_log_fn(_: &str) {}

#[test]
fn logging_can_set_info() {
    // initial state
    reset_info_callback();
    assert!(get_info_callback().is_none());

    // set
    set_info_callback(null_log_fn);
    assert!(get_info_callback().is_some());
    get_info_callback().unwrap()("Using info callback!.");

    // reset
    reset_info_callback();
    assert!(get_info_callback().is_none());
}

#[test]
fn logging_can_set_error() {
    // initial state
    reset_error_callback();
    assert!(get_error_callback().is_none());

    // set
    set_error_callback(null_log_fn);
    assert!(get_error_callback().is_some());
    get_error_callback().unwrap()("Using error callback!.");

    // reset
    reset_error_callback();
    assert!(get_error_callback().is_none());
}
