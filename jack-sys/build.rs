fn main() {
    // With `dynamic_loading`, libjack is opened at runtime with `dlopen`, so it is neither needed
    // nor linked at build time.
    if std::env::var_os("CARGO_FEATURE_DYNAMIC_LOADING").is_some() {
        return;
    }
    let target_os = std::env::var("CARGO_CFG_TARGET_OS");
    let library_found = pkg_config::find_library("jack");
    if target_os.as_ref().map(|s| s.as_str()) == Ok("linux") {
        if let Err(err) = library_found {
            panic!(
                "Could not find the JACK library with pkg-config. Install the JACK development \
                 package (for example libjack-jackd2-dev or pipewire-jack), or enable the \
                 `dynamic_loading` feature to load JACK at runtime instead.\n{}",
                err
            );
        }
    }
}
