#![allow(unused)]

use core::ffi::c_int;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

use alloc::ffi::CString;
use alloc::string::String;
use alloc::sync::Arc;

use crate::init::VideoSubsystem;
use crate::{sys, Error};

impl VideoSubsystem {
    pub fn create_window(
        &self,
        name: &str,
        width: u32,
        height: u32,
        flags: WindowFlags,
    ) -> Result<Window, Error> {
        let c_string =
            CString::new(name).map_err(|_| Error(String::from("Window name is not valid.")))?;
        let c_str = c_string.as_c_str();
        let width =
            c_int::try_from(width).map_err(|_| Error(String::from("Window width is too big.")))?;
        let height = c_int::try_from(height)
            .map_err(|_| Error(String::from("Window height is too big.")))?;
        // SAFETY: the existence of &self guarantees that SDL has been initialized.
        // The string pointer refers to valid memory (c_string/c_str).
        let ptr = unsafe { sys::video::SDL_CreateWindow(c_str.as_ptr(), width, height, flags.0) };
        if ptr.is_null() {
            return Err(Error::from_sdl());
        }
        Ok(Window(Arc::new(WindowInner {
            video: VideoSubsystem(Arc::clone(&self.0)),
            ptr,
            flags,
        })))
    }
}

// We're refcounting SDL_Windows because of SDL_Renderers.
// The window must outlive any SDL_Renderer it creates.
// The alternative to refcounting would be to encode the window's lifetime into the Renderer.
// That wouldn't be as good in terms of usability.
pub struct Window(pub(crate) Arc<WindowInner>);

pub struct WindowInner {
    video: VideoSubsystem,
    ptr: *mut sys::video::SDL_Window,
    flags: WindowFlags,
}

impl Drop for WindowInner {
    fn drop(&mut self) {
        // SAFETY: by keeping a live reference to the VideoSubsystem we guarantee that both SDL and
        // the Video subsystem are valid. The window pointer is never cloned, copied or passed around.
        // Therefore destroying this window using this pointer is a safe operation.
        unsafe { sys::video::SDL_DestroyWindow(self.ptr) };
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct WindowFlags(sys::video::SDL_WindowFlags);

impl WindowFlags {
    pub const FULLSCREEN: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_FULLSCREEN);
    pub const OPEN_GL: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_OPENGL);
    pub const OCCLUDED: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_OCCLUDED);
    pub const HIDDEN: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_HIDDEN);
    pub const BORDERLESS: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_BORDERLESS);
    pub const RESIZABLE: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_RESIZABLE);
    pub const MINIMIZED: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_MINIMIZED);
    pub const MAXIMIZED: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_MAXIMIZED);
    pub const MOUSE_GRABBED: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_MOUSE_GRABBED);
    pub const INPUT_FOCUS: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_INPUT_FOCUS);
    pub const MOUSE_FOCUS: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_MOUSE_FOCUS);
    pub const EXTERNAL: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_EXTERNAL);
    pub const MODAL: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_MODAL);
    pub const HIGH_PIXEL_DENSITY: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_HIGH_PIXEL_DENSITY);
    pub const MOUSE_CAPTURE: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_MOUSE_CAPTURE);
    pub const ALWAYS_ON_TOP: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_ALWAYS_ON_TOP);
    pub const UTILITY: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_UTILITY);
    pub const TOOLTIP: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_TOOLTIP);
    pub const POPUP_MENU: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_POPUP_MENU);
    pub const KEYBOARD_GRABBED: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_KEYBOARD_GRABBED);
    pub const VULKAN: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_VULKAN);
    pub const METAL: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_METAL);
    pub const TRANSPARENT: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_TRANSPARENT);
    pub const NOT_FOCUSABLE: WindowFlags = WindowFlags(sys::video::SDL_WINDOW_NOT_FOCUSABLE);
}

impl BitOr for WindowFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        WindowFlags(self.0 | rhs.0)
    }
}

impl BitOr for &WindowFlags {
    type Output = WindowFlags;

    fn bitor(self, rhs: Self) -> Self::Output {
        WindowFlags(self.0 | rhs.0)
    }
}

impl BitOrAssign for WindowFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 = self.0 | rhs.0;
    }
}

impl BitAnd for WindowFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        WindowFlags(self.0 & rhs.0)
    }
}

impl BitAnd for &WindowFlags {
    type Output = WindowFlags;

    fn bitand(self, rhs: Self) -> Self::Output {
        WindowFlags(self.0 & rhs.0)
    }
}

impl BitAndAssign for WindowFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl Default for WindowFlags {
    fn default() -> Self {
        Self(0)
    }
}
