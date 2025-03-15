use crate::blendmode::BlendMode;
use crate::pixels::{Color, ColorF32, PixelFormat};
use crate::rect::{Rect, RectF32};
use crate::surface::{Surface, SurfaceRef};
use crate::video::{Window, WindowRef};
use crate::{sys, Error};
use alloc::ffi::CString;
use alloc::rc::{Rc, Weak};
use alloc::string::String;
use core::ffi::CStr;
use core::mem::{ManuallyDrop, MaybeUninit};

/// A structure representing rendering state.
pub struct Renderer<T: Backbuffer> {
    /// This ptr is ref-counted so we can hand out Weak references.
    ptr: Rc<*mut sys::SDL_Renderer>,
    inner: T::Inner,
}

impl Renderer<Window> {
    /// Creates a `Renderer` from an existing `Window` using the specified `driver`.
    ///
    /// The `driver` name can be obtained by calling [`crate::VideoSubsystem::render_driver`] using the driver's index.
    ///
    /// If `driver` is `None`, SDL will choose the best available option.
    ///
    /// The `Window` can later be borrowed by calling `Renderer::as_window_ref` or `Renderer::as_window_mut`.
    pub fn from_window(mut window: Window, driver: Option<&str>) -> Result<Self, Error> {
        unsafe {
            let driver = match driver {
                Some(driver) => Some(CString::new(driver)?),
                None => None,
            };
            let driver = driver.map(|s| s.as_ptr()).unwrap_or(core::ptr::null());
            let ptr = sys::SDL_CreateRenderer(window.as_mut_ptr(), driver);
            if ptr.is_null() {
                return Err(Error);
            }
            // The window will be dropped when RendererContext::drop_inner gets called.
            let _ = ManuallyDrop::new(window);
            Ok(Self {
                inner: (),
                ptr: Rc::new(ptr),
            })
        }
    }

    /// Returns a reference to the renderer's window, if it has one.
    pub fn as_window_ref(&self) -> &WindowRef {
        unsafe {
            let ptr = sys::SDL_GetRenderWindow(self.raw());
            WindowRef::from_ptr(ptr)
        }
    }

    /// Returns a mutable reference to the renderer's window, if it has one.
    pub fn as_window_mut(&mut self) -> &mut WindowRef {
        unsafe {
            let ptr = sys::SDL_GetRenderWindow(self.raw());
            WindowRef::from_mut_ptr(ptr)
        }
    }
}

impl<'a> Renderer<Surface<'a>> {
    /// Creates a software `Renderer` from an existing `Surface`.
    ///
    /// The surface can later be borrowed by calling `Renderer::as_surface_ref` or `Renderer::as_surface_mut`.
    pub fn from_owned_surface(surface: Surface<'a>) -> Result<Self, Error> {
        unsafe {
            let ptr = sys::SDL_CreateSoftwareRenderer(surface.raw());
            if ptr.is_null() {
                return Err(Error);
            }
            Ok(Self {
                inner: surface,
                ptr: Rc::new(ptr),
            })
        }
    }

    /// Returns a reference to the renderer's underlying surface, if it has one.
    pub fn as_surface_ref(&self) -> &SurfaceRef {
        &self.inner
    }

    /// Returns a mutable reference to the renderer's underlying surface, if it has one.
    pub fn as_surface_mut(&mut self) -> &mut SurfaceRef {
        &mut self.inner
    }
}

impl<'a> Renderer<&'a mut SurfaceRef> {
    /// Creates a software `Renderer` from an existing `Surface`.
    ///
    /// The surface can later be borrowed by calling `Renderer::as_surface_ref` or `Renderer::as_surface_mut`.
    pub fn from_surface(surface: &'a mut SurfaceRef) -> Result<Self, Error> {
        unsafe {
            let ptr = sys::SDL_CreateSoftwareRenderer(surface.raw());
            if ptr.is_null() {
                return Err(Error);
            }
            Ok(Self {
                inner: surface,
                ptr: Rc::new(ptr),
            })
        }
    }

    /// Returns a reference to the renderer's underlying surface, if it has one.
    pub fn as_surface_ref(&self) -> &SurfaceRef {
        self.inner
    }

    /// Returns a mutable reference to the renderer's underlying surface, if it has one.
    pub fn as_surface_mut(&mut self) -> &mut SurfaceRef {
        self.inner
    }
}

impl<T: Backbuffer> Renderer<T> {
    /// Returns the name of the renderer.
    pub fn name(&self) -> Result<String, Error> {
        let name = unsafe {
            let ptr = sys::SDL_GetRendererName(self.raw());
            if ptr.is_null() {
                return Err(Error);
            }
            CStr::from_ptr(ptr)
        };
        Ok(name.to_string_lossy().into_owned())
    }

    /// Creates a texture for a rendering context.
    ///
    /// The contents of a texture when first created are not defined.
    ///
    /// This method is equivalent to [`Texture::new`].
    pub fn create_texture(
        &mut self,
        format: PixelFormat,
        access: TextureAccess,
        width: u32,
        height: u32,
    ) -> Result<Texture, Error> {
        Texture::new(self, format, access, width, height)
    }

    /// Create a texture from an existing surface.
    ///
    /// The surface is not modified by this function.
    ///
    /// The [`TextureAccess`] hint for the created texture is [`TextureAccess::Static`].
    ///
    /// The pixel format of the created texture may be different from the pixel format of the surface.
    ///
    /// This method is equivalent to [`Texture::from_surface`].
    pub fn create_texture_from_surface(&mut self, surface: &SurfaceRef) -> Result<Texture, Error> {
        Texture::from_surface(self, surface)
    }

    /// Returns the color used for drawing operations.
    pub fn draw_color(&self) -> Result<Color, Error> {
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        let mut a = 0;
        let result = unsafe {
            sys::SDL_GetRenderDrawColor(
                self.raw() as *mut _,
                &raw mut r,
                &raw mut g,
                &raw mut b,
                &raw mut a,
            )
        };
        if !result {
            return Err(Error);
        }
        Ok(Color::new(r, g, b, a))
    }

    /// Set the color used for drawing operations.
    ///
    /// Set the color for drawing or filling rectangles, lines, and points, and for [`Renderer::clear`].
    pub fn set_draw_color(&mut self, color: Color) -> Result<(), Error> {
        let result = unsafe {
            sys::SDL_SetRenderDrawColor(self.raw(), color.r(), color.g(), color.b(), color.a())
        };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Returns the color used for drawing operations.
    pub fn draw_color_float(&self) -> Result<ColorF32, Error> {
        let mut r = 0.0;
        let mut g = 0.0;
        let mut b = 0.0;
        let mut a = 0.0;
        let result = unsafe {
            sys::SDL_GetRenderDrawColorFloat(
                self.raw() as *mut _,
                &raw mut r,
                &raw mut g,
                &raw mut b,
                &raw mut a,
            )
        };
        if !result {
            return Err(Error);
        }
        Ok(ColorF32::new(r, g, b, a))
    }

    /// Set the color used for drawing operations.
    ///
    /// Set the color for drawing or filling rectangles, lines, and points, and for [`Renderer::clear`].
    pub fn set_draw_color_float(&mut self, color: ColorF32) -> Result<(), Error> {
        let result = unsafe {
            sys::SDL_SetRenderDrawColorFloat(self.raw(), color.r(), color.g(), color.b(), color.a())
        };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Get the output size in pixels of a rendering context.
    ///
    /// This returns the true output size in pixels, ignoring any render targets or logical size and presentation.
    /// Get the current output size in pixels of a rendering context.
    ///
    /// If a rendering target is active, this will return the size of the rendering target in pixels, otherwise if
    /// a logical size is set, it will return the logical size, otherwise it will return the value of
    /// [`Renderer::output_size`].
    pub fn current_output_size(&self) -> Result<(u32, u32), Error> {
        let mut w = 0;
        let mut h = 0;
        let res = unsafe {
            sys::SDL_GetCurrentRenderOutputSize(self.raw() as *mut _, &raw mut w, &raw mut h)
        };
        if !res {
            return Err(Error);
        }
        Ok((u32::try_from(w)?, u32::try_from(h)?))
    }

    /// Get the output size in pixels of a rendering context.
    ///
    /// This returns the true output size in pixels, ignoring any render targets or logical size and presentation.
    pub fn output_size(&self) -> Result<(u32, u32), Error> {
        let mut w = 0;
        let mut h = 0;
        let res = unsafe { sys::SDL_GetRenderOutputSize(self.raw(), &raw mut w, &raw mut h) };
        if !res {
            return Err(Error);
        }
        Ok((u32::try_from(w)?, u32::try_from(h)?))
    }

    /// Get the clip rectangle for the current target.
    pub fn clip_rect(&self) -> Result<Rect, Error> {
        let mut rect: MaybeUninit<sys::SDL_Rect> = MaybeUninit::uninit();
        let res = unsafe { sys::SDL_GetRenderClipRect(self.raw(), rect.as_mut_ptr()) };
        if !res {
            return Err(Error);
        }
        let rect = unsafe { rect.assume_init() };
        Ok(Rect::from_ll(rect))
    }

    /// Get the color scale used for render operations.
    pub fn color_scale(&self) -> Result<f32, Error> {
        let mut scale = 0.0;
        let res = unsafe { sys::SDL_GetRenderColorScale(self.raw(), &raw mut scale) };
        if !res {
            return Err(Error);
        }
        Ok(scale)
    }

    /// Get the blend mode used for drawing operations.
    pub fn draw_blend_mode(&self) -> Result<Option<BlendMode>, Error> {
        let mut blend_mode: MaybeUninit<sys::SDL_BlendMode> = MaybeUninit::uninit();
        let res = unsafe { sys::SDL_GetRenderDrawBlendMode(self.raw(), blend_mode.as_mut_ptr()) };
        if !res {
            return Err(Error);
        }
        BlendMode::try_from_ll(unsafe { blend_mode.assume_init() })
    }

    /// Set the blend mode used for drawing operations.
    /// If the blend mode is not supported, the closest supported mode is chosen.
    pub fn set_draw_blend_mode(&mut self, blend_mode: BlendMode) -> Result<(), Error> {
        let res = unsafe { sys::SDL_SetRenderDrawBlendMode(self.raw(), blend_mode.to_ll()) };
        if !res {
            return Err(Error);
        }
        Ok(())
    }

    /// Returns the VSync of the given renderer.
    pub fn vsync(&self) -> Result<RendererVSync, Error> {
        let mut vsync = 0;
        let result = unsafe { sys::SDL_GetRenderVSync(self.raw(), &raw mut vsync) };
        if !result {
            return Err(Error);
        }
        Ok(unsafe { RendererVSync::from_ll_unchecked(vsync) })
    }

    /// Toggle VSync of the given renderer.
    ///
    /// When a renderer is created, vsync defaults to `RendererVSync::Disabled`.
    pub fn set_vsync(&mut self, value: RendererVSync) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetRenderVSync(self.raw(), value.to_raw()) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Fill a rectangle on the current rendering target with the drawing color at subpixel precision.
    pub fn fill_rect(&mut self, rect: RectF32) -> Result<(), Error> {
        let rect = rect.to_ll();
        let result = unsafe { sys::SDL_RenderFillRect(self.raw(), &raw const rect) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Fill some number of rectangles on the current rendering target with the drawing color at subpixel precision.
    pub fn fill_rects(&mut self, rects: &[RectF32]) -> Result<(), Error> {
        let count = i32::try_from(rects.len())
            .map_err(|_| Error::register(c"Invalid rects length (TryFromIntError)."))?;
        let rects = rects.as_ptr() as *const sys::SDL_FRect;
        let result = unsafe { sys::SDL_RenderFillRects(self.raw(), rects, count) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    pub fn render_debug_text(&mut self, x: f32, y: f32, text: &str) -> Result<(), Error> {
        let string = CString::new(text).map_err(|_| {
            Error::register(c"Invalid debug text. Interior null byte found (NulError)")
        })?;
        let result = unsafe { sys::SDL_RenderDebugText(self.raw(), x, y, string.as_ptr()) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Copy a portion of the texture to the current rendering target at subpixel precision.
    ///
    /// * `texture` - the source texture
    /// * `src_rect` - the source rectangle or `None` for the entire texture.
    /// * `dest_rect` - the destination rectangle or `None` for the entire rendering target.
    pub fn render_texture(
        &mut self,
        texture: &Texture,
        src_rect: Option<RectF32>,
        dest_rect: Option<RectF32>,
    ) -> Result<(), Error> {
        self.validate_texture(texture)?;

        let src_rect = src_rect.map(RectF32::to_ll);
        let src_rect_ptr = src_rect
            .as_ref()
            .map_or(core::ptr::null(), core::ptr::from_ref);

        let dest_rect = dest_rect.map(RectF32::to_ll);
        let dest_rect_ptr = dest_rect
            .as_ref()
            .map_or(core::ptr::null(), core::ptr::from_ref);

        let result =
            unsafe { sys::SDL_RenderTexture(self.raw(), texture.ptr, src_rect_ptr, dest_rect_ptr) };

        if !result {
            return Err(Error);
        }

        Ok(())
    }

    /// Replaces the current rendering target with the given texture. Returns the previously used texture if there was one.
    ///
    /// The default render target is the window (or surface) for which the renderer was created.
    ///
    /// To stop rendering to a texture and render to the window (or surface), use `None` as the `texture` parameter.
    pub fn replace_render_target(
        &mut self,
        texture: Option<Texture>,
    ) -> Result<Option<Texture>, Error> {
        let previous_target = unsafe {
            let ptr = sys::SDL_GetRenderTarget(self.raw());
            if !ptr.is_null() {
                Some(Texture::from_mut_ptr(self, ptr))
            } else {
                None
            }
        };

        match texture {
            Some(texture) => {
                self.validate_texture(&texture)?;
                let result = unsafe { sys::SDL_SetRenderTarget(self.raw(), texture.ptr) };
                if !result {
                    return Err(Error);
                }
            }
            _ => {
                let result = unsafe { sys::SDL_SetRenderTarget(self.raw(), core::ptr::null_mut()) };
                if !result {
                    return Err(Error);
                }
            }
        }

        Ok(previous_target)
    }

    /// Update the screen with any rendering performed since the previous call.
    ///
    /// SDL's rendering functions operate on a backbuffer; that is, calling a rendering function such as [`Renderer::render_line`]
    /// does not directly put a line on the screen, but rather updates the backbuffer. As such, you compose your entire scene and
    /// *present* the composed backbuffer to the screen as a complete picture.
    ///
    /// Therefore, when using SDL's rendering API, one does all drawing intended for the frame, and then calls this function once
    /// per frame to present the final drawing to the user.
    ///
    /// The backbuffer should be considered invalidated after each present; do not assume that previous contents will exist between
    /// frames. You are strongly encouraged to call [`Renderer::clear`] to initialize the backbuffer before starting each new frame's
    /// drawing, even if you plan to overwrite every pixel.
    ///
    /// Please note, that in case of rendering to a texture - there is **no need** to call [`Renderer::present`] after drawing needed
    /// objects to a texture, and should not be done; you are only required to change back the rendering target to default via
    /// [`Renderer::set_render_target`] afterwards, as textures by themselves do not have a concept of backbuffers.
    /// Calling [`Renderer::present`] while rendering to a texture will still update the screen with any current drawing that
    /// has been done _to the window itself_.
    pub fn present(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_RenderPresent(self.raw()) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Clear the current rendering target with the drawing color.
    ///
    /// This function clears the entire rendering target, ignoring the viewport and the clip rectangle. Note, that clearing will also
    /// set/fill all pixels of the rendering target to current renderer draw color, so make sure to invoke [`Renderer::set_draw_color`]
    /// when needed.
    pub fn clear(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_RenderClear(self.raw()) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Returns a mutable pointer to the underlying raw `SDL_Renderer` used by this `Renderer`.
    #[inline]
    pub fn raw(&self) -> *mut sys::SDL_Renderer {
        *self.ptr
    }

    fn validate_texture(&self, texture: &Texture) -> Result<(), Error> {
        // We could check whether or not this texture belongs to this renderer, but SDL does it for us.
        // So we only check whether or not texture's renderer is still alive.
        if texture.renderer.strong_count() == 0 {
            return Err(Error::register(c"Renderer already destroyed."));
        }
        Ok(())
    }
}

impl<T: Backbuffer> Drop for Renderer<T> {
    fn drop(&mut self) {
        unsafe {
            sys::SDL_DestroyRenderer(*self.ptr);
            T::drop_inner(self.raw());
        }
    }
}

/// Toggle VSync of the given renderer.
///
/// When a renderer is created, vsync defaults to SDL_RENDERER_VSYNC_DISABLED.
///
/// The `vsync` parameter can be
/// 1 to synchronize present with every vertical refresh,
/// 2 to synchronize present with every second vertical refresh,
/// SDL_RENDERER_VSYNC_ADAPTIVE for late swap tearing (adaptive vsync),
/// SDL_RENDERER_VSYNC_DISABLED to disable.
///
///
/// Not every value is supported by\n every driver, so you should check the return value to see whether the\n requested setting is supported.\n\n \\param renderer the renderer to toggle.\n \\param vsync the vertical refresh sync interval.\n \\returns true on success or false on failure; call SDL_GetError() for more\n          information.\n\n \\threadsafety This function should only be called on the main thread.\n\n \\since This function is available since SDL 3.2.0.\n\n \\sa SDL_GetRenderVSync"]
#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RendererVSync {
    EveryVerticalRefresh = 1,
    EverySecondVerticalRefresh = 2,
    Adaptive = sys::SDL_RENDERER_VSYNC_ADAPTIVE,
    Disabled = sys::SDL_RENDERER_VSYNC_DISABLED as i32,
}

impl RendererVSync {
    /// SAFETY: `value` must be a valid variant of the enum.
    unsafe fn from_ll_unchecked(value: i32) -> Self {
        unsafe { core::mem::transmute(value) }
    }

    pub fn to_raw(&self) -> i32 {
        *self as i32
    }
}

/// Driver-specific representation of pixel data.
///
/// This struct wraps [`sys::SDL_Texture`].
pub struct Texture {
    /// This renderer owns this surface.
    /// If this renderer is not alive (we can tell by calling `Weak::strong_count`),
    /// then this texture is stale.
    /// This must *never* be upgraded to an Rc.
    renderer: Weak<*mut sys::SDL_Renderer>,
    ptr: *mut sys::SDL_Texture,
}

impl Texture {
    /// Creates a texture for a rendering context.
    ///
    /// The contents of a texture when first created are not defined.
    pub fn new<T: Backbuffer>(
        renderer: &mut Renderer<T>,
        format: PixelFormat,
        access: TextureAccess,
        width: u32,
        height: u32,
    ) -> Result<Self, Error> {
        let format = format.to_ll();
        let access = access.to_ll();
        let ptr = unsafe {
            sys::SDL_CreateTexture(
                renderer.raw(),
                format,
                access,
                width.try_into()?,
                height.try_into()?,
            )
        };
        if ptr.is_null() {
            return Err(Error);
        }
        Ok(Self {
            renderer: Rc::downgrade(&renderer.ptr),
            ptr,
        })
    }

    /// Create a texture from an existing surface.
    ///
    /// The surface is not modified by this function.
    ///
    /// The [`TextureAccess`] hint for the created texture is [`TextureAccess::Static`].
    ///
    /// The pixel format of the created texture may be different from the pixel format of the surface.
    pub fn from_surface<T: Backbuffer>(
        renderer: &mut Renderer<T>,
        surface: &SurfaceRef,
    ) -> Result<Self, Error> {
        let ptr =
            unsafe { sys::SDL_CreateTextureFromSurface(renderer.raw(), surface.raw() as *mut _) };
        if ptr.is_null() {
            return Err(Error);
        }
        Ok(Texture {
            renderer: Rc::downgrade(&renderer.ptr),
            ptr,
        })
    }

    unsafe fn from_mut_ptr<T: Backbuffer>(
        renderer: &mut Renderer<T>,
        ptr: *mut sys::SDL_Texture,
    ) -> Self {
        Self {
            renderer: Rc::downgrade(&renderer.ptr),
            ptr,
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        // We only drop the texture if the parent renderer is alive.
        if self.renderer.strong_count() > 0 {
            unsafe { sys::SDL_DestroyTexture(self.ptr) };
        }
    }
}

/// The access pattern allowed for a texture.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureAccess {
    Static = sys::SDL_TextureAccess_SDL_TEXTUREACCESS_STATIC,
    Streaming = sys::SDL_TextureAccess_SDL_TEXTUREACCESS_STREAMING,
    Target = sys::SDL_TextureAccess_SDL_TEXTUREACCESS_TARGET,
}

impl TextureAccess {
    pub fn to_ll(self) -> sys::SDL_TextureAccess {
        self as sys::SDL_TextureAccess
    }
}

#[doc(hidden)]
pub trait Backbuffer {
    type Inner;

    unsafe fn drop_inner(_renderer: *mut sys::SDL_Renderer) {}
}

impl Backbuffer for Window {
    /// With a Window as backbuffer we can just call SDL_GetRenderWindow and get a pointer
    /// to the underlying window, so we don't have to actually store the surface inside
    /// the Renderer struct.
    type Inner = ();

    unsafe fn drop_inner(renderer: *mut rsdl3_sys::SDL_Renderer) {
        let window = sys::SDL_GetRenderWindow(renderer);
        if window.is_null() {
            return;
        }
        sys::SDL_DestroyWindow(window);
    }
}

impl<'a> Backbuffer for Surface<'a> {
    type Inner = Self;
}

impl<'a> Backbuffer for &'a mut SurfaceRef {
    type Inner = Self;
}
