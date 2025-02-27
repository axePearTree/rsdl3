use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let include_path = manifest_dir.join("SDL").join("include");
    let header_path = include_path.join("SDL3").join("SDL.h");
    bindgen::Builder::default()
        .use_core()
        .clang_arg(format!("-I{}", include_path.to_str().unwrap()))
        .ctypes_prefix("libc")
        .header(header_path.to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap()
        .write_to_file(manifest_dir.join("src").join("bindings.rs"))
        .unwrap();
}
