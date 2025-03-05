use crate::init::VideoSubsystem;
use crate::pixels::{ColorPalette, PixelFormat, PixelFormatRgbaMask};
use crate::rect::{Point, Rect};
use crate::render::Renderer;
use crate::surface::{Surface, SurfaceRef};
use crate::{sys, Error};
use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::{c_int, c_void, CStr};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Deref, DerefMut};

impl VideoSubsystem {
    /// Creates a `Window`.
    /// This method is equivalent to [`Window::new`].
    pub fn create_window(
        &self,
        name: &str,
        width: u32,
        height: u32,
        flags: WindowFlags,
    ) -> Result<Window, Error> {
        Window::new(self, name, width, height, flags)
    }

    /// Creates a `Window`.
    /// This method is equivalent to [`Surface::new`].
    pub fn create_surface(&self, w: u32, h: u32, format: PixelFormat) -> Result<Surface, Error> {
        Surface::new(self, w, h, format)
    }

    /// Creates a `ColorPalette`.
    /// This method is equivalent to [`ColorPalette::new`].
    pub fn create_palette(&self, count: usize) -> Result<ColorPalette, Error> {
        ColorPalette::new(self, count)
    }

    /// Converts an RGBA mask into a `PixelFormat`.
    /// This will return a `PixelFormat::Unknown` if the conversion wasn't possible.
    pub fn pixel_format_for_mask(&self, mask: PixelFormatRgbaMask) -> PixelFormat {
        unsafe {
            let pixel_format = sys::SDL_GetPixelFormatForMasks(
                mask.bpp,
                mask.r_mask,
                mask.g_mask,
                mask.b_mask,
                mask.a_mask,
            );
            PixelFormat::from_ll_unchecked(pixel_format)
        }
    }

    /// Creates a new surface identical to the existing surface.
    /// If the original surface has alternate images, the new surface will have a reference to them as well.
    pub fn duplicate_surface(&self, surface: &SurfaceRef) -> Result<Surface, Error> {
        let ptr = unsafe { sys::SDL_DuplicateSurface(surface.as_ptr() as *mut _) };
        if ptr.is_null() {
            return Err(Error::from_sdl());
        }
        Ok(unsafe { Surface::from_mut_ptr(self, ptr) })
    }

    /// Returns the number of video drivers compiled into SDL.
    pub fn num_drivers(&self) -> Result<usize, Error> {
        Ok(usize::try_from(unsafe { sys::SDL_GetNumVideoDrivers() })?)
    }

    /// Returns the name of a builtin video driver.
    /// The number of drivers can be obtained by calling [`VideoSubsystem::num_drivers`].
    pub fn driver(&self, driver_index: usize) -> Result<String, Error> {
        unsafe {
            let driver_index = i32::try_from(driver_index)?;
            let ptr = sys::SDL_GetVideoDriver(driver_index);
            if ptr.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }

    /// Returns the number of 2D rendering drivers available for the current display.
    pub fn num_render_drivers(&self) -> Result<usize, Error> {
        Ok(usize::try_from(unsafe { sys::SDL_GetNumRenderDrivers() })?)
    }

    /// Returns the name of a builtin render driver.
    /// The number of drivers can be obtained by calling [`VideoSubsystem::num_render_drivers`].
    pub fn render_driver(&self, index: usize) -> Result<String, Error> {
        unsafe {
            let ptr = sys::SDL_GetRenderDriver(i32::try_from(index)?);
            if ptr.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }

    /// Returns the name of the currently initialized video driver.
    pub fn current_driver(&self) -> Result<String, Error> {
        unsafe {
            let ptr = sys::SDL_GetCurrentVideoDriver();
            if ptr.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }

    /// Returns a `Vec<u32>` containing the names of all available displays.
    pub fn displays(&self) -> Result<Vec<u32>, Error> {
        let mut num_displays = 0;
        unsafe {
            let displays = sys::SDL_GetDisplays(&raw mut num_displays);
            if displays.is_null() {
                return Err(Error::from_sdl());
            }
            let vec = core::slice::from_raw_parts(displays, num_displays as usize).to_vec();
            sys::SDL_free(displays as *mut c_void);
            Ok(vec)
        }
    }

    /// Returns the id of the primary display.
    pub fn primary_display(&self) -> Result<u32, Error> {
        let result = unsafe { sys::SDL_GetPrimaryDisplay() };
        if result == 0 {
            return Err(Error::from_sdl());
        }
        Ok(result)
    }

    /// Returns the name of a given display.
    pub fn display_name(&self, display_id: u32) -> Result<String, Error> {
        unsafe {
            let name = sys::SDL_GetDisplayName(display_id);
            let c_str = CStr::from_ptr(name);
            Ok(c_str.to_string_lossy().into_owned())
        }
    }

    /// Returns the desktop area represented by a display.
    /// The primary display is often located at (0,0), but may be placed at a different location depending on monitor layout.
    pub fn display_bounds(&self, display_id: u32) -> Result<Rect, Error> {
        let mut rect = Rect::new(0, 0, 0, 0).to_ll();
        let result = unsafe { sys::SDL_GetDisplayBounds(display_id, &raw mut rect) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(Rect::new(rect.x, rect.y, rect.w as u32, rect.h as u32))
    }

    /// Returns the usable desktop area represented by a display, in screen coordinates.
    /// This is the same area as `VideoSubsystem::display_bounds`, but with portions reserved by the system removed.
    pub fn display_usable_bounds(&self, display_id: u32) -> Result<Rect, Error> {
        let mut out: MaybeUninit<sys::SDL_Rect> = MaybeUninit::uninit();
        unsafe {
            let result = sys::SDL_GetDisplayUsableBounds(display_id, out.as_mut_ptr());
            if !result {
                return Err(Error::from_sdl());
            }
            let out = out.assume_init();
            Ok(Rect::from_ll(out))
        }
    }

    /// Returns the id of the display primarily containing a rect.
    pub fn display_for_rect(&self, rect: &Rect) -> Result<u32, Error> {
        let rect = rect.to_ll();
        let display_id = unsafe { sys::SDL_GetDisplayForRect(&raw const rect) };
        if display_id == 0 {
            return Err(Error::from_sdl());
        }
        Ok(display_id)
    }

    /// Returns the id of the display containing a point.
    pub fn display_for_point(&self, point: &Point) -> Result<u32, Error> {
        let point = point.to_ll();
        let display_id = unsafe { sys::SDL_GetDisplayForPoint(&raw const point) };
        if display_id == 0 {
            return Err(Error::from_sdl());
        }
        Ok(display_id)
    }

    /// Returns the content scale of a display.
    ///
    /// The content scale is the expected scale for content based on the DPI settings of the display.
    ///
    /// For example, a 4K display might have a 2.0 (200%) display scale, which means that the user expects UI elements to be twice as big on this display, to aid in readability.
    ///
    /// After window creation, [`Window::display_scale`] should be used to query the content scale factor for individual windows instead of querying the display for a window and
    /// calling this function, as the per-window content scale factor may differ from the base value of the display it is on, particularly on high-DPI and/or multi-monitor desktop configurations.
    pub fn display_content_scale(&self, display_id: u32) -> Result<f32, Error> {
        let scale = unsafe { sys::SDL_GetDisplayContentScale(display_id) };
        if scale == 0.0 {
            return Err(Error::from_sdl());
        }
        Ok(scale)
    }

    /// Returns information about the desktop's display mode.
    ///
    /// There's a difference between this function and [`VideoSubsystem::current_display_mode`] when SDL runs fullscreen and has changed the resolution.
    ///
    /// In that case this function will return the previous native display mode, and not the current display mode.
    pub fn desktop_display_mode(&self, display_id: u32) -> Result<DisplayMode, Error> {
        unsafe {
            let ptr = sys::SDL_GetDesktopDisplayMode(display_id);
            if ptr.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(DisplayMode::from_ptr(ptr))
        }
    }

    /// Returns a `Vec` containing all of the fullscreen display modes available on a display.
    /// The display modes are sorted in this priority:
    /// - w -> largest to smallest
    /// - h -> largest to smallest
    /// - bits per pixel -> more colors to fewer colors
    /// - packed pixel layout -> largest to smallest
    /// - refresh rate -> highest to lowest
    /// - pixel density -> lowest to highest
    pub fn fullscreen_display_modes(&self, display_id: u32) -> Result<Vec<DisplayMode>, Error> {
        unsafe {
            let mut count = 0;
            let ptr = sys::SDL_GetFullscreenDisplayModes(display_id, &raw mut count);
            if ptr.is_null() {
                return Err(Error::from_sdl());
            }
            let mut display_modes = Vec::new();
            for i in 0..count {
                let display_mode = *ptr.offset(isize::try_from(i)?);
                display_modes.push(DisplayMode::from_ptr(display_mode));
            }
            sys::SDL_free(ptr as *mut c_void);
            Ok(display_modes)
        }
    }

    /// Returns the current display mode.
    /// There's a difference between this function and [`VideoSubsystem::desktop_display_mode`] when SDL runs fullscreen and has changed the resolution.
    /// In that case this\n function will return the current display mode, and not the previous native display mode.
    pub fn current_display_mode(&self, display_id: u32) -> Result<DisplayMode, Error> {
        unsafe {
            let ptr = sys::SDL_GetCurrentDisplayMode(display_id);
            if ptr.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(DisplayMode::from_ptr(ptr))
        }
    }

    /// Returns the orientation of a display.
    pub fn current_display_orientation(
        &self,
        display_id: u32,
    ) -> Result<DisplayOrientation, Error> {
        DisplayOrientation::try_from_ll(unsafe {
            sys::SDL_GetCurrentDisplayOrientation(display_id)
        })
    }

    /// Returns the orientation of a display when it is unrotated.
    pub fn natural_display_orientation(
        &self,
        display_id: u32,
    ) -> Result<DisplayOrientation, Error> {
        DisplayOrientation::try_from_ll(unsafe {
            sys::SDL_GetNaturalDisplayOrientation(display_id)
        })
    }

    /// Returns the closest match to the requested display mode.
    /// The available display modes are scanned and `closest` is filled in with the closest mode matching the requested mode and returned.
    /// The mode format and refresh rate default to the desktop mode if they are set to 0.
    /// The modes are scanned with size being first priority, format being second priority, and finally checking the refresh rate.
    /// If all the available modes are too small, then an `Error` is returned.
    pub fn closest_fullscreen_display_mode(
        &self,
        display_id: u32,
        w: i32,
        h: i32,
        refresh_rate: f32,
        include_high_density_modes: bool,
    ) -> Result<DisplayMode, Error> {
        unsafe {
            let mut out: MaybeUninit<sys::SDL_DisplayMode> = MaybeUninit::uninit();
            let result = sys::SDL_GetClosestFullscreenDisplayMode(
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

    /// Check whether the screensaver is currently enabled. The screensaver is disabled by default.
    pub fn screensaver_enabled(&self) -> bool {
        unsafe { sys::SDL_ScreenSaverEnabled() }
    }

    /// Allow the screen to be blanked by a screen saver.
    pub fn enable_screensaver(&self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_EnableScreenSaver() };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    /// Prevent the screen from being blanked by a screen saver. If you disable the screensaver, it is automatically re-enabled when SDL quits.
    pub fn disable_screensaver(&self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_DisableScreenSaver() };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    /// Returns the current `SystemTheme`.
    pub fn system_theme(&self) -> Result<SysthemTheme, Error> {
        SysthemTheme::try_from_ll(unsafe { sys::SDL_GetSystemTheme() })
    }
}

/// Type used to identify a window.
pub struct Window {
    _video: VideoSubsystem,
    /// This pointer should be safe to dereference as long as the window is still alive.
    ptr: *mut sys::SDL_Window,
}

impl Window {
    /// Creates a new [`Window`].
    pub fn new(
        video: &VideoSubsystem,
        name: &str,
        width: u32,
        height: u32,
        flags: WindowFlags,
    ) -> Result<Window, Error> {
        let c_string = CString::new(name)?;
        let c_str = c_string.as_c_str();
        let width = c_int::try_from(width)?;
        let height = c_int::try_from(height)?;
        let ptr = unsafe { sys::SDL_CreateWindow(c_str.as_ptr(), width, height, flags.0) };
        if ptr.is_null() {
            return Err(Error::from_sdl());
        }
        Ok(Window {
            _video: video.clone(),
            ptr,
        })
    }

    /// Creates a [`Renderer`]. Consumes the [`Window`].
    /// Once the renderer is instantiated, the window can be accessed again via [`Renderer::as_window_mut`] or [`Renderer::as_window_ref`].
    pub fn into_renderer(self, driver: Option<&str>) -> Result<Renderer, Error> {
        Renderer::from_window(self, driver)
    }
}

impl Deref for Window {
    type Target = WindowRef;

    fn deref(&self) -> &Self::Target {
        unsafe { WindowRef::from_ptr(self.ptr) }
    }
}

impl DerefMut for Window {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { WindowRef::from_mut_ptr(self.ptr as *mut _) }
    }
}

/// A reference to a [`Window`].
// We cast pointers to &WindowRef and &mut WindowRef.
// This allows us to safely expose references to a window from a Renderer.
pub struct WindowRef {
    _inner: PhantomData<*const ()>, // !Send + !Sync
}

impl WindowRef {
    pub(crate) unsafe fn from_ptr<'a>(ptr: *const sys::SDL_Window) -> &'a Self {
        &*(ptr as *const Self)
    }

    pub(crate) unsafe fn from_mut_ptr<'a>(ptr: *mut sys::SDL_Window) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }

    /// Returns the numeric ID of this window.
    /// The numeric ID is what [`crate::events::WindowEvent`] references, and is necessary to map
    /// these events to specific `WindowRef` objects.
    pub fn id(&self) -> Result<u32, Error> {
        let id = unsafe { sys::SDL_GetWindowID(self.as_ptr() as *mut _) };
        if id == 0 {
            return Err(Error::from_sdl());
        }
        Ok(id)
    }

    /// Returns the ID of the display associated with a window.
    pub fn display(&self) -> Result<u32, Error> {
        let id = unsafe { sys::SDL_GetDisplayForWindow(self.as_ptr() as *mut _) };
        if id == 0 {
            return Err(Error::from_sdl());
        }
        Ok(id)
    }

    /// Returns the content display scale relative to a `WindowRef`'s pixel size.
    ///
    /// This is a combination of the window pixel density and the display content scale,
    /// and is the expected scale for displaying content in this window.
    ///
    /// For example, if a 3840x2160 window had a display scale of 2.0, the user expects
    /// the content to take twice as many pixels and be the same physical size as if it were being
    /// displayed in a 1920x1080 window with a display scale of 1.0.
    ///
    /// Conceptually this value corresponds to the scale display setting, and is updated
    /// when that setting is changed, or the window moves to a display with a different
    /// scale setting.
    pub fn display_scale(&self) -> Result<f32, Error> {
        let scale = unsafe { sys::SDL_GetWindowDisplayScale(self.as_ptr() as *mut _) };
        if scale == 0.0 {
            return Err(Error::from_sdl());
        }
        Ok(scale)
    }

    /// Return whether the `WindowRef` has a [`Surface`] associated with it.
    pub fn has_surface(&self) -> bool {
        unsafe { sys::SDL_WindowHasSurface(self.as_ptr() as *mut _) }
    }

    /// Returns a reference to a [`SurfaceRef`] associated with the `WindowRef`.
    /// A new surface will be created with the optimal format for the window, if necessary.
    pub fn as_surface_ref(&self) -> Result<&SurfaceRef, Error> {
        unsafe {
            let surface = sys::SDL_GetWindowSurface(self.as_ptr() as *mut _);
            if surface.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(SurfaceRef::from_mut_ptr(surface))
        }
    }

    /// Returns a mutable reference to a [`SurfaceRef`] associated with the `WindowRef`.
    /// A new surface will be created with the optimal format for the window, if necessary.
    pub fn as_surface_mut(&mut self) -> Result<&mut SurfaceRef, Error> {
        unsafe {
            let surface = sys::SDL_GetWindowSurface(self.as_ptr() as *mut _);
            if surface.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(SurfaceRef::from_mut_ptr(surface))
        }
    }

    /// Returns the mouse confinement rectangle of a `WindowRef`.
    pub fn mouse_rect(&self) -> Result<Rect, Error> {
        unsafe {
            let result = sys::SDL_GetWindowMouseRect(self.as_ptr() as *mut _);
            if result.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(Rect::from_ll(*result))
        }
    }

    /// Confines the cursor to the specified area of a window.
    /// Note that this does NOT grab the cursor, it only defines the area a cursor is
    /// restricted to when the window has mouse focus.
    pub fn set_mouse_rect(&mut self, rect: Rect) -> Result<(), Error> {
        let rect = rect.to_ll();
        let result =
            unsafe { sys::SDL_SetWindowMouseRect(self.as_ptr() as *mut _, &raw const rect) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    /// Returns the size of a `Window`'s client area.
    pub fn aspect_ratio(&self) -> Result<(f32, f32), Error> {
        let mut min = 0.0;
        let mut max = 0.0;
        let result = unsafe {
            sys::SDL_GetWindowAspectRatio(self.as_ptr() as *mut _, &raw mut min, &raw mut max)
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok((min, max))
    }

    /// Request that the aspect ratio of a `Window`'s client area be set.
    ///
    /// The aspect ratio is the ratio of width divided by height, e.g. 2560x1600 would be 1.6.
    /// Larger aspect ratios are wider and smaller aspect ratios are narrower.
    ///
    /// If, at the time of this request, the window in a fixed-size state, such as maximized
    /// or fullscreen, the request will be deferred until the window exits this state and becomes resizable again.
    ///
    /// On some windowing systems, this request is asynchronous and the new window aspect ratio may not have have
    /// been applied immediately upon the return of this function. If an immediate change is required, call
    /// [`Window::sync`] to block until the changes have taken effect.
    ///
    /// When the window size changes, an [`crate::events::Event::Window`] event with payload
    /// [`crate::events::WindowEventPayload::Resized`] will be emitted with the new window dimensions. Note that
    /// the new dimensions may not match the exact aspect ratio requested, as some windowing
    /// systems can restrict the window size in certain scenarios (e.g. constraining the size of the content area
    /// to remain within the usable desktop bounds). Additionally, as this is just a request, it can be denied by
    /// the windowing system.
    pub fn set_aspect_ratio(&mut self, min_aspect: f32, max_aspect: f32) -> Result<(), Error> {
        let result = unsafe {
            sys::SDL_SetWindowAspectRatio(self.as_ptr() as *mut _, min_aspect, max_aspect)
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    /// Shows the window.
    pub fn show(&self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_ShowWindow(self.as_ptr() as *mut _) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    /// Hides the window.
    pub fn hide(&self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_HideWindow(self.as_ptr() as *mut _) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    /// Returns the window's [`WindowFlags`].
    pub fn flags(&self) -> WindowFlags {
        let result = unsafe { sys::SDL_GetWindowFlags(self.as_ptr() as *mut _) };
        WindowFlags(result)
    }

    /// Request that the window's fullscreen state be changed.
    ///
    /// By default a window in fullscreen state uses borderless fullscreen desktop mode, but a
    /// specific exclusive display mode can be set using [`WindowRef::select_fullscreen_mode`]
    ///
    /// On some windowing systems this request is asynchronous and the new fullscreen state may
    /// not have have been applied immediately upon the return of this function. If an immediate
    /// change is required, call [`WindowRef::sync`] to block until the changes have taken effect.
    ///
    /// When the window state changes, [`crate::events::Event::Window`] with payload
    /// [`crate::events::Event::WindowEvent::EnterFullscreen`] or  [`crate::events::Event::WindowEvent::LeaveFullscreen`]
    /// will be emitted. Note that, as this is just a request, it can be denied by the windowing system.
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetWindowFullscreen(self.as_ptr() as *mut _, fullscreen) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    /// Query the display mode to use when a window is visible at fullscreen.
    pub fn fullscreen_mode(&self) -> Result<DisplayMode, Error> {
        unsafe {
            let ptr = sys::SDL_GetWindowFullscreenMode(self.as_ptr() as *mut _);
            if ptr.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(DisplayMode::from_ptr(ptr))
        }
    }

    /// Selects one of the available display modes to be this window's fullscreen mode.
    /// NOTE: This method is very different from the original SDL function for memory safety
    /// reasons.
    // TODO: refactor this using ZSTs for DisplayMode.
    pub fn select_fullscreen_mode(
        &mut self,
        display_id: u32,
        select: impl Fn(DisplayMode) -> bool,
    ) -> Result<(), Error> {
        // This method is a kind of a shit show and very different from the original SDL function
        // because the lifetimes of SDL_DisplayModes are somewhat weird.
        // Originally, SDL_SetWindowFullscreenMode takes a *SDL_DisplayMode as a parameter.
        // A *SDL_DisplayMode can be obtained by calling SDL_GetFullscreenDisplayModes.
        // The issue is: the pointer might get invalidated internally by SDL at any time since the
        // underlying values are stored inside a dynamic array that can get reallocated.
        unsafe {
            let mut count = 0;
            let ptr = sys::SDL_GetFullscreenDisplayModes(display_id, &raw mut count);
            if ptr.is_null() {
                return Err(Error::from_sdl());
            }
            let count: usize = count.try_into()?;
            for i in 0..count {
                let display_mode_ptr = *ptr.offset(isize::try_from(i)?);
                let display_mode = DisplayMode::from_ptr(display_mode_ptr);
                if select(display_mode) {
                    let result =
                        sys::SDL_SetWindowFullscreenMode(self.as_ptr() as *mut _, display_mode_ptr);
                    if !result {
                        return Err(Error::from_sdl());
                    }
                    return Ok(());
                }
            }
            Ok(())
        }
    }

    /// Returns the window's opacity.
    pub fn opacity(&self) -> Result<f32, Error> {
        let result = unsafe { sys::SDL_GetWindowOpacity(self.as_ptr() as *mut _) };
        if result == -1.0 {
            return Err(Error::from_sdl());
        }
        Ok(result)
    }

    /// Sets the window's opacity.
    ///
    /// The parameter `opacity` will be clamped internally between 0.0f (transparent) and 1.0f (opaque).
    ///
    /// This function also returns an `Error` if setting the opacity isn't supported.
    pub fn set_opacity(&mut self, opacity: f32) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetWindowOpacity(self.as_ptr() as *mut _, opacity) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    /// Returns the window's position.
    ///
    /// This is the current position of the window as last reported by the windowing system.
    pub fn position(&self) -> Result<(i32, i32), Error> {
        let mut x = 0;
        let mut y = 0;
        let result =
            unsafe { sys::SDL_GetWindowPosition(self.as_ptr() as *mut _, &raw mut x, &raw mut y) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok((x, y))
    }

    /// Request that the window's position be set.
    ///
    /// If the window is in an exclusive fullscreen or maximized state, this request has no effect.
    ///
    /// This can be used to reposition fullscreen-desktop windows onto a different display, however, as exclusive
    /// fullscreen windows are locked to a specific display, they can only be repositioned programmatically via
    /// SDL_SetWindowFullscreenMode().
    ///
    /// On some windowing systems this request is asynchronous and the new coordinates may not have have been
    /// applied immediately upon the return of this function.
    ///
    /// If an immediate change is required, call [`WindowRef::sync`] to block until the changes have taken effect.
    ///
    /// When the window state changes, [`crate::events::Event::Window`] with payload
    /// [`crate::events::Event::WindowEvent::Moved`] will be emitted. Note that, as this is just a request,
    /// it can be denied by the windowing system.
    ///
    /// When the window position changes, an SDL_EVENT_WINDOW_MOVED event will be emitted with the window's new
    /// coordinates. Note that the new coordinates may not match the exact coordinates requested, as some windowing
    /// systems can restrict the position of the window in certain scenarios (e.g. constraining the position so the
    /// window is always within desktop bounds). Additionally, as this is just a request, it can be denied by the
    /// windowing system.
    pub fn set_position(&mut self, x: i32, y: i32) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetWindowPosition(self.as_ptr() as *mut _, x, y) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    /// Returns the size of a window's client area.
    ///
    /// The window pixel size may differ from its window coordinate size if the window is on a high pixel density
    /// display. Use [`WindowRef::size_in_pixels`] or [`crate::render::Renderer::output_size`] to get the real
    /// client area size in pixels.
    pub fn size(&self) -> Result<(i32, i32), Error> {
        let mut x = 0;
        let mut y = 0;
        let result =
            unsafe { sys::SDL_GetWindowSize(self.as_ptr() as *mut _, &raw mut x, &raw mut y) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok((x, y))
    }

    pub fn set_size(&mut self, x: i32, y: i32) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetWindowSize(self.as_ptr() as *mut _, x, y) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn title(&self) -> Result<String, Error> {
        let c_str = unsafe {
            let ptr = sys::SDL_GetWindowTitle(self.as_ptr() as *mut _);
            CStr::from_ptr(ptr)
        };
        Ok(c_str.to_string_lossy().into_owned())
    }

    pub fn set_title(&self, title: impl Into<String>) -> Result<(), Error> {
        let s: String = title.into();
        let c_string = CString::new(s)?;
        let c_str = c_string.as_c_str();
        let result = unsafe { sys::SDL_SetWindowTitle(self.as_ptr() as *mut _, c_str.as_ptr()) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn set_resizable(&mut self, resizable: bool) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetWindowResizable(self.as_ptr() as *mut _, resizable) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn pixel_format(&self) -> Result<PixelFormat, Error> {
        unsafe {
            let result = sys::SDL_GetWindowPixelFormat(self.as_ptr() as *mut _);
            // Even though the unknown PixelFormat is valid SDL tells us to handle it as an error.
            if result == sys::SDL_PixelFormat_SDL_PIXELFORMAT_UNKNOWN {
                return Err(Error::from_sdl());
            }
            return Ok(PixelFormat::from_ll_unchecked(result));
        }
    }

    pub fn safe_area(&self) -> Result<Rect, Error> {
        let mut out: MaybeUninit<sys::SDL_Rect> = MaybeUninit::uninit();
        unsafe {
            let result = sys::SDL_GetWindowSafeArea(self.as_ptr() as *mut _, out.as_mut_ptr());
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
        let result = unsafe {
            sys::SDL_GetWindowMaximumSize(self.as_ptr() as *mut _, &raw mut x, &raw mut y)
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok((x, y))
    }

    pub fn set_max_size(&mut self, w: u32, h: u32) -> Result<(), Error> {
        let w = w.min(i32::MAX as u32) as i32;
        let h = h.min(i32::MAX as u32) as i32;
        let result = unsafe { sys::SDL_SetWindowMaximumSize(self.as_ptr() as *mut _, w, h) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn min_size(&self) -> Result<(i32, i32), Error> {
        let mut x = 0;
        let mut y = 0;
        let result = unsafe {
            sys::SDL_GetWindowMinimumSize(self.as_ptr() as *mut _, &raw mut x, &raw mut y)
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok((x, y))
    }

    pub fn set_min_size(&mut self, w: u32, h: u32) -> Result<(), Error> {
        let w = w.min(i32::MAX as u32) as i32;
        let h = h.min(i32::MAX as u32) as i32;
        let result = unsafe { sys::SDL_SetWindowMinimumSize(self.as_ptr() as *mut _, w, h) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn borders_size(&self) -> Result<(i32, i32, i32, i32), Error> {
        let mut top = 0;
        let mut left = 0;
        let mut bottom = 0;
        let mut right = 0;
        let result = unsafe {
            sys::SDL_GetWindowBordersSize(
                self.as_ptr() as *mut _,
                &raw mut top,
                &raw mut left,
                &raw mut bottom,
                &raw mut right,
            )
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok((top, left, bottom, right))
    }

    pub fn set_bordered(&mut self, bordered: bool) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetWindowBordered(self.as_ptr() as *mut _, bordered) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn set_always_on_top(&mut self, always_on_top: bool) -> Result<(), Error> {
        let result =
            unsafe { sys::SDL_SetWindowAlwaysOnTop(self.as_ptr() as *mut _, always_on_top) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn set_focusable(&mut self, focusable: bool) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetWindowFocusable(self.as_ptr() as *mut _, focusable) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    // SDL mutates the original surface but also creates a copy.
    // So we're free to use it after calling this; hence why it takes a mutable surface as
    // parameter.
    pub fn set_icon(&mut self, icon: &mut SurfaceRef) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetWindowIcon(self.as_ptr() as *mut _, icon.as_mut_ptr()) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn set_mouse_grabbed(&mut self, grabbed: bool) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetWindowMouseGrab(self.as_ptr() as *mut _, grabbed) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn is_mouse_grabbed(&self) -> bool {
        unsafe { sys::SDL_GetWindowMouseGrab(self.as_ptr() as *mut _) }
    }

    pub fn set_keyboard_grabbed(&mut self, grabbed: bool) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetWindowKeyboardGrab(self.as_ptr() as *mut _, grabbed) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn is_keyboard_grabbed(&self) -> bool {
        unsafe { sys::SDL_GetWindowKeyboardGrab(self.as_ptr() as *mut _) }
    }

    pub fn surface_vsync(&self) -> Result<WindowSurfaceVSync, Error> {
        let mut vsync = 0;
        let result =
            unsafe { sys::SDL_GetWindowSurfaceVSync(self.as_ptr() as *mut _, &raw mut vsync) };
        if !result {
            return Err(Error::from_sdl());
        }
        WindowSurfaceVSync::try_from_ll(vsync)
    }

    pub fn set_surface_vsync(&mut self, vsync: WindowSurfaceVSync) -> Result<(), Error> {
        let result =
            unsafe { sys::SDL_SetWindowSurfaceVSync(self.as_ptr() as *mut _, vsync.to_ll()) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn set_window_shape(&mut self, surface: &mut SurfaceRef) -> Result<(), Error> {
        let result =
            unsafe { sys::SDL_SetWindowShape(self.as_ptr() as *mut _, surface.as_mut_ptr()) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn pixel_density(&self) -> Result<f32, Error> {
        let pixel_density = unsafe { sys::SDL_GetWindowPixelDensity(self.as_ptr() as *mut _) };
        if pixel_density == 0.0 {
            return Err(Error::from_sdl());
        }
        Ok(pixel_density)
    }

    pub fn size_in_pixels(&self) -> Result<(i32, i32), Error> {
        let mut w = 0;
        let mut h = 0;
        let result = unsafe {
            sys::SDL_GetWindowSizeInPixels(self.as_ptr() as *mut _, &raw mut w, &raw mut h)
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok((w, h))
    }

    pub fn flash(&mut self, operation: WindowFlashOperation) -> Result<(), Error> {
        let result = unsafe { sys::SDL_FlashWindow(self.as_ptr() as *mut _, operation.0) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn maximize(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_MaximizeWindow(self.as_ptr() as *mut _) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn minimize(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_MinimizeWindow(self.as_ptr() as *mut _) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn raise(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_RaiseWindow(self.as_ptr() as *mut _) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn restore(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_RestoreWindow(self.as_ptr() as *mut _) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn update_surface(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_UpdateWindowSurface(self.as_ptr() as *mut _) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn update_surface_rects(&mut self, rects: &[Rect]) -> Result<(), Error> {
        let rects: Vec<sys::SDL_Rect> = rects.iter().map(|r| r.to_ll()).collect();
        let result = unsafe {
            sys::SDL_UpdateWindowSurfaceRects(
                self.as_ptr() as *mut _,
                rects.as_ptr(),
                rects.len().try_into()?,
            )
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn show_system_menu(&mut self, x: u32, y: u32) -> Result<(), Error> {
        let result = unsafe {
            sys::SDL_ShowWindowSystemMenu(self.as_ptr() as *mut _, x.try_into()?, y.try_into()?)
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn sync(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SyncWindow(self.as_ptr() as *mut _) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    #[inline]
    pub fn as_ptr(&self) -> *const sys::SDL_Window {
        self as *const Self as *const sys::SDL_Window
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut sys::SDL_Window {
        self.as_ptr() as *mut Self as *mut sys::SDL_Window
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe { sys::SDL_DestroyWindow(self.as_ptr() as *mut _) };
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct WindowFlags(sys::SDL_WindowFlags);

impl WindowFlags {
    pub const FULLSCREEN: WindowFlags = WindowFlags(sys::SDL_WINDOW_FULLSCREEN);
    pub const OPEN_GL: WindowFlags = WindowFlags(sys::SDL_WINDOW_OPENGL);
    pub const OCCLUDED: WindowFlags = WindowFlags(sys::SDL_WINDOW_OCCLUDED);
    pub const HIDDEN: WindowFlags = WindowFlags(sys::SDL_WINDOW_HIDDEN);
    pub const BORDERLESS: WindowFlags = WindowFlags(sys::SDL_WINDOW_BORDERLESS);
    pub const RESIZABLE: WindowFlags = WindowFlags(sys::SDL_WINDOW_RESIZABLE);
    pub const MINIMIZED: WindowFlags = WindowFlags(sys::SDL_WINDOW_MINIMIZED);
    pub const MAXIMIZED: WindowFlags = WindowFlags(sys::SDL_WINDOW_MAXIMIZED);
    pub const MOUSE_GRABBED: WindowFlags = WindowFlags(sys::SDL_WINDOW_MOUSE_GRABBED);
    pub const INPUT_FOCUS: WindowFlags = WindowFlags(sys::SDL_WINDOW_INPUT_FOCUS);
    pub const MOUSE_FOCUS: WindowFlags = WindowFlags(sys::SDL_WINDOW_MOUSE_FOCUS);
    pub const EXTERNAL: WindowFlags = WindowFlags(sys::SDL_WINDOW_EXTERNAL);
    pub const MODAL: WindowFlags = WindowFlags(sys::SDL_WINDOW_MODAL);
    pub const HIGH_PIXEL_DENSITY: WindowFlags = WindowFlags(sys::SDL_WINDOW_HIGH_PIXEL_DENSITY);
    pub const MOUSE_CAPTURE: WindowFlags = WindowFlags(sys::SDL_WINDOW_MOUSE_CAPTURE);
    pub const ALWAYS_ON_TOP: WindowFlags = WindowFlags(sys::SDL_WINDOW_ALWAYS_ON_TOP);
    pub const UTILITY: WindowFlags = WindowFlags(sys::SDL_WINDOW_UTILITY);
    pub const TOOLTIP: WindowFlags = WindowFlags(sys::SDL_WINDOW_TOOLTIP);
    pub const POPUP_MENU: WindowFlags = WindowFlags(sys::SDL_WINDOW_POPUP_MENU);
    pub const KEYBOARD_GRABBED: WindowFlags = WindowFlags(sys::SDL_WINDOW_KEYBOARD_GRABBED);
    pub const VULKAN: WindowFlags = WindowFlags(sys::SDL_WINDOW_VULKAN);
    pub const METAL: WindowFlags = WindowFlags(sys::SDL_WINDOW_METAL);
    pub const TRANSPARENT: WindowFlags = WindowFlags(sys::SDL_WINDOW_TRANSPARENT);
    pub const NOT_FOCUSABLE: WindowFlags = WindowFlags(sys::SDL_WINDOW_NOT_FOCUSABLE);
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
pub struct WindowFlashOperation(sys::SDL_FlashOperation);

impl WindowFlashOperation {
    pub const CANCEL: Self = Self(sys::SDL_FlashOperation_SDL_FLASH_CANCEL);
    pub const BRIEFLY: Self = Self(sys::SDL_FlashOperation_SDL_FLASH_BRIEFLY);
    pub const UNTIL_FOCUSED: Self = Self(sys::SDL_FlashOperation_SDL_FLASH_UNTIL_FOCUSED);

    #[inline]
    pub fn to_ll(&self) -> sys::SDL_FlashOperation {
        self.0
    }
}

// We need to copy the SDL_DisplayMode values into this struct because SDL usually hands them out
// as pointers whose lifetimes are a bit messy. Adding or removing a display might move the
// underlying memory of the pointer to a different location.
#[repr(C)]
#[derive(Clone, PartialEq)]
pub struct DisplayMode {
    pub display_id: u32,
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
    unsafe fn from_ptr(ptr: *const sys::SDL_DisplayMode) -> Self {
        Self {
            display_id: (*ptr).displayID,
            format: PixelFormat::from_ll_unchecked((*ptr).format),
            w: (*ptr).w,
            h: (*ptr).h,
            pixel_density: (*ptr).pixel_density,
            refresh_rate: (*ptr).refresh_rate,
            refresh_rate_numerator: (*ptr).refresh_rate_numerator,
            refresh_rate_denominator: (*ptr).refresh_rate_denominator,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DisplayOrientation {
    Unknown,
    Landscape,
    LandscapeFlipped,
    Portrait,
    PortraitFlipped,
}

impl DisplayOrientation {
    pub fn try_from_ll(value: sys::SDL_DisplayOrientation) -> Result<Self, Error> {
        Ok(match value {
            sys::SDL_DisplayOrientation_SDL_ORIENTATION_UNKNOWN => Self::Unknown,
            sys::SDL_DisplayOrientation_SDL_ORIENTATION_LANDSCAPE => Self::Landscape,
            sys::SDL_DisplayOrientation_SDL_ORIENTATION_LANDSCAPE_FLIPPED => Self::LandscapeFlipped,
            sys::SDL_DisplayOrientation_SDL_ORIENTATION_PORTRAIT => Self::Portrait,
            sys::SDL_DisplayOrientation_SDL_ORIENTATION_PORTRAIT_FLIPPED => Self::PortraitFlipped,
            _ => return Err(Error::UnknownDisplayOrientation(value)),
        })
    }

    pub fn to_ll(&self) -> sys::SDL_DisplayOrientation {
        match self {
            DisplayOrientation::Unknown => sys::SDL_DisplayOrientation_SDL_ORIENTATION_UNKNOWN,
            DisplayOrientation::Landscape => sys::SDL_DisplayOrientation_SDL_ORIENTATION_LANDSCAPE,
            DisplayOrientation::LandscapeFlipped => {
                sys::SDL_DisplayOrientation_SDL_ORIENTATION_LANDSCAPE_FLIPPED
            }
            DisplayOrientation::Portrait => sys::SDL_DisplayOrientation_SDL_ORIENTATION_PORTRAIT,
            DisplayOrientation::PortraitFlipped => {
                sys::SDL_DisplayOrientation_SDL_ORIENTATION_PORTRAIT_FLIPPED
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WindowSurfaceVSync {
    EveryVerticalRefresh,
    EverySecondVerticalRefresh,
    Adaptive,
    Disabled,
}

impl WindowSurfaceVSync {
    pub fn try_from_ll(value: i32) -> Result<Self, Error> {
        if value == 1 {
            Ok(Self::EveryVerticalRefresh)
        } else if value == 2 {
            Ok(Self::EverySecondVerticalRefresh)
        } else if value == sys::SDL_WINDOW_SURFACE_VSYNC_ADAPTIVE {
            Ok(Self::Adaptive)
        } else if value == sys::SDL_WINDOW_SURFACE_VSYNC_DISABLED as i32 {
            Ok(Self::Disabled)
        } else {
            Err(Error::UnknownSurfaceVsyncType(value))
        }
    }

    pub fn to_ll(&self) -> i32 {
        match self {
            WindowSurfaceVSync::EveryVerticalRefresh => 1,
            WindowSurfaceVSync::EverySecondVerticalRefresh => 2,
            WindowSurfaceVSync::Adaptive => sys::SDL_WINDOW_SURFACE_VSYNC_ADAPTIVE,
            WindowSurfaceVSync::Disabled => sys::SDL_WINDOW_SURFACE_VSYNC_DISABLED as i32,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SysthemTheme {
    Light,
    Dark,
    Unknown,
}

impl SysthemTheme {
    pub fn try_from_ll(theme: sys::SDL_SystemTheme) -> Result<Self, Error> {
        Ok(match theme {
            sys::SDL_SystemTheme_SDL_SYSTEM_THEME_UNKNOWN => SysthemTheme::Unknown,
            sys::SDL_SystemTheme_SDL_SYSTEM_THEME_LIGHT => SysthemTheme::Light,
            sys::SDL_SystemTheme_SDL_SYSTEM_THEME_DARK => SysthemTheme::Dark,
            _ => return Err(Error::InvalidSystemTheme),
        })
    }

    pub fn to_ll(&self) -> sys::SDL_SystemTheme {
        match self {
            SysthemTheme::Light => sys::SDL_SystemTheme_SDL_SYSTEM_THEME_LIGHT,
            SysthemTheme::Dark => sys::SDL_SystemTheme_SDL_SYSTEM_THEME_LIGHT,
            SysthemTheme::Unknown => sys::SDL_SystemTheme_SDL_SYSTEM_THEME_LIGHT,
        }
    }
}
