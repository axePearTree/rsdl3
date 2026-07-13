fn main() {
    println!("cargo:rerun-if-changed=src/runtime/sdl_main_shim.c");

    if std::env::var_os("CARGO_FEATURE_RUNTIME_SHIM").is_some() {
        cc::Build::new()
            .file("src/runtime/sdl_main_shim.c")
            .compile("rsdl3_main_shim");
    }
}
