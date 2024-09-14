type LogFn = unsafe extern "C" fn(*const libc::c_char);

unsafe extern "C" fn test_info_callback(_msg: *const libc::c_char) {}
unsafe extern "C" fn test_error_callback(_msg: *const libc::c_char) {}

#[test]
fn can_set_logger() {
    crate::set_logger(crate::LoggerType::Custom {
        info: test_info_callback,
        error: test_error_callback,
    });
    #[cfg(feature = "dynamic_loading")]
    unsafe {
        let lib = jack_sys::library().unwrap();
        assert!(**lib.get::<*const LogFn>(b"jack_info_callback").unwrap() == test_info_callback);
        assert!(**lib.get::<*const LogFn>(b"jack_error_callback").unwrap() == test_error_callback);
    }
    #[cfg(not(feature = "dynamic_loading"))]
    {
        assert!(unsafe { crate::jack_sys::jack_info_callback } == Some(test_info_callback),);
        assert!(unsafe { crate::jack_sys::jack_error_callback } == Some(test_error_callback),);
    }
    super::log_to_stdio(); // Revert to enable debugging in other tests.
}
