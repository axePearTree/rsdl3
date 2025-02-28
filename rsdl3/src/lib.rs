// #![no_std]

extern crate alloc;

pub mod blendmode;
pub mod events;
#[cfg(feature = "image")]
pub mod image;
pub mod init;
pub mod pixels;
pub mod rect;
pub mod render;
pub mod surface;
pub mod video;

use alloc::borrow::ToOwned;
use alloc::string::String;
use core::ffi::CStr;
use core::num::TryFromIntError;
pub use rsdl3_sys as sys;

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Error(String);

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    pub(crate) fn from_sdl() -> Self {
        unsafe {
            let err = sys::SDL_GetError();
            Error(CStr::from_ptr(err as *const _).to_str().unwrap().to_owned())
        }
    }
}

impl core::error::Error for Error {}

impl From<alloc::ffi::NulError> for Error {
    fn from(_value: alloc::ffi::NulError) -> Self {
        Self(String::from("Interior null byte found in string."))
    }
}

impl From<TryFromIntError> for Error {
    fn from(_value: TryFromIntError) -> Self {
        Self(String::from("Integer conversion failed."))
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
