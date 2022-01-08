fn main() {
    // pkg-config is required to find PipeWire's implementation of libjack
    // Refer to https://github.com/RustAudio/rust-jack/issues/142 for details.
    // Do not unwrap this because linking might still work if pkg-config is
    // not installed, for example on Windows.
    let _ = pkg_config::probe_library("jack");
}
