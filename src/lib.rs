#![no_std]

extern crate alloc;

pub mod init;
pub mod video;

pub use sdl3_sys as sys;
use alloc::{borrow::ToOwned, string::String};
use core::ffi::CStr;

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
