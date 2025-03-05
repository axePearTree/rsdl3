use std::path::PathBuf;

fn main() {
    generate_core_bindings();
    generate_image_bindings();
}

fn generate_core_bindings() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let core_include_path = manifest_dir.join("SDL").join("include");
    let core_header_path = core_include_path.join("SDL3").join("SDL.h");
    bindgen::Builder::default()
        .use_core()
        .clang_arg(format!("-I{}", core_include_path.to_str().unwrap()))
        .ctypes_prefix("libc")
        .header(core_header_path.to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap()
        .write_to_file(manifest_dir.join("../src").join("core.rs"))
        .unwrap();
}

fn generate_image_bindings() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let img_header_path = manifest_dir
        .join("SDL_image")
        .join("include")
        .join("SDL3_image")
        .join("SDL_image.h");
    bindgen::Builder::default()
        .use_core()
        .raw_line("use crate::*;")
        .ctypes_prefix("libc")
        .header(img_header_path.to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .blocklist_type("SDL_.*")
        .generate()
        .unwrap()
        .write_to_file(manifest_dir.join("../src").join("image.rs"))
        .unwrap();
}
