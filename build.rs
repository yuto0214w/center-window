use std::{env, path::Path};

const MANIFEST_PATH: &'static str = "center-window.exe.manifest";

fn main() {
    if !env::var("TARGET").unwrap().ends_with("windows-msvc") {
        panic!("target must be windows-msvc");
    }
    let manifest = Path::new(MANIFEST_PATH).canonicalize().unwrap();
    println!("cargo:rerun-if-changed={}", MANIFEST_PATH);
    println!("cargo:rustc-link-arg-bins=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg-bins=/MANIFESTINPUT:{}",
        manifest.to_str().unwrap()
    );
}
