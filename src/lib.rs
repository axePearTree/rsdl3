#![no_std]

extern crate alloc;

pub mod init;
pub mod pixels;
pub mod rect;
pub mod video;

use alloc::{borrow::ToOwned, string::String};
use core::ffi::CStr;
pub use sdl3_sys as sys;

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Error(String);

impl Error {
    pub(crate) fn from_sdl() -> Self {
        unsafe {
            let err = sdl3_sys::error::SDL_GetError();
            Error(CStr::from_ptr(err as *const _).to_str().unwrap().to_owned())
        }
    }
}

impl core::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
