use std::panic::catch_unwind;

static INIT_LOGGING: std::sync::Once = std::sync::Once::new();

pub(crate) fn maybe_init_logging() {
    INIT_LOGGING.call_once_force(|state| {
        if state.is_poisoned() {
            return;
        }
        set_logger_impl(LoggerType::default());
    });
}

/// Describes how JACK should log info and error messages.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LoggerType {
    /// Ignore all logging from JACK.
    None,
    /// Use stdout and stderr to print messages.
    Stdio,
    /// Use the info! and error! macro from the [log crate](https://crates.io/crates/log).
    #[cfg(feature = "log")]
    Log,
    /// Use custom functions to handle logging.
    Custom {
        /// The function to call for info messages. Must not panic.
        info: unsafe extern "C" fn(*const ::libc::c_char),
        /// The function to call for error messages. Must not panic.
        error: unsafe extern "C" fn(*const ::libc::c_char),
    },
}

impl Default for LoggerType {
    #[cfg(feature = "log")]
    fn default() -> LoggerType {
        LoggerType::Log
    }

    #[cfg(not(feature = "log"))]
    fn default() -> LoggerType {
        LoggerType::Stdio
    }
}

/// Set the logger.
pub fn set_logger(logger: LoggerType) {
    // Prevents maybe_init_logging from resetting the logger.
    if !INIT_LOGGING.is_completed() {
        INIT_LOGGING.call_once(|| {});
    }
    set_logger_impl(logger);
}

fn set_logger_impl(logger: LoggerType) {
    let info_fn = match logger {
        LoggerType::None => silent_handler,
        LoggerType::Stdio => stdout_handler,
        #[cfg(feature = "log")]
        LoggerType::Log => info_handler,
        LoggerType::Custom { info, .. } => info,
    };
    let error_fn = match logger {
        LoggerType::None => silent_handler,
        LoggerType::Stdio => stderr_handler,
        #[cfg(feature = "log")]
        LoggerType::Log => error_handler,
        LoggerType::Custom { error, .. } => error,
    };
    unsafe {
        jack_sys::jack_set_error_function(Some(error_fn));
        jack_sys::jack_set_info_function(Some(info_fn));
    }
}

#[cfg(feature = "log")]
unsafe extern "C" fn error_handler(msg: *const libc::c_char) {
    let res = catch_unwind(|| match std::ffi::CStr::from_ptr(msg).to_str() {
        Ok(msg) => log::error!("{}", msg),
        Err(err) => log::error!("failed to log to JACK error: {:?}", err),
    });
    if let Err(err) = res {
        eprintln!("{err:?}");
        std::mem::forget(err);
    }
}

#[cfg(feature = "log")]
unsafe extern "C" fn info_handler(msg: *const libc::c_char) {
    let res = catch_unwind(|| match std::ffi::CStr::from_ptr(msg).to_str() {
        Ok(msg) => log::info!("{}", msg),
        Err(err) => log::error!("failed to log to JACK info: {:?}", err),
    });
    if let Err(err) = res {
        eprintln!("{err:?}");
        std::mem::forget(err);
    }
}

unsafe extern "C" fn stderr_handler(msg: *const libc::c_char) {
    let res = catch_unwind(|| match std::ffi::CStr::from_ptr(msg).to_str() {
        Ok(msg) => eprintln!("{}", msg),
        Err(err) => eprintln!("failed to log to JACK error: {:?}", err),
    });
    if let Err(err) = res {
        eprintln!("{err:?}");
        std::mem::forget(err);
    }
}

unsafe extern "C" fn stdout_handler(msg: *const libc::c_char) {
    let res = catch_unwind(|| match std::ffi::CStr::from_ptr(msg).to_str() {
        Ok(msg) => println!("{}", msg),
        Err(err) => println!("failed to log to JACK info: {:?}", err),
    });
    if let Err(err) = res {
        eprintln!("{err:?}");
        std::mem::forget(err);
    }
}

unsafe extern "C" fn silent_handler(_msg: *const libc::c_char) {}
