#![allow(non_camel_case_types, non_upper_case_globals)]

mod consts;
mod types;

mod functions {
    include!(concat!(env!("OUT_DIR"), "/functions.rs"));
}

pub use consts::*;
pub use functions::*;
pub use types::*;

#[cfg(not(target_os = "windows"))]
#[link(name = "jack")]
extern "C" {}
