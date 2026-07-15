use core::alloc::{GlobalAlloc, Layout};

pub struct SDLAllocator;

unsafe impl GlobalAlloc for SDLAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { crate::sys::SDL_malloc(layout.size()).cast() }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe { crate::sys::SDL_free(ptr.cast()) }
    }
}
