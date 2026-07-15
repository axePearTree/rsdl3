use core::panic::PanicInfo;

use crate::allocator::SDLAllocator;

#[global_allocator]
static ALLOCATOR: SDLAllocator = SDLAllocator;

#[unsafe(no_mangle)]
extern "C" fn rust_eh_personality() {}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    use crate::logs::LogCategory;
    let message = info.message();
    crate::log_error!(LogCategory::Error, "{}", message);
    unsafe { libc::exit(1) }
}
