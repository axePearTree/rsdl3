use crate::init::SdlDrop;
use crate::sys;
use crate::{init::Sdl, Error};
use alloc::ffi::CString;
use alloc::rc::Rc;
use core::ffi::c_void;
use core::marker::PhantomData;

/// An interface for reading and writing data streams.
pub struct IOStream<'a> {
    _sdl: Rc<SdlDrop>,
    ptr: *mut sys::SDL_IOStream,
    _m: PhantomData<&'a ()>,
}

impl IOStream<'static> {
    /// Opens a file and returns a read-write `IOStream`.
    pub fn from_file(sdl: &Sdl, file: &str, mode: &str) -> Result<Self, Error> {
        let file = CString::new(file)?;
        let mode = CString::new(mode)?;
        let ptr = unsafe { sys::SDL_IOFromFile(file.as_ptr(), mode.as_ptr()) };
        if ptr.is_null() {
            return Err(Error);
        }
        Ok(IOStream {
            _sdl: Rc::clone(&sdl.drop),
            ptr,
            _m: PhantomData,
        })
    }
}

impl<'a> IOStream<'a> {
    /// Creates a new `IOStream` from an existing mutable byte buffer.
    pub fn from_bytes_mut(sdl: &Sdl, bytes: &'a mut [u8]) -> Result<Self, Error> {
        let ptr = unsafe { sys::SDL_IOFromMem(bytes.as_mut_ptr() as *mut c_void, bytes.len()) };
        Ok(IOStream {
            _sdl: Rc::clone(&sdl.drop),
            ptr,
            _m: PhantomData,
        })
    }

    /// Creates an `IOStream` from an existing read-only buffer.
    pub fn from_bytes(sdl: &Sdl, bytes: &'a [u8]) -> Result<Self, Error> {
        let ptr = unsafe { sys::SDL_IOFromConstMem(bytes.as_ptr() as *const c_void, bytes.len()) };
        Ok(IOStream {
            _sdl: Rc::clone(&sdl.drop),
            ptr,
            _m: PhantomData,
        })
    }

    #[inline]
    pub fn raw(&self) -> *mut sys::SDL_IOStream {
        self.ptr
    }
}

impl<'a> Drop for IOStream<'a> {
    fn drop(&mut self) {
        // SAFETY:
        // SDL is guaranteed to live beyond IOStream's lifetime via _drop.
        // The ptr is owned by the IOStream and not shared.
        unsafe { sys::SDL_CloseIO(self.ptr) };
    }
}
