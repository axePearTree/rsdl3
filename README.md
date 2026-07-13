# rsdl3

SDL3 bindings for Rust.

## Crates

- `rsdl3`: safe `no_std` wrapper over SDL3.
- `rsdl3-sys`: raw SDL3 bindings.
- `rsdl3-macros`: internal proc-macro crate used by `rsdl3::runtime`.

## Features

- `image`: enables SDL_image bindings through `rsdl3-sys/image`.
- `runtime`: enables the SDL-provided entrypoint runtime for `#![no_std]` / `#![no_main]` apps, including `#[rsdl3::main]`, `rsdl3::runtime::Args`, allocator glue, panic glue, and the SDL main shim.
- `no_panic_handler`: disables `rsdl3::runtime`'s default panic handler so the final application can provide its own `#[panic_handler]`.

## Adding to an existing project

Add `rsdl3` normally:

```toml
[dependencies]
rsdl3 = { path = "../rsdl3/rsdl3" }
```

If SDL3 is not already in your platform's default linker search path, add a build script to your application:

```rust
// build.rs
fn main() {
    println!("cargo:rustc-link-search=native=/path/to/sdl3/lib");
    println!("cargo:rustc-link-lib=SDL3");

    // If rsdl3's `image` feature is enabled, link SDL3_image too.
    println!("cargo:rustc-link-lib=SDL3_image");
}
```

Cargo expects the library name without the `lib` prefix or extension, so `libSDL3.so`, `libSDL3.a`, or `SDL3.lib` are linked as `SDL3`, and `libSDL3_image.so`, `libSDL3_image.a`, or `SDL3_image.lib` are linked as `SDL3_image`.

Use your regular Rust `main`:

```rust
fn main() -> Result<(), rsdl3::Error> {
    let mut sdl = unsafe { rsdl3::Sdl::init() }?;
    let video = sdl.video()?;

    let _window = video.create_window("rsdl3 app", 800, 600, None)?;

    Ok(())
}
```

## Running a project using `SDL_MAIN` via `runtime` feature

Enable the runtime feature:

```toml
[dependencies]
rsdl3 = { path = "../rsdl3/rsdl3", features = ["runtime"] }

[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

Then let SDL provide the platform entrypoint:

```rust
#![no_std]
#![no_main]

#[rsdl3::main]
fn main(_args: rsdl3::runtime::Args) -> Result<(), rsdl3::Error> {
    let mut sdl = unsafe { rsdl3::Sdl::init() }?;
    let video = sdl.video()?;

    let _window = video.create_window("rsdl3 runtime app", 800, 600, None)?;

    Ok(())
}
```

`#[rsdl3::main]` exports `SDL_main`; the runtime shim includes SDL's `SDL_main.h` support so SDL owns the real platform startup path.

## Panic Handling

With `runtime`, `rsdl3` provides a default panic handler unless `no_panic_handler` is enabled. The default handler logs the panic message through SDL logging and exits with status `1`.

If your platform needs different panic behavior, enable `no_panic_handler` and define your own handler in the final application:

```toml
[dependencies]
rsdl3 = { path = "../rsdl3/rsdl3", features = ["runtime", "no_panic_handler"] }
```

```rust
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    // Platform-specific abort, logging, reset, trap, etc.
    loop {}
}
```

## Native SDL3

The target system must provide SDL3 headers and libraries. `rsdl3-sys` links against SDL3, and the `runtime` feature also compiles a small C shim that includes SDL's `SDL_main.h`.
