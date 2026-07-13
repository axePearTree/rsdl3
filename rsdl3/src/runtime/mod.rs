use core::alloc::{GlobalAlloc, Layout};
use core::ffi::{c_char, c_int};
use core::panic::PanicInfo;

pub use rsdl3_macros::main;

#[global_allocator]
static ALLOCATOR: SDLAllocator = SDLAllocator;

struct SDLAllocator;

unsafe impl GlobalAlloc for SDLAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { crate::sys::SDL_malloc(layout.size()).cast() }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe { crate::sys::SDL_free(ptr.cast()) }
    }
}

#[link(name = "c")]
unsafe extern "C" {}

#[cfg(target_env = "gnu")]
#[link(name = "gcc_s")]
unsafe extern "C" {}

#[unsafe(no_mangle)]
extern "C" fn rust_eh_personality() {}

#[cfg(not(feature = "no_panic_handler"))]
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
