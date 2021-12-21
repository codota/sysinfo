fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS");
    if target_os == Ok("macos".to_string()) || target_os == Ok("ios".to_string()) {
        println!("cargo:rustc-link-lib=framework=IOKit");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=Foundation");
    }
}
