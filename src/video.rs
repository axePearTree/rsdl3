#![allow(unused)]

use core::ffi::{c_int, c_void, CStr};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

use alloc::ffi::CString;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::init::VideoSubsystem;
use crate::pixels::PixelFormat;
use crate::rect::{Point, Rect};
use crate::render::WindowRenderer;
use crate::surface::{Surface, SurfaceMut, SurfaceOwned, SurfaceRef};
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

    pub fn create_surface(
        &self,
        w: u32,
        h: u32,
        format: PixelFormat,
    ) -> Result<SurfaceOwned, Error> {
        SurfaceOwned::new(self, w, h, format)
    }

    pub fn num_drivers(&self) -> usize {
        unsafe { sys::video::SDL_GetNumVideoDrivers() as usize }
    }

    pub fn driver(&self, driver_index: i32) -> Result<String, Error> {
        unsafe {
            let ptr = sys::video::SDL_GetVideoDriver(driver_index as i32);
            if ptr.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }

    pub fn drivers(&self) -> impl Iterator<Item = Result<String, Error>> + use<'_> {
        let num_drivers = self.num_drivers() as i32;
        (0..num_drivers).map(|i| self.driver(i))
    }

    pub fn current_driver(&self) -> Result<String, Error> {
        unsafe {
            let ptr = sys::video::SDL_GetCurrentVideoDriver();
            if ptr.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
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

    pub fn primary_display(&self) -> Result<u32, Error> {
        let result = unsafe { sys::video::SDL_GetPrimaryDisplay() };
        if result == 0 {
            return Err(Error::from_sdl());
        }
        Ok(result)
    }

    pub fn display_name(&self, display_id: u32) -> Result<String, Error> {
        unsafe {
            let name = sys::video::SDL_GetDisplayName(display_id);
            let c_str = CStr::from_ptr(name);
            Ok(c_str.to_string_lossy().into_owned())
        }
    }

    pub fn display_bounds(&self, display_id: u32) -> Result<Rect, Error> {
        let mut rect = Rect::new(0, 0, 0, 0).to_ll();
        let result = unsafe { sys::video::SDL_GetDisplayBounds(display_id, &raw mut rect) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(Rect::new(rect.x, rect.y, rect.w as u32, rect.h as u32))
    }

    pub fn display_usable_bounds(&self, display_id: u32) -> Result<Rect, Error> {
        let mut out: MaybeUninit<sys::rect::SDL_Rect> = MaybeUninit::uninit();
        unsafe {
            let result = sys::video::SDL_GetDisplayUsableBounds(display_id, out.as_mut_ptr());
            if !result {
                return Err(Error::from_sdl());
            }
            let out = out.assume_init();
            Ok(Rect::from_ll(out))
        }
    }

    pub fn display_for_rect(&self, rect: &Rect) -> Result<u32, Error> {
        let rect = rect.to_ll();
        let display_id = unsafe { sys::video::SDL_GetDisplayForRect(&raw const rect) };
        if display_id == 0 {
            return Err(Error::from_sdl());
        }
        Ok(display_id)
    }

    pub fn display_for_point(&self, point: &Point) -> Result<u32, Error> {
        let point = point.to_ll();
        let display_id = unsafe { sys::video::SDL_GetDisplayForPoint(&raw const point) };
        if display_id == 0 {
            return Err(Error::from_sdl());
        }
        Ok(display_id)
    }

    pub fn display_content_scale(&self, display_id: u32) -> Result<f32, Error> {
        let scale = unsafe { sys::video::SDL_GetDisplayContentScale(display_id) };
        if scale == 0.0 {
            return Err(Error::from_sdl());
        }
        Ok(scale)
    }

    pub fn desktop_display_mode(&self, display_id: u32) -> Result<DisplayMode, Error> {
        unsafe {
            let ptr = sys::video::SDL_GetDesktopDisplayMode(display_id);
            if ptr.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(DisplayMode::from_ptr(ptr))
        }
    }

    pub fn current_display_mode(&self, display_id: u32) -> Result<DisplayMode, Error> {
        unsafe {
            let ptr = sys::video::SDL_GetCurrentDisplayMode(display_id);
            if ptr.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(DisplayMode::from_ptr(ptr))
        }
    }

    pub fn current_display_orientation(
        &self,
        display_id: u32,
    ) -> Result<DisplayOrientation, Error> {
        Ok(DisplayOrientation(unsafe {
            sys::video::SDL_GetCurrentDisplayOrientation(display_id)
        }))
    }

    pub fn natural_display_orientation(
        &self,
        display_id: u32,
    ) -> Result<DisplayOrientation, Error> {
        Ok(DisplayOrientation(unsafe {
            sys::video::SDL_GetNaturalDisplayOrientation(display_id)
        }))
    }

    pub fn closest_fullscreen_display_mode(
        &self,
        display_id: u32,
        w: i32,
        h: i32,
        refresh_rate: f32,
        include_high_density_modes: bool,
    ) -> Result<DisplayMode, Error> {
        unsafe {
            let mut out: MaybeUninit<sys::video::SDL_DisplayMode> = MaybeUninit::uninit();
            let result = sys::video::SDL_GetClosestFullscreenDisplayMode(
                display_id,
                w,
                h,
                refresh_rate,
                include_high_density_modes,
                out.as_mut_ptr(),
            );
            if !result {
                return Err(Error::from_sdl());
            }
            let out = out.assume_init();
            let display_mode = DisplayMode::from_ptr(&raw const out);
            Ok(display_mode)
        }
    }

    pub fn screensaver_enabled(&self) -> bool {
        unsafe { sys::video::SDL_ScreenSaverEnabled() }
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
    // This pointer should be safe to dereference as long as the window is still alive.
}

impl Window {
    pub fn create_renderer(self, driver: Option<&str>) -> Result<WindowRenderer, Error> {
        WindowRenderer::new(self, driver)
    }

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

    pub fn display_scale(&self) -> Result<f32, Error> {
        let scale = unsafe { sys::video::SDL_GetWindowDisplayScale(self.ptr) };
        if scale == 0.0 {
            return Err(Error::from_sdl());
        }
        Ok(scale)
    }

    pub fn destroy_surface(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_DestroyWindowSurface(self.ptr) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn has_surface(&self) -> bool {
        unsafe { sys::video::SDL_WindowHasSurface(self.ptr) }
    }

    pub fn surface_ref(&self) -> Result<SurfaceRef, Error> {
        unsafe {
            let surface = sys::video::SDL_GetWindowSurface(self.ptr);
            if surface.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(SurfaceRef::from_mut_ptr(surface))
        }
    }

    pub fn surface_mut(&mut self) -> Result<SurfaceMut, Error> {
        unsafe {
            let surface = sys::video::SDL_GetWindowSurface(self.ptr);
            if surface.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(SurfaceMut::from_mut_ptr(surface))
        }
    }

    pub fn mouse_rect(&self) -> Result<Rect, Error> {
        unsafe {
            let result = sys::video::SDL_GetWindowMouseRect(self.ptr);
            if result.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(Rect::from_ll(*result))
        }
    }

    pub fn set_mouse_rect(&mut self, rect: Rect) -> Result<(), Error> {
        let rect = rect.to_ll();
        let result = unsafe { sys::video::SDL_SetWindowMouseRect(self.ptr, &raw const rect) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
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

    pub fn set_aspect_ratio(&mut self, min_aspect: f32, max_aspect: f32) -> Result<(), Error> {
        let result =
            unsafe { sys::video::SDL_SetWindowAspectRatio(self.ptr, min_aspect, max_aspect) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn show(&self) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_ShowWindow(self.ptr) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn hide(&self) -> Result<(), Error> {
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

    pub fn pixel_format(&self) -> Result<PixelFormat, Error> {
        let result = unsafe { sys::video::SDL_GetWindowPixelFormat(self.ptr) };
        if result == sys::pixels::SDL_PixelFormat::UNKNOWN {
            return Err(Error::from_sdl());
        }
        return Ok(PixelFormat::from_ll(result));
    }

    pub fn safe_area(&self) -> Result<Rect, Error> {
        let mut out: MaybeUninit<sys::rect::SDL_Rect> = MaybeUninit::uninit();
        unsafe {
            let result = unsafe { sys::video::SDL_GetWindowSafeArea(self.ptr, out.as_mut_ptr()) };
            if !result {
                return Err(Error::from_sdl());
            }
            let out = out.assume_init();
            Ok(Rect::from_ll(out))
        }
    }

    pub fn max_size(&self) -> Result<(i32, i32), Error> {
        let mut x = 0;
        let mut y = 0;
        let result =
            unsafe { sys::video::SDL_GetWindowMaximumSize(self.ptr, &raw mut x, &raw mut y) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok((x, y))
    }

    pub fn set_max_size(&mut self, w: u32, h: u32) -> Result<(), Error> {
        let w = w.min(i32::MAX as u32) as i32;
        let h = h.min(i32::MAX as u32) as i32;
        let result = unsafe { sys::video::SDL_SetWindowMaximumSize(self.ptr, w, h) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn min_size(&self) -> Result<(i32, i32), Error> {
        let mut x = 0;
        let mut y = 0;
        let result =
            unsafe { sys::video::SDL_GetWindowMinimumSize(self.ptr, &raw mut x, &raw mut y) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok((x, y))
    }

    pub fn set_min_size(&mut self, w: u32, h: u32) -> Result<(), Error> {
        let w = w.min(i32::MAX as u32) as i32;
        let h = h.min(i32::MAX as u32) as i32;
        let result = unsafe { sys::video::SDL_SetWindowMinimumSize(self.ptr, w, h) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn set_bordered(&mut self, bordered: bool) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_SetWindowBordered(self.ptr, bordered) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn set_always_on_top(&mut self, always_on_top: bool) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_SetWindowAlwaysOnTop(self.ptr, always_on_top) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn set_focusable(&mut self, focusable: bool) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_SetWindowFocusable(self.ptr, focusable) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    // SDL mutates the original surface but also creates a copy.
    // So we're free to use it after calling this; hence why it takes a mutable surface as
    // parameter.
    pub fn set_icon(&mut self, icon: &mut Surface) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_SetWindowIcon(self.ptr, icon.as_mut_ptr()) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn set_mouse_grabbed(&mut self, grabbed: bool) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_SetWindowMouseGrab(self.ptr, grabbed) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn is_mouse_grabbed(&self) -> bool {
        unsafe { sys::video::SDL_GetWindowMouseGrab(self.ptr) }
    }

    pub fn pixel_density(&self) -> Result<f32, Error> {
        let pixel_density = unsafe { sys::video::SDL_GetWindowPixelDensity(self.ptr) };
        if pixel_density == 0.0 {
            return Err(Error::from_sdl());
        }
        Ok(pixel_density)
    }

    pub fn size_in_pixels(&self) -> Result<(i32, i32), Error> {
        let mut w = 0;
        let mut h = 0;
        let result =
            unsafe { sys::video::SDL_GetWindowSizeInPixels(self.ptr, &raw mut w, &raw mut h) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok((w, h))
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

    pub fn update_surface(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::video::SDL_UpdateWindowSurface(self.ptr) };
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

    #[inline]
    pub fn to_ll(&self) -> sys::video::SDL_FlashOperation {
        self.0
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct DisplayMode {
    pub id: u32,
    pub format: PixelFormat,
    pub w: i32,
    pub h: i32,
    pub pixel_density: f32,
    pub refresh_rate: f32,
    pub refresh_rate_numerator: i32,
    pub refresh_rate_denominator: i32,
}

impl DisplayMode {
    /// SAFETY: safe to call as long as the pointer is valid.
    /// This copies the contents of *ptr to a new DisplayMode value.
    unsafe fn from_ptr(ptr: *const sys::video::SDL_DisplayMode) -> Self {
        Self {
            id: (*ptr).displayID,
            format: PixelFormat::from_ll((*ptr).format),
            w: (*ptr).w,
            h: (*ptr).h,
            pixel_density: (*ptr).pixel_density,
            refresh_rate: (*ptr).refresh_rate,
            refresh_rate_numerator: (*ptr).refresh_rate_numerator,
            refresh_rate_denominator: (*ptr).refresh_rate_denominator,
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayOrientation(sys::video::SDL_DisplayOrientation);

impl DisplayOrientation {
    /// The display orientation can't be determined
    pub const UNKNOWN: Self = Self(sys::video::SDL_DisplayOrientation::UNKNOWN);
    /// The display is in landscape mode, with the right side up, relative to portrait mode
    pub const LANDSCAPE: Self = Self(sys::video::SDL_DisplayOrientation::LANDSCAPE);
    /// The display is in landscape mode, with the left side up, relative to portrait mode
    pub const LANDSCAPE_FLIPPED: Self = Self(sys::video::SDL_DisplayOrientation::LANDSCAPE_FLIPPED);
    /// The display is in portrait mode
    pub const PORTRAIT: Self = Self(sys::video::SDL_DisplayOrientation::PORTRAIT);
    /// The display is in portrait mode, upside down
    pub const PORTRAIT_FLIPPED: Self = Self(sys::video::SDL_DisplayOrientation::PORTRAIT_FLIPPED);

    #[inline]
    pub fn to_ll(&self) -> sys::video::SDL_DisplayOrientation {
        self.0
    }
}
