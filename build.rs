#[cfg(any(target_os = "macos", target_os = "ios"))]
extern crate cc;

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn main() {
    println!("cargo:rustc-link-lib=framework=IOKit");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    println!("cargo:rustc-link-lib=framework=Foundation");
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
fn main() {}
