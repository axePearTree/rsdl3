# rsdl3

SDL3 bindings for Rust.

## Crates

- `rsdl3`: safe `no_std` wrapper over SDL3.
- `rsdl3-sys`: raw SDL3 bindings.
- `rsdl3-macros`: internal proc-macro crate used by `rsdl3::runtime`.

## Features

- `image`: enables SDL_image bindings through `rsdl3-sys/image`.
- `main`: enables SDL entrypoint support, including `#[rsdl3::main]` and `rsdl3::runtime::Args`.
- `runtime_shim`: compiles and links the bundled SDL main C shim. Use this unless your final project must compile the shim with a custom platform toolchain.
- `panic_handler`: enables `rsdl3::runtime`'s default panic handler.
- `runtime`: convenience feature for managed `#![no_std]` / `#![no_main]` apps; enables `main`, `runtime_shim`, allocator glue, and `panic_handler`.

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

If you want to use a specific C compiler or SDK toolchain, disable the bundled shim and provide it from your final project instead:

```toml
[dependencies]
rsdl3 = { path = "../rsdl3/rsdl3", features = ["main", "panic_handler"] }
```

Your application build system must then compile an equivalent C source with the platform toolchain:

```c
#define SDL_MAIN_AVAILABLE 1
#include <SDL3/SDL_main.h>
```

## Panic Handling

With `runtime`, `rsdl3` provides a default panic handler. The default handler logs the panic message through SDL logging and exits with status `1`.

If your platform needs different panic behavior, use `main` instead of `runtime` and define your own handler in the final application:

```toml
[dependencies]
rsdl3 = { path = "../rsdl3/rsdl3", features = ["main", "runtime_shim"] }
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
