use jack_sys as j;
use std::ffi;

/// Collects strings from an array of c-strings into a Rust vector of strings
/// and frees the memory pointed to by `ptr`. The end of the array is marked by
/// the value of the c-string being the null pointer. `ptr` may be `null`, in
/// which case nothing (deallocating) is done and an empty vector is returned.
pub unsafe fn collect_strs(ptr: *const *const libc::c_char) -> Vec<String> {
    if ptr.is_null() {
        return Vec::new();
    };
    let len = {
        let mut len = 0;
        while !(*ptr.offset(len)).is_null() {
            len += 1;
        }
        len
    };
    let mut strs = Vec::with_capacity(len as usize);
    for i in 0..len {
        let cstr_ptr = *ptr.offset(i);
        let s = ffi::CStr::from_ptr(cstr_ptr).to_string_lossy().into_owned();
        strs.push(s);
    }
    j::jack_free(ptr as *mut ::libc::c_void);
    strs
}

/// Convert `s` into a `CString` for passing to JACK, or return `Err(Error::InvalidString)` if it
/// contains a NUL byte.
pub(crate) fn to_cstring(s: &str) -> Result<ffi::CString, crate::Error> {
    ffi::CString::new(s).map_err(|_| crate::Error::InvalidString(s.to_string()))
}

/// Convert a C string returned by JACK into an owned `String`, replacing invalid UTF-8.
///
/// # Safety
/// `ptr` must be NULL or point to a NUL terminated string.
pub(crate) unsafe fn lossy_string(ptr: *const libc::c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

/// Defers POSIX thread cancellation on the current thread for as long as it is alive.
///
/// Some JACK implementations, notably PipeWire's, stop their process thread with
/// `pthread_cancel`. If the cancellation is acted on at a cancellation point inside a Rust
/// callback (for example, the `write` behind `eprintln!`), glibc starts a forced unwind through
/// Rust frames. `catch_unwind` cannot handle that and the process aborts with "FATAL: exception not
/// rethrown". Every callback that JACK invokes holds this guard, so any pending cancellation is
/// acted on only after the callback has returned to JACK.
///
/// This only flips a thread-local flag in libc, so it is realtime safe.
pub(crate) struct DeferThreadCancellation {
    #[cfg(unix)]
    old_state: libc::c_int,
}

#[cfg(unix)]
mod cancel {
    extern "C" {
        // Not exposed by the `libc` crate on all targets.
        pub fn pthread_setcancelstate(
            state: libc::c_int,
            oldstate: *mut libc::c_int,
        ) -> libc::c_int;
    }

    #[cfg(target_vendor = "apple")]
    pub const PTHREAD_CANCEL_DISABLE: libc::c_int = 0x00;
    #[cfg(not(target_vendor = "apple"))]
    pub const PTHREAD_CANCEL_DISABLE: libc::c_int = 1;
}

impl DeferThreadCancellation {
    #[inline]
    pub(crate) fn new() -> Self {
        #[cfg(unix)]
        {
            let mut old_state = cancel::PTHREAD_CANCEL_DISABLE;
            unsafe {
                cancel::pthread_setcancelstate(cancel::PTHREAD_CANCEL_DISABLE, &mut old_state)
            };
            DeferThreadCancellation { old_state }
        }
        #[cfg(not(unix))]
        DeferThreadCancellation {}
    }
}

impl Drop for DeferThreadCancellation {
    #[inline]
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            cancel::pthread_setcancelstate(self.old_state, std::ptr::null_mut());
        }
    }
}
