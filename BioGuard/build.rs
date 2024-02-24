use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=static=fingerprint");
    println!("cargo:rustc-link-search=native=../Fingerprint");

    // Add the bindgen crate as a dependency
    // by adding the following line to your Cargo.toml file:
    // bindgen = "0.59.1"

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}