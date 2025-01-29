#![no_std]

use core::ffi::CStr;
use alloc::{borrow::ToOwned, string::String};

extern crate alloc;

pub mod init;
pub use sdl3_sys as sys;

#[derive(Clone, Debug)]
pub struct Error(pub(crate) String);

impl Error {
    pub(crate) fn from_sdl() -> Self {
        unsafe {
            let err = sdl3_sys::error::SDL_GetError();
            Error(CStr::from_ptr(err as *const _).to_str().unwrap().to_owned())
        }
    }
}

pub struct Sdl {
}
