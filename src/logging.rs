use jack_sys as j;
use std::ffi;

unsafe extern "C" fn error_fn(msg: *const libc::c_char) {
    match ffi::CStr::from_ptr(msg).to_str() {
        Ok(msg) => log::error!("{}", msg),
        Err(err) => log::error!("failed to parse JACK error: {:?}", err),
    }
}

unsafe extern "C" fn info_fn(msg: *const libc::c_char) {
    match ffi::CStr::from_ptr(msg).to_str() {
        Ok(msg) => log::info!("{}", msg),
        Err(err) => log::error!("failed to parse JACK error: {:?}", err),
    }
}

pub fn initialize_logging() {
    unsafe {
        j::jack_set_error_function(Some(error_fn));
        j::jack_set_info_function(Some(info_fn));
    }
}
