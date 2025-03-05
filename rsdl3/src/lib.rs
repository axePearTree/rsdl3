#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

extern crate alloc;

#[cfg(feature = "image")]
pub mod image;

pub mod blendmode;
pub mod events;
pub mod iostream;
pub mod pixels;
pub mod rect;
pub mod render;
pub mod surface;
pub mod video;

mod init;

use alloc::borrow::ToOwned;
use alloc::string::String;
use core::ffi::CStr;
pub use init::*;
pub use rsdl3_sys as sys;

/// Error type for any operations involving SDL.
/// This type also includes variants for conversion errors that happen outside of SDL.
#[allow(unused)]
#[derive(Clone, Debug)]
pub enum Error {
    SdlError(String),
    SdlAlreadyInitialized,
    EventPumpAlreadyBorrowed,
    RendererAlreadyDestroyed,
    TextureFromDifferentRenderer,
    UnknownBlendMode(sys::SDL_BlendMode),
    UnknownScaleMode(sys::SDL_ScaleMode),
    UnknownDisplayOrientation(sys::SDL_DisplayOrientation),
    UnknownSurfaceVsyncType(i32),
    InvalidSystemTheme,
    NulError(alloc::ffi::NulError),
    TryFromIntError,
}

impl Error {
    pub(crate) fn from_sdl() -> Self {
        unsafe {
            let err = sys::SDL_GetError();
            Error::SdlError(CStr::from_ptr(err as *const _).to_str().unwrap().to_owned())
        }
    }
}

impl core::error::Error for Error {}

impl From<alloc::ffi::NulError> for Error {
    fn from(value: alloc::ffi::NulError) -> Self {
        Self::NulError(value)
    }
}

impl From<core::num::TryFromIntError> for Error {
    fn from(_value: core::num::TryFromIntError) -> Self {
        Self::TryFromIntError
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
