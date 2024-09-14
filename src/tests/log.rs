#[test]
fn can_set_logger() {
    crate::set_logger(crate::LoggerType::Custom {
        info: test_info_callback,
        error: test_error_callback,
    });
    assert!(unsafe { crate::jack_sys::jack_info_callback } == Some(test_info_callback),);
    assert!(unsafe { crate::jack_sys::jack_error_callback } == Some(test_error_callback),);
    super::log_to_stdio(); // Revert to enable debugging in other tests.
}

unsafe extern "C" fn test_info_callback(_msg: *const libc::c_char) {}
unsafe extern "C" fn test_error_callback(_msg: *const libc::c_char) {}
