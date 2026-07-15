# rsdl3

Simple Rust bindings for SDL3.

`rsdl3` is a safe, `no_std`-friendly wrapper over SDL3. It can be used as a
regular Rust library, or as a complete SDL-managed runtime for `#![no_std]` /
`#![no_main]` applications.

## Workspace

- `rsdl3`: safe SDL3 wrapper.
- `rsdl3-sys`: raw SDL3 bindings.
- `rsdl3-macros`: proc macros used by SDL entrypoint support.

## Features

- `image`: enables SDL_image bindings through `rsdl3-sys/image`.
- `main`: enables SDL entrypoint support, `#[rsdl3::main]`, and `rsdl3::runtime::Args`.
- `callbacks`: enables SDL callback mode and `#[rsdl3::application]`; also enables `main`.
- `app`: complete final-app runtime; enables `callbacks`, the bundled SDL main shim, SDL-backed global allocation, and the default panic handler.

The `allocator` module is always available. The `app` feature installs
`rsdl3::allocator::SDLAllocator` as the global allocator for the final binary.

## Native SDL3

The target system must provide SDL3 headers and libraries. If SDL3 is not in the
default linker search path, add it from your application build script.

```rust
// build.rs
fn main() {
    // Add the path to the SDL3 lib.
    // This is not needed if you have a system-wide installation of SDL3.
    println!("cargo:rustc-link-search=native=/path/to/sdl3/lib");

    // Link SDL3
    println!("cargo:rustc-link-lib=SDL3");

    // Link SDL3_image - required when rsdl3's `image` feature is enabled.
    println!("cargo:rustc-link-lib=SDL3_image");
}
```

Cargo expects the library name without the `lib` prefix or extension:
`libSDL3.so`, `libSDL3.a`, or `SDL3.lib` are linked as `SDL3`.

## Regular Rust App

If you want a normal Rust `main`, use the default features or choose the exact
library features you need.

```toml
[dependencies]
rsdl3 = { path = "../rsdl3/rsdl3" }
```

```rust
fn main() -> Result<(), rsdl3::Error> {
    // SAFETY: Sdl must be initialized in the main thread.
    let mut sdl = unsafe { rsdl3::Sdl::init() }?;
    let video = sdl.video()?;

    let _window = video.create_window("rsdl3 app", 800, 600, None)?;

    Ok(())
}
```

## SDL App Runtime

Use `app` for final `#![no_std]` SDL apps. SDL owns startup and calls your
application through callbacks.

```toml
[dependencies]
rsdl3 = { path = "../rsdl3/rsdl3", features = ["app"] }

[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

```rust
#![no_std]
#![no_main]

use rsdl3::runtime::Args;
use rsdl3::runtime::callbacks::{Callbacks, ControlFlow};

#[rsdl3::application]
struct App {
    _sdl: rsdl3::Sdl,
}

impl Callbacks for App {
    fn init(_args: Args) -> Result<Self, rsdl3::Error> {
        let sdl = unsafe { rsdl3::Sdl::init() }?;

        Ok(Self { _sdl: sdl })
    }

    fn iterate(&mut self) -> Result<ControlFlow, rsdl3::Error> {
        Ok(ControlFlow::Continue)
    }
}
```

Callback return values are ordinary Rust results:

- `Ok(ControlFlow::Continue)`: keep running.
- `Ok(ControlFlow::Success)`: stop successfully.
- `Err(rsdl3::Error)`: stop with failure.

With `app` and `callbacks`, `rsdl3` compiles a tiny C shim that includes
SDL's `SDL_main.h` with `SDL_MAIN_USE_CALLBACKS=1`, so SDL handles platform startup.
Do not use `#[rsdl3::main]` in callback mode.

## Classic `SDL_main`

Use `#[rsdl3::main]` when you want SDL's classic `SDL_main` entrypoint instead
of callback mode.

```toml
[dependencies]
rsdl3 = { path = "../rsdl3/rsdl3", default-features = false, features = ["main"] }
```

```rust
#![no_std]
#![no_main]

use rsdl3::allocator::SDLAllocator;
use rsdl3::runtime::Args;

#[global_allocator]
static ALLOCATOR: SDLAllocator = SDLAllocator;

#[rsdl3::main]
fn main(_args: Args) -> Result<(), rsdl3::Error> {
    let mut sdl = unsafe { rsdl3::Sdl::init() }?;
    let video = sdl.video()?;

    let _window = video.create_window("rsdl3 app", 800, 600, None)?;

    Ok(())
}
```

For classic mode, provide your own panic handler and SDL main shim from the
final application or platform build system.

```c
#define SDL_MAIN_AVAILABLE 1
#include <SDL3/SDL_main.h>
```

## Panic Handling

For `#![no_std]` / `#![no_main]` final binaries, enable aborting panics:

```toml
[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

The `app` feature installs a panic handler that logs through SDL and exits with
status `1`.

If your platform needs different behavior, disable `app` and define your own
handler in the final application.

```rust
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {}
}
```
