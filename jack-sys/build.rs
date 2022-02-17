fn main() {
    println!("cargo:rerun-if-env-changed=RUST_JACK_DLOPEN");
    let dlopen = std::env::var("RUST_JACK_DLOPEN").is_ok();
    if dlopen {
        println!("cargo:rustc-cfg=feature=\"dlopen\"");
    }
    if !(dlopen || cfg!(feature = "dlopen")) {
        // pkg-config is required to find PipeWire's implementation of libjack
        // Refer to https://github.com/RustAudio/rust-jack/issues/142 for details.
        // Do not unwrap this because linking might still work if pkg-config is
        // not installed, for example on Windows.
        pkg_config::find_library("jack").unwrap();
    }
}
