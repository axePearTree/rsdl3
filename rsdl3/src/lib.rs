#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

extern crate alloc;

pub mod video;

pub mod blendmode;
pub mod camera;
pub mod clipboard;
pub mod events;
mod init;
pub mod iostream;
pub mod logs;
pub mod pixels;
pub mod rect;
pub mod render;
pub mod surface;

use core::ffi::CStr;
use core::marker::PhantomData;

use alloc::string::String;
use alloc::string::ToString;
pub use init::*;
pub use rsdl3_sys as sys;

/// Zero-sized error type for any operations involving SDL.
///
/// The actual error message is stored by SDL and can be retrieved by calling
/// [`get_error`].
#[allow(unused)]
#[derive(Clone)]
pub struct Error {
    _m: PhantomData<*const ()>, // !Send + !Sync
}

impl Error {
    pub fn new() -> Self {
        Self { _m: PhantomData }
    }

    /// This methods sets SDL's internal error message .
    pub(crate) fn register(err: &CStr) -> Self {
        unsafe { sys::SDL_SetError(err.as_ptr()) };
        Self { _m: PhantomData }
    }
}

impl core::error::Error for Error {}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SDL Error")
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SDL Error")
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

impl From<Error> for String {
    fn from(_: Error) -> Self {
        get_error().unwrap_or(String::from("No error reported in SDL."))
    }
}

pub fn get_error() -> Option<String> {
    unsafe {
        let msg = sys::SDL_GetError();
        if msg.is_null() {
            return None;
        }
        Some(CStr::from_ptr(msg).to_string_lossy().into_owned())
    }
}

/// Returns the version of SDL that is linked against your program.
///
/// If you are linking to SDL dynamically, then it is possible that the current version will be
/// different than the version you compiled against.
pub fn version() -> i32 {
    unsafe { sys::SDL_GetVersion() }
}

/// Get the code revision of SDL that is linked against your program.
///
/// This value is the revision of the code you are linked with and may be different from the code
/// you are compiling with. The revision is arbitrary string (a hash value) uniquely identifying
/// the exact revision of the SDL library in use, and is only useful in comparing against other
/// revisions. It is NOT an incrementing number.
///
/// If SDL wasn't built from a git repository with the appropriate tools, this will return an
/// empty string.
///
/// You shouldn't use this function for anything but logging it for debugging purposes. The string
/// is not intended to be reliable in any way.
pub fn revision() -> String {
    unsafe {
        CStr::from_ptr(sys::SDL_GetRevision())
            .to_string_lossy()
            .to_string()
    }
}
