#![allow(non_camel_case_types, non_upper_case_globals)]
use lazy_static::lazy_static;

mod consts;
mod types;

mod functions {
    include!(concat!(env!("OUT_DIR"), "/functions.rs"));
}

pub use consts::*;
pub use functions::*;
pub use types::*;

lazy_static! {
    static ref LIB_RESULT: Result<libloading::Library, libloading::Error> =
        unsafe { libloading::Library::new(JACK_LIB) };
}

/// Get the underlying library handle. Can be used to extract symbols from the library.
///
/// # Example
/// ```rust
/// let symbol = library.get::<unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int>(b"jack_release_timebase").unwrap();
/// let raw_symbol = symbol.into_raw();
/// let func = *raw_symbol.deref() as unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int;
/// ```
pub fn library() -> Result<&'static libloading::Library, impl std::error::Error> {
    LIB_RESULT.as_ref()
}
