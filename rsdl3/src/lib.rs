#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

extern crate alloc;

pub mod blendmode;
pub mod events;
pub mod iostream;
pub mod pixels;
pub mod rect;
pub mod render;
pub mod surface;
pub mod video;

mod init;

use core::ffi::CStr;

pub use init::*;
pub use rsdl3_sys as sys;

/// Zero-sized error type for any operations involving SDL.
///
/// The actual error message is stored inside SDL and retrieved when `Display::display` gets called.
#[allow(unused)]
#[derive(Clone)]
pub struct Error;

impl Error {
    /// This methods sets SDL's internal error message .
    pub(crate) fn register(err: &CStr) -> Self {
        unsafe { sys::SDL_SetError(err.as_ptr()) };
        Self
    }
}

impl core::error::Error for Error {}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe {
            let err = sys::SDL_GetError();
            if err.is_null() {
                return write!(f, "NULL");
            }
            let str = CStr::from_ptr(err as *const _);
            write!(f, "{:?}", str)
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

impl From<alloc::ffi::NulError> for Error {
    fn from(_: alloc::ffi::NulError) -> Self {
        static ERROR_MESSAGE: &CStr = c"alloc::ffi::NulError";
        Error::register(ERROR_MESSAGE)
    }
}

impl From<core::num::TryFromIntError> for Error {
    fn from(_value: core::num::TryFromIntError) -> Self {
        static ERROR_MESSAGE: &CStr = c"alloc::ffi::NulError";
        Error::register(ERROR_MESSAGE)
    }
}
