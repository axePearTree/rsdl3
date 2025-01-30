#![allow(unused)]

use core::ffi::{c_int, c_void, CStr};
use core::marker::PhantomData;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

use alloc::ffi::CString;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::init::VideoSubsystem;
use crate::rect::Rect;
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
        let ptr = unsafe { sys::video::SDL_CreateWindow(c_str.as_ptr(), width, height, flags.0) };
        if ptr.is_null() {
            return Err(Error::from_sdl());
        }
        Ok(Window {
            video: VideoSubsystem(Arc::clone(&self.0)),
            ptr,
        })
    }

    pub fn displays(&self) -> Result<Vec<u32>, Error> {
        let mut num_displays = 0;
        unsafe {
            let displays = sys::video::SDL_GetDisplays(&raw mut num_displays);
            if displays.is_null() {
                return Err(Error::from_sdl());
            }
            let vec = core::slice::from_raw_parts(displays, num_displays as usize).to_vec();
            sys::stdinc::SDL_free(displays as *mut c_void);
            Ok(vec)
        }
    }

    pub fn display_name(&self, display_id: u32) -> Result<String, Error> {
        unsafe {
            let name = sys::video::SDL_GetDisplayName(display_id);
            let c_str = CStr::from_ptr(name);
            Ok(c_str.to_string_lossy().into_owned())
        }
    }

    pub fn display_bounds(&self, display_id: u32) -> Result<Rect, Error> {
        let mut rect = Rect::new(0, 0, 0, 0).raw();
        let result = unsafe { sys::video::SDL_GetDisplayBounds(display_id, &raw mut rect) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(Rect::new(rect.x, rect.y, rect.w as u32, rect.h as u32))
    }

    pub fn enable_screensaver(&self) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_EnableScreenSaver() };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn disable_screensaver(&self) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_DisableScreenSaver() };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }
}

pub struct Window {
    video: VideoSubsystem,
    ptr: *mut sys::video::SDL_Window,
    // This pointer should be safe to dereference while the window is still alive.
}

impl Window {
    pub fn id(&self) -> Result<u32, Error> {
        let id = unsafe { sys::video::SDL_GetWindowID(self.ptr) };
        if id == 0 {
            return Err(Error::from_sdl());
        }
        Ok(id)
    }

    pub fn display(&self) -> Result<u32, Error> {
        let id = unsafe { sys::video::SDL_GetDisplayForWindow(self.ptr) };
        if id == 0 {
            return Err(Error::from_sdl());
        }
        Ok(id)
    }

    pub fn aspect_ratio(&self) -> Result<(f32, f32), Error> {
        let mut min = 0.0;
        let mut max = 0.0;
        let result =
            unsafe { sys::video::SDL_GetWindowAspectRatio(self.ptr, &raw mut min, &raw mut max) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok((min, max))
    }

    pub fn show(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_ShowWindow(self.ptr) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn hide(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_HideWindow(self.ptr) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn flags(&self) -> WindowFlags {
        let result = unsafe { sys::video::SDL_GetWindowFlags(self.ptr) };
        WindowFlags(result)
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_SetWindowFullscreen(self.ptr, fullscreen) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn opacity(&self) -> Result<f32, Error> {
        let result = unsafe { sys::video::SDL_GetWindowOpacity(self.ptr) };
        if result == -1.0 {
            return Err(Error::from_sdl());
        }
        Ok(result)
    }

    pub fn set_opacity(&mut self, opacity: f32) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_SetWindowOpacity(self.ptr, opacity) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn position(&self) -> Result<(i32, i32), Error> {
        let mut x = 0;
        let mut y = 0;
        let result = unsafe { sys::video::SDL_GetWindowPosition(self.ptr, &raw mut x, &raw mut y) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok((x, y))
    }

    pub fn set_position(&mut self, x: i32, y: i32) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_SetWindowPosition(self.ptr, x, y) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn size(&self) -> Result<(i32, i32), Error> {
        let mut x = 0;
        let mut y = 0;
        let result = unsafe { sys::video::SDL_GetWindowSize(self.ptr, &raw mut x, &raw mut y) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok((x, y))
    }

    pub fn set_size(&mut self, x: i32, y: i32) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_SetWindowSize(self.ptr, x, y) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn title(&self) -> Result<String, Error> {
        let c_str = unsafe {
            let ptr = sys::video::SDL_GetWindowTitle(self.ptr);
            CStr::from_ptr(ptr)
        };
        Ok(c_str.to_string_lossy().into_owned())
    }

    pub fn set_title(&self, title: impl Into<String>) -> Result<(), Error> {
        let s: String = title.into();
        let c_string = CString::new(s).map_err(|_| Error("Invalid string title.".into()))?;
        let c_str = c_string.as_c_str();
        let result = unsafe { sys::video::SDL_SetWindowTitle(self.ptr, c_str.as_ptr()) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn set_resizable(&mut self, resizable: bool) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_SetWindowResizable(self.ptr, resizable) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn max_size(&mut self) -> Result<(i32, i32), Error> {
        let mut x = 0;
        let mut y = 0;
        let result =
            unsafe { sys::video::SDL_GetWindowMaximumSize(self.ptr, &raw mut x, &raw mut y) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok((x, y))
    }

    pub fn min_size(&mut self) -> Result<(i32, i32), Error> {
        let mut x = 0;
        let mut y = 0;
        let result =
            unsafe { sys::video::SDL_GetWindowMinimumSize(self.ptr, &raw mut x, &raw mut y) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok((x, y))
    }

    pub fn flash(&mut self, operation: WindowFlashOperation) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_FlashWindow(self.ptr, operation.0) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn maximize(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_MaximizeWindow(self.ptr) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn minimize(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_MinimizeWindow(self.ptr) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn raise(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_RaiseWindow(self.ptr) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn restore(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_RestoreWindow(self.ptr) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn as_ptr(&self) -> *const sys::video::SDL_Window {
        self.ptr as *const sys::video::SDL_Window
    }

    pub fn as_mut_ptr(&self) -> *mut sys::video::SDL_Window {
        self.ptr
    }
}

impl Drop for Window {
    fn drop(&mut self) {
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
    pub const HIGH_PIXEL_DENSITY: WindowFlags =
        WindowFlags(sys::video::SDL_WINDOW_HIGH_PIXEL_DENSITY);
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

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowFlashOperation(sys::video::SDL_FlashOperation);

impl WindowFlashOperation {
    pub const CANCEL: Self = Self(sys::video::SDL_FlashOperation::CANCEL);
    pub const BRIEFLY: Self = Self(sys::video::SDL_FlashOperation::BRIEFLY);
    pub const UNTIL_FOCUSED: Self = Self(sys::video::SDL_FlashOperation::UNTIL_FOCUSED);
}
