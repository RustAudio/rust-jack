fn main() {
    println!("cargo:rerun-if-env-changed=RUST_JACK_DLOPEN");
    let dlopen = std::env::var("RUST_JACK_DLOPEN").is_ok();
    if dlopen {
        println!("cargo:rustc-cfg=feature=\"dlopen\"");
    }
}
