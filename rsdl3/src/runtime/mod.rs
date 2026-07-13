#[cfg(any(feature = "runtime", feature = "panic_handler"))]
use core::alloc::{GlobalAlloc, Layout};
use core::ffi::{c_char, c_int};
#[cfg(feature = "panic_handler")]
use core::panic::PanicInfo;

pub use rsdl3_macros::main;

#[cfg(any(feature = "runtime", feature = "panic_handler"))]
#[global_allocator]
static ALLOCATOR: SDLAllocator = SDLAllocator;

#[cfg(any(feature = "runtime", feature = "panic_handler"))]
struct SDLAllocator;

#[cfg(any(feature = "runtime", feature = "panic_handler"))]
unsafe impl GlobalAlloc for SDLAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { crate::sys::SDL_malloc(layout.size()).cast() }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe { crate::sys::SDL_free(ptr.cast()) }
    }
}

#[cfg(any(feature = "runtime", feature = "panic_handler"))]
#[link(name = "c")]
unsafe extern "C" {}

#[cfg(all(
    any(feature = "runtime", feature = "panic_handler"),
    target_env = "gnu"
))]
#[link(name = "gcc_s")]
unsafe extern "C" {}

#[cfg(any(feature = "runtime", feature = "panic_handler"))]
#[unsafe(no_mangle)]
extern "C" fn rust_eh_personality() {}

#[cfg(feature = "panic_handler")]
#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    use crate::logs::LogCategory;
    let message = info.message();
    crate::log_error!(LogCategory::Error, "{}", message);
    unsafe { libc::exit(1) }
}

#[derive(Copy, Clone)]
pub struct Args {
    argc: c_int,
    argv: *mut *mut c_char,
}

impl Args {
    pub const fn from_raw(argc: c_int, argv: *mut *mut c_char) -> Self {
        Self { argc, argv }
    }

    #[inline]
    pub const fn argc(&self) -> c_int {
        self.argc
    }

    #[inline]
    pub const fn argv(&self) -> *mut *mut c_char {
        self.argv
    }
}
