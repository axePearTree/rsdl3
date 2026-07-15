#[cfg(feature = "app")]
mod app;

use core::ffi::{c_char, c_int};

#[cfg(feature = "callbacks")]
pub mod callbacks;

#[cfg(feature = "callbacks")]
pub use rsdl3_macros::application;

#[cfg(not(feature = "callbacks"))]
pub use rsdl3_macros::main;

#[link(name = "c")]
unsafe extern "C" {}

#[cfg(target_env = "gnu")]
#[link(name = "gcc_s")]
unsafe extern "C" {}

#[derive(Copy, Clone)]
pub struct Args {
    argc: c_int,
    argv: *mut *mut c_char,
}

impl Args {
    pub const fn from_raw(argc: c_int, argv: *mut *mut c_char) -> Self {
        Self { argc, argv }
    }

    #[inline]
    pub const fn argc(&self) -> c_int {
        self.argc
    }

    #[inline]
    pub const fn argv(&self) -> *mut *mut c_char {
        self.argv
    }
}
