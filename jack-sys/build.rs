fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS");
    let library_found = pkg_config::find_library("jack");
    if target_os.as_ref().map(|s| s.as_str()) == Ok("linux") {
        library_found.unwrap();
    }
}
