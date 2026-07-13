# rsdl3

Simple Rust bindings for SDL3.

`rsdl3` is a safe, `no_std`-friendly wrapper over SDL3. You can use it like a
normal Rust library, or let SDL own the platform entrypoint for portable
`#![no_main]` applications.

## Crates

- `rsdl3`: safe SDL3 wrapper.
- `rsdl3-sys`: raw SDL3 bindings.
- `rsdl3-macros`: internal proc-macro crate used by `rsdl3` entrypoint support.

## Quick Start

```toml
[dependencies]
rsdl3 = { path = "../rsdl3/rsdl3" }
```

### Native SDL3

The target system must provide SDL3 headers and libraries. If SDL3 is not in the
default linker search path, add it from your application build script.

```rust
// build.rs
fn main() {
    println!("cargo:rustc-link-search=native=/path/to/sdl3/lib");
    println!("cargo:rustc-link-lib=SDL3");

    // If rsdl3's `image` feature is enabled, link SDL3_image too.
    println!("cargo:rustc-link-lib=SDL3_image");
}
```

Cargo expects the library name without the `lib` prefix or extension:
`libSDL3.so`, `libSDL3.a`, or `SDL3.lib` are linked as `SDL3`.

### Example

```rust
fn main() -> Result<(), rsdl3::Error> {
    let mut sdl = unsafe { rsdl3::Sdl::init() }?;
    let video = sdl.video()?;

    let _window = video.create_window("rsdl3 app", 800, 600, None)?;

    Ok(())
}
```

## SDL Entrypoints

SDL can provide the real platform entrypoint. This is useful for portable
desktop/mobile startup and for `#![no_std]` / `#![no_main]` applications.

### Classic `SDL_main`

Use `#[rsdl3::main]` when your app has one main function.

```toml
[dependencies]
rsdl3 = { path = "../rsdl3/rsdl3", features = [
    "main",
    "runtime_shim",
    "panic_handler",
] }

[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

```rust
#![no_std]
#![no_main]

#[rsdl3::main]
fn main(_args: rsdl3::runtime::Args) -> Result<(), rsdl3::Error> {
    let mut sdl = unsafe { rsdl3::Sdl::init() }?;
    let video = sdl.video()?;

    let _window = video.create_window("rsdl3 app", 800, 600, None)?;

    Ok(())
}
```

`#[rsdl3::main]` exports `SDL_main`. With `runtime_shim`, `rsdl3` compiles a
tiny C shim that includes SDL's `SDL_main.h`, so SDL handles platform startup.

### SDL Callback Mode

Use `#[rsdl3::application]` when SDL should drive your app through callbacks.

```toml
[dependencies]
rsdl3 = { path = "../rsdl3/rsdl3", features = [
    "use_callbacks",
    "runtime_shim",
    "panic_handler",
] }

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

When `runtime_shim` and `use_callbacks` are enabled together, `rsdl3` compiles
the shim with `SDL_MAIN_USE_CALLBACKS=1`. Do not use `#[rsdl3::main]` in
callback mode.

## Features

- `image`: enables SDL_image bindings through `rsdl3-sys/image`.
- `main`: enables SDL entrypoint support and `rsdl3::runtime::Args`.
- `runtime_shim`: compiles and links the bundled SDL main C shim.
- `panic_handler`: provides a default panic handler for final `no_std` apps.
- `use_callbacks`: enables `#[rsdl3::application]` and SDL callback mode.

There are no default features. Choose classic mode with `main`, or callback mode
with `use_callbacks`. Callback mode intentionally hides `#[rsdl3::main]`.

## Custom Shim

Most applications should use `runtime_shim`. If your final project must compile
the SDL main shim with a custom C compiler or SDK, disable `runtime_shim` and
provide the shim yourself.

Classic mode shim:

```c
#define SDL_MAIN_AVAILABLE 1
#include <SDL3/SDL_main.h>
```

Callback mode shim:

```c
#define SDL_MAIN_USE_CALLBACKS 1
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

The `panic_handler` feature logs the panic through SDL logging and exits with
status `1`.

If your platform needs different behavior, omit `panic_handler` and define your
own handler in the final application.

```rust
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {}
}
```
