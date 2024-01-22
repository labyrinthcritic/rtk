fn main() {
    println!("cargo:rustc-link-lib=OpenImageDenoise");

    let bindings = bindgen::Builder::default()
        .header("include/wrapper.h")
        // make cargo invalidate the crate when the header changes
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap();

    _ = std::fs::create_dir("target");
    bindings.write_to_file("target/bindings.rs").unwrap();
}
