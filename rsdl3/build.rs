fn main() {
    println!("cargo:rerun-if-changed=src/runtime/sdl_main_shim.c");

    if std::env::var_os("CARGO_FEATURE_APP").is_some() {
        let mut build = cc::Build::new();
        build.file("src/runtime/sdl_main_shim.c");

        if std::env::var_os("CARGO_FEATURE_CALLBACKS").is_some() {
            // SDL callback mode requires this before including SDL_main.h.
            // SDL then provides SDL_main, which calls SDL_EnterAppMainCallbacks.
            build.define("SDL_MAIN_USE_CALLBACKS", "1");
        } else {
            // Force the standard SDL_main shim on platforms where SDL considers
            // it optional, such as Linux.
            build.define("SDL_MAIN_AVAILABLE", "1");
        }

        build.compile("rsdl3_main_shim");
    }
}
