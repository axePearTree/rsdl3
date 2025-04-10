use crate::blendmode::BlendMode;
use crate::events::Event;
use crate::pixels::{Color, ColorF32, PixelFormat};
use crate::rect::{Point, PointF32, Rect, RectF32};
use crate::surface::{FlipMode, ScaleMode, Surface, SurfaceRef};
use crate::video::{Window, WindowRef};
use crate::{sys, Error, VideoSubsystem};
use alloc::ffi::CString;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;
use core::ffi::CStr;
use core::hint::unreachable_unchecked;
use core::mem::MaybeUninit;
use core::ptr::NonNull;

/// A structure representing rendering state.
///
/// Dropping the renderer does not necessarily mean the raw SDL renderer
/// will be destroyed; same for its' backbuffer. Internally, the raw
/// SDL renderer is shared by any [`Texture`]s created by this struct.
pub struct Renderer<T = Window> {
    /// An internal renderer that takes care of destroying the raw renderer
    /// once it goes out of scope.
    /// Textures created by this renderer will also hold references to the
    /// internal renderer.
    internal: Rc<RendererInternal<T>>,
    /// The owner of the renderer can be a window, an owned surface or a
    /// borrowed surface. We need mutable access to the owner while this
    /// struct is alive. If this struct gets dropped and it's not the sole
    /// owner of `internal` ([`Rc::strong_count`]), then we move the owner
    /// to the internal renderer so it can be dropped later.
    ///
    /// SAFETY: `owner` must be `Some` until this value gets dropped.
    owner: Option<T>,
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
            let ptr = NonNull::new(ptr).ok_or(Error::new())?;
            Ok(Self {
                internal: Rc::new(RendererInternal {
                    ptr,
                    owner: RefCell::new(None),
                }),
                owner: Some(window),
            })
        }
    }

    /// Read pixels from the current rendering target.
    ///
    /// **WARNING**: This is a very slow operation, and should not be used frequently.
    /// If you're using this on the main rendering target, it should be called after
    /// rendering and before [`Renderer::present`].
    pub fn read_pixels(&self, rect: Option<Rect>) -> Result<Surface<'static>, Error> {
        let rect = rect.as_ref().map(Rect::as_raw).unwrap_or(core::ptr::null());
        unsafe {
            let surface = sys::SDL_RenderReadPixels(self.raw(), rect);
            if surface.is_null() {
                return Err(Error::new());
            }
            let video = &self.owner.as_ref().unwrap_unchecked().video;
            Ok(Surface::from_mut_ptr(video, surface))
        }
    }

    /// Returns a reference to the renderer's window, if it has one.
    #[inline]
    pub fn as_window_ref(&self) -> &WindowRef {
        // owner only becomes `None` once this struct gets dropped.
        unsafe { self.owner.as_ref().unwrap_unchecked() }
    }

    /// Returns a mutable reference to the renderer's window, if it has one.
    #[inline]
    pub fn as_window_mut(&mut self) -> &mut WindowRef {
        // owner only becomes `None` once this struct gets dropped.
        unsafe { self.owner.as_mut().unwrap_unchecked() }
    }
}

impl<'a> Renderer<Surface<'a>> {
    /// Creates a software `Renderer` from an existing `Surface`.
    ///
    /// The surface can later be borrowed by calling `Renderer::as_surface_ref` or `Renderer::as_surface_mut`.
    pub fn from_owned_surface(surface: Surface<'a>) -> Result<Self, Error> {
        unsafe {
            let ptr = sys::SDL_CreateSoftwareRenderer(surface.raw());
            let ptr = NonNull::new(ptr).ok_or(Error::new())?;
            Ok(Self {
                internal: Rc::new(RendererInternal {
                    ptr,
                    owner: RefCell::new(None),
                }),
                owner: Some(surface),
            })
        }
    }

    /// Read pixels from the current rendering target.
    ///
    /// **WARNING**: This is a very slow operation, and should not be used frequently.
    /// If you're using this on the main rendering target, it should be called after
    /// rendering and before [`Renderer::present`].
    pub fn read_pixels(&self, rect: Option<Rect>) -> Result<Surface<'static>, Error> {
        let rect = rect.as_ref().map(Rect::as_raw).unwrap_or(core::ptr::null());
        unsafe {
            let surface = sys::SDL_RenderReadPixels(self.raw(), rect);
            if surface.is_null() {
                return Err(Error::new());
            }
            let video = &self.owner.as_ref().unwrap_unchecked().video;
            Ok(Surface::from_mut_ptr(video, surface))
        }
    }

    /// Returns a reference to the renderer's underlying surface, if it has one.
    #[inline]
    pub fn as_surface_ref(&self) -> &SurfaceRef {
        // owner only becomes `None` once this struct gets dropped.
        unsafe { self.owner.as_ref().unwrap_unchecked() }
    }

    /// Returns a mutable reference to the renderer's underlying surface, if it has one.
    #[inline]
    pub fn as_surface_mut(&mut self) -> &mut SurfaceRef {
        // owner only becomes `None` once this struct gets dropped.
        unsafe { self.owner.as_mut().unwrap_unchecked() }
    }
}

impl<'a> Renderer<&'a mut SurfaceRef> {
    /// Creates a software `Renderer` from an existing `Surface`.
    ///
    /// The surface can later be borrowed by calling `Renderer::as_surface_ref` or `Renderer::as_surface_mut`.
    pub fn from_surface(surface: &'a mut SurfaceRef) -> Result<Self, Error> {
        let ptr = unsafe { sys::SDL_CreateSoftwareRenderer(surface.raw()) };
        let ptr = NonNull::new(ptr).ok_or(Error::new())?;
        Ok(Self {
            internal: Rc::new(RendererInternal {
                ptr,
                owner: RefCell::new(None),
            }),
            owner: Some(surface),
        })
    }

    /// Read pixels from the current rendering target.
    ///
    /// **WARNING**: This is a very slow operation, and should not be used frequently.
    /// If you're using this on the main rendering target, it should be called after
    /// rendering and before [`Renderer::present`].
    pub fn read_pixels(
        &self,
        video: &VideoSubsystem,
        rect: Option<Rect>,
    ) -> Result<Surface<'static>, Error> {
        let rect = rect.as_ref().map(Rect::as_raw).unwrap_or(core::ptr::null());
        unsafe {
            let surface = sys::SDL_RenderReadPixels(self.raw(), rect);
            if surface.is_null() {
                return Err(Error::new());
            }
            Ok(Surface::from_mut_ptr(video, surface))
        }
    }

    /// Returns a reference to the renderer's underlying surface, if it has one.
    #[inline]
    pub fn as_surface_ref(&self) -> &SurfaceRef {
        match &self.owner {
            Some(surface) => surface,
            None => unsafe {
                // The owner is always alive while the Renderer is alive.
                unreachable_unchecked()
            },
        }
    }

    /// Returns a mutable reference to the renderer's underlying surface, if it has one.
    #[inline]
    pub fn as_surface_mut(&mut self) -> &mut SurfaceRef {
        match &mut self.owner {
            Some(surface) => surface,
            None => unsafe {
                // The owner is always alive while the Renderer is alive.
                unreachable_unchecked()
            },
        }
    }
}

impl<T> Renderer<T> {
    /// Returns the name of the renderer.
    pub fn name(&self) -> Result<String, Error> {
        let name = unsafe {
            let ptr = sys::SDL_GetRendererName(self.raw());
            if ptr.is_null() {
                return Err(Error::new());
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
    ) -> Result<Texture<T>, Error> {
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
    pub fn create_texture_from_surface(
        &mut self,
        surface: &SurfaceRef,
    ) -> Result<Texture<T>, Error> {
        Texture::from_surface(self, surface)
    }

    /// Returns a pointer to the `CAMetalLayer` associated with the given Metal renderer.
    ///
    /// This function returns `*mut core::ffi::c_void`, so SDL doesn't have to include Metal's headers, but it can be
    /// safely cast to a `*mut CAMetalLayer`.
    ///
    /// Returns `core::ptr::null()` if the renderer isn't a metal renderer.
    pub fn metal_layer(&self) -> *mut core::ffi::c_void {
        unsafe { sys::SDL_GetRenderMetalLayer(self.raw()) }
    }

    /// Returns the Metal command encoder for the current frame.
    ///
    /// This function returns `*mut core::ffi::c_void`, so SDL doesn't have to include Metal's headers, but it can be
    /// safely cast to an `id<MTLRenderCommandEncoder>`.
    ///
    /// This will return `core::ptr::null()` if Metal refuses to give SDL a drawable to render to, which might happen
    /// if the window is hidden/minimized/offscreen. This doesn't apply to command encoders for render targets, just
    /// the window's backbuffer. Check your return values!
    pub fn metal_encoder(&self) -> *mut core::ffi::c_void {
        unsafe { sys::SDL_GetRenderMetalCommandEncoder(self.raw()) }
    }

    /// Returns the safe area for rendering within the current viewport.
    ///
    /// Some devices have portions of the screen which are partially obscured or not interactive,
    /// possibly due to on-screen controls, curved edges, camera notches, TV overscan, etc. This
    /// function provides the area of the current viewport which is safe to have interactible content.
    /// You should continue rendering into the rest of the render target, but it should not contain
    /// visually important or interactible content.
    pub fn safe_area(&self) -> Result<Rect, Error> {
        let mut rect: MaybeUninit<sys::SDL_Rect> = MaybeUninit::uninit();
        let result = unsafe { sys::SDL_GetRenderSafeArea(self.raw(), rect.as_mut_ptr()) };
        if !result {
            return Err(Error::new());
        }
        let rect = Rect::from_ll(unsafe { rect.assume_init() });
        Ok(rect)
    }

    /// Get the final presentation rectangle for rendering.
    ///
    /// This function returns the calculated rectangle used for logical presentation, based on the
    /// presentation mode and output size. If logical presentation is disabled, it will fill the
    /// rectangle with the output size, in pixels.
    pub fn logical_presentation_rect(&self) -> Result<RectF32, Error> {
        let mut rect: MaybeUninit<sys::SDL_FRect> = MaybeUninit::uninit();
        let result =
            unsafe { sys::SDL_GetRenderLogicalPresentationRect(self.raw(), rect.as_mut_ptr()) };
        if !result {
            return Err(Error::new());
        }
        let rect = RectF32::from_ll(unsafe { rect.assume_init() });
        Ok(rect)
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
            return Err(Error::new());
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
            return Err(Error::new());
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
            return Err(Error::new());
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
            return Err(Error::new());
        }
        Ok(())
    }

    /// Returns the output size in pixels of a rendering context.
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
            return Err(Error::new());
        }
        Ok((u32::try_from(w)?, u32::try_from(h)?))
    }

    /// Returns the output size in pixels of a rendering context.
    ///
    /// This returns the true output size in pixels, ignoring any render targets or logical size and presentation.
    pub fn output_size(&self) -> Result<(u32, u32), Error> {
        let mut w = 0;
        let mut h = 0;
        let res = unsafe { sys::SDL_GetRenderOutputSize(self.raw(), &raw mut w, &raw mut h) };
        if !res {
            return Err(Error::new());
        }
        Ok((u32::try_from(w)?, u32::try_from(h)?))
    }

    /// Returns the clip rectangle for the current target.
    pub fn clip_rect(&self) -> Result<Rect, Error> {
        let mut rect: MaybeUninit<sys::SDL_Rect> = MaybeUninit::uninit();
        let res = unsafe { sys::SDL_GetRenderClipRect(self.raw(), rect.as_mut_ptr()) };
        if !res {
            return Err(Error::new());
        }
        let rect = unsafe { rect.assume_init() };
        Ok(Rect::from_ll(rect))
    }

    /// Set the clip rectangle for rendering on the specified target.
    pub fn set_clip_rect(&mut self, rect: Rect) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetRenderClipRect(self.raw(), &raw const rect.0) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Returns whether clipping is enabled on the renderer.
    pub fn is_clip_enabled(&self) -> bool {
        unsafe { sys::SDL_RenderClipEnabled(self.raw()) }
    }

    /// Returns the color scale used for render operations.
    pub fn color_scale(&self) -> Result<f32, Error> {
        let mut scale = 0.0;
        let res = unsafe { sys::SDL_GetRenderColorScale(self.raw(), &raw mut scale) };
        if !res {
            return Err(Error::new());
        }
        Ok(scale)
    }

    /// Set the color scale used for render operations.
    ///
    /// The color scale is an additional scale multiplied into the pixel color value while rendering.
    /// This can be used to adjust the brightness of colors during HDR rendering, or changing HDR
    /// video brightness when playing on an SDR display.
    ///
    /// The color scale does not affect the alpha channel, only the color brightness.
    pub fn set_color_scale(&mut self, color_scale: f32) -> Result<(), Error> {
        let res = unsafe { sys::SDL_SetRenderColorScale(self.raw(), color_scale) };
        if !res {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Returns the blend mode used for drawing operations.
    pub fn draw_blend_mode(&self) -> Result<Option<BlendMode>, Error> {
        let mut blend_mode: MaybeUninit<sys::SDL_BlendMode> = MaybeUninit::uninit();
        let res = unsafe { sys::SDL_GetRenderDrawBlendMode(self.raw(), blend_mode.as_mut_ptr()) };
        if !res {
            return Err(Error::new());
        }
        BlendMode::try_from_ll(unsafe { blend_mode.assume_init() })
    }

    /// Set the blend mode used for drawing operations.
    /// If the blend mode is not supported, the closest supported mode is chosen.
    pub fn set_draw_blend_mode(&mut self, blend_mode: BlendMode) -> Result<(), Error> {
        let res = unsafe { sys::SDL_SetRenderDrawBlendMode(self.raw(), blend_mode.to_ll()) };
        if !res {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Returns the VSync of the given renderer.
    pub fn vsync(&self) -> Result<RendererVSync, Error> {
        let mut vsync = 0;
        let result = unsafe { sys::SDL_GetRenderVSync(self.raw(), &raw mut vsync) };
        if !result {
            return Err(Error::new());
        }
        Ok(unsafe { RendererVSync::from_ll_unchecked(vsync) })
    }

    /// Toggle VSync of the given renderer.
    ///
    /// When a renderer is created, vsync defaults to `RendererVSync::Disabled`.
    pub fn set_vsync(&mut self, value: RendererVSync) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetRenderVSync(self.raw(), value.to_raw()) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Get device independent resolution and presentation mode for rendering.
    ///
    /// `RendererLogicalPresentationMode` contains the width and height of the logical rendering output,
    /// or the output size in pixels if a logical resolution is not enabled.
    pub fn logical_presentation(&self) -> Result<RenderLogicalPresentation, Error> {
        let mut w = 0;
        let mut h = 0;
        let mut mode: MaybeUninit<sys::SDL_RendererLogicalPresentation> = MaybeUninit::uninit();
        unsafe {
            let result = sys::SDL_GetRenderLogicalPresentation(
                self.raw(),
                &raw mut w,
                &raw mut h,
                mode.as_mut_ptr(),
            );
            if !result {
                return Err(Error::new());
            }
            let mode = mode.assume_init();
            let mode = RenderLogicalPresentationMode::from_ll_unchecked(mode);
            Ok(RenderLogicalPresentation { w, h, mode })
        }
    }

    /// Set a device independent resolution and presentation mode for rendering.
    ///
    /// This function sets the width and height of the logical rendering output. The renderer will act as if the window
    /// is always the requested dimensions, scaling to the actual window resolution as necessary.
    ///
    /// This can be useful for games that expect a fixed size, but would like to scale the output to whatever is available,
    /// regardless of how a user resizes a window, or if the display is high DPI.
    ///
    /// You can disable logical coordinates by setting the mode to [`RendererLogicalPresentationMode::Disabled`], and in
    /// that case you get the full pixel resolution of the output window; it is safe to toggle logical presentation during
    /// the rendering of a frame: perhaps most of the rendering is done to specific dimensions but to make fonts look sharp,
    /// the app turns off logical presentation while drawing text.
    ///
    /// Letterboxing will only happen if logical presentation is enabled during [`Renderer::present`]; be sure to reenable
    /// it first if you were using it.
    ///
    /// You can convert coordinates in an event into rendering coordinates using SDL_ConvertEventToRenderCoordinates().
    pub fn set_logical_presentation_mode(
        &mut self,
        w: u32,
        h: u32,
        mode: RenderLogicalPresentationMode,
    ) -> Result<(), Error> {
        let w: i32 = i32::try_from(w)?;
        let h: i32 = i32::try_from(h)?;
        let mode = mode.to_ll();
        let result = unsafe { sys::SDL_SetRenderLogicalPresentation(self.raw(), w, h, mode) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Returns the drawing scale for the current target.
    pub fn scale(&self) -> Result<(f32, f32), Error> {
        let mut scale_x = 0.0;
        let mut scale_y = 0.0;
        let result =
            unsafe { sys::SDL_GetRenderScale(self.raw(), &raw mut scale_x, &raw mut scale_y) };
        if !result {
            return Err(Error::new());
        }
        Ok((scale_x, scale_y))
    }

    /// Set the drawing scale for rendering on the current target.
    ///
    /// The drawing coordinates are scaled by the x/y scaling factors before they are used
    /// by the renderer. This allows resolution independent drawing with a single coordinate
    /// system.
    ///
    /// If this results in scaling or subpixel drawing by the rendering backend, it will be
    /// handled using the appropriate quality hints. For best results use integer scaling factors.
    pub fn set_scale(&mut self, x: f32, y: f32) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetRenderScale(self.raw(), x, y) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Returns the drawing area for the current target.
    pub fn viewport(&self) -> Result<Rect, Error> {
        let mut rect: MaybeUninit<sys::SDL_Rect> = MaybeUninit::uninit();
        unsafe {
            let result = sys::SDL_GetRenderViewport(self.raw(), rect.as_mut_ptr());
            if !result {
                return Err(Error::new());
            }
            let rect = rect.assume_init();
            Ok(Rect::from_ll(rect))
        }
    }

    /// Set the drawing area for rendering on the current target.
    ///
    /// Drawing will clip to this area (separately from any clipping done with [`Renderer::set_clip_rect`],
    /// and the top left of the area will become coordinate (0, 0) for future drawing commands.
    ///
    /// The area's width and height must be >= 0.
    pub fn set_viewport(&mut self, rect: Rect) -> Result<(), Error> {
        let result = unsafe {
            sys::SDL_SetRenderViewport(self.raw(), &raw const rect as *const sys::SDL_Rect)
        };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Return whether an explicit rectangle was set as the viewport.
    ///
    /// This is useful if you're saving and restoring the viewport and want to know whether you should
    /// restore a specific rectangle or NULL. Note that the viewport is always reset when changing rendering
    /// targets.
    pub fn is_viewport_set(&mut self) -> bool {
        unsafe { sys::SDL_RenderViewportSet(self.raw()) }
    }

    /// Convert the coordinates in an event to render coordinates.
    ///
    /// This takes into account several states:
    ///
    /// - The window dimensions.
    /// - The logical presentation settings [`RenderLogicalPresentationMode`]
    /// - The scale ([`Renderer::set_scale`])
    /// - The viewport ([`Renderer::set_viewport`])
    ///
    /// Various event types are converted with this function: mouse, touch, pen, etc.
    ///
    /// Touch coordinates are converted from normalized coordinates in the window to non-normalized rendering coordinates.
    ///
    /// Relative mouse coordinates (xrel and yrel event fields) are _also_ converted. Applications that do not want these
    /// fields converted should use [`Renderer::coordinates_from_window`] on the specific event fields instead of converting
    /// the entire event structure.
    ///
    /// Once converted, coordinates may be outside the rendering area.
    pub fn convert_event_to_render_coordinates(&mut self, event: &mut Event) -> Result<(), Error> {
        let event = &raw mut event.0;
        let result = unsafe { sys::SDL_ConvertEventToRenderCoordinates(self.raw(), event) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Returns a point in render coordinates when given a point in window coordinates.
    ///
    /// This takes into account several states:
    ///
    /// - The window dimensions.
    /// - The logical presentation settings [`Renderer::set_logical_presentation_mode`].
    /// - The scale [`Renderer::set_scale`].
    /// - The viewport [`Renderer::set_viewport`].
    pub fn coordinates_from_window(
        &self,
        window_x: f32,
        window_y: f32,
    ) -> Result<(f32, f32), Error> {
        let mut x = 0.0;
        let mut y = 0.0;
        let result = unsafe {
            sys::SDL_RenderCoordinatesFromWindow(
                self.raw(),
                window_x,
                window_y,
                &raw mut x,
                &raw mut y,
            )
        };
        if !result {
            return Err(Error::new());
        }
        Ok((x, y))
    }

    /// Returns a point in window coordinates when given a point in render coordinates.
    ///
    /// This takes into account several states:
    ///
    /// - The window dimensions.
    /// - The logical presentation settings [`Renderer::set_logical_presentation_mode`].
    /// - The scale [`Renderer::set_scale`].
    /// - The viewport [`Renderer::set_viewport`].
    pub fn coordinates_to_window(&self, x: f32, y: f32) -> Result<(f32, f32), Error> {
        let mut window_x = 0.0;
        let mut window_y = 0.0;
        let result = unsafe {
            sys::SDL_RenderCoordinatesToWindow(
                self.raw(),
                x,
                y,
                &raw mut window_x,
                &raw mut window_y,
            )
        };
        if !result {
            return Err(Error::new());
        }
        Ok((window_x, window_y))
    }

    /// Draw a line on the current rendering target at subpixel precision.
    pub fn render_line(&mut self, start: PointF32, end: PointF32) -> Result<(), Error> {
        let result =
            unsafe { sys::SDL_RenderLine(self.raw(), start.x(), start.y(), end.x(), end.y()) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Draw a series of connected lines on the current rendering target at subpixel precision.
    pub fn render_lines(&mut self, points: &[Point]) -> Result<(), Error> {
        let count = i32::try_from(points.len())
            .map_err(|_| Error::register(c"Unable to convert usize to i32."))?;
        let points = points.as_ptr() as *const sys::SDL_FPoint;
        let result = unsafe { sys::SDL_RenderLines(self.raw(), points, count) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Draw a point on the current rendering target at subpixel precision.
    pub fn render_point(&mut self, point: PointF32) -> Result<(), Error> {
        let result = unsafe { sys::SDL_RenderPoint(self.raw(), point.x(), point.y()) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Draw multiple points on the current rendering target at subpixel precision.
    pub fn render_points(&mut self, points: &[PointF32]) -> Result<(), Error> {
        let count = i32::try_from(points.len())
            .map_err(|_| Error::register(c"Unable to convert usize to i32."))?;
        let points = points.as_ptr() as *const sys::SDL_FPoint;
        let result = unsafe { sys::SDL_RenderPoints(self.raw(), points, count) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Draw a rectangle on the current rendering target at subpixel precision.
    pub fn render_rect(&mut self, rect: RectF32) -> Result<(), Error> {
        let result = unsafe { sys::SDL_RenderRect(self.raw(), rect.as_raw()) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Draw some number of rectangles on the current rendering target at subpixel precision.
    pub fn render_rects(&mut self, rects: &[RectF32]) -> Result<(), Error> {
        let count = i32::try_from(rects.len())
            .map_err(|_| Error::register(c"Unable to convert usize to i32."))?;
        let rects = rects.as_ptr() as *const sys::SDL_FRect;
        let result = unsafe { sys::SDL_RenderRects(self.raw(), rects, count) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Fill a rectangle on the current rendering target with the drawing color at subpixel precision.
    pub fn fill_rect(&mut self, rect: RectF32) -> Result<(), Error> {
        let rect = rect.to_ll();
        let result = unsafe { sys::SDL_RenderFillRect(self.raw(), &raw const rect) };
        if !result {
            return Err(Error::new());
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
            return Err(Error::new());
        }
        Ok(())
    }

    /// Draw debug text to a `Renderer`.
    ///
    /// This function will render a string of text to a `Renderer`. Note that this is a convenience function for
    /// debugging, with severe limitations, and not intended to be used for production apps and games.
    ///
    /// Among these limitations:
    /// - It accepts UTF-8 strings, but will only renders ASCII characters.
    /// - It has a single, tiny size (8x8 pixels). One can use logical presentation or scaling to adjust it, but
    /// it will be blurry.
    /// - It uses a simple, hardcoded bitmap font. It does not allow different font selections and it does not
    /// support truetype, for proper scaling.
    /// - It does no word-wrapping and does not treat newline characters as a line break. If the text goes out of
    /// the window, it's gone.
    pub fn render_debug_text(&mut self, x: f32, y: f32, text: &str) -> Result<(), Error> {
        let string = CString::new(text).map_err(|_| {
            Error::register(c"Invalid debug text. Interior null byte found (NulError)")
        })?;
        let result = unsafe { sys::SDL_RenderDebugText(self.raw(), x, y, string.as_ptr()) };
        if !result {
            return Err(Error::new());
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
        texture: &Texture<T>,
        src_rect: Option<RectF32>,
        dest_rect: Option<RectF32>,
    ) -> Result<(), Error> {
        let src_rect_ptr = src_rect
            .as_ref()
            .map(RectF32::as_raw)
            .unwrap_or(core::ptr::null());
        let dest_rect_ptr = dest_rect
            .as_ref()
            .map(RectF32::as_raw)
            .unwrap_or(core::ptr::null());
        let result = unsafe {
            sys::SDL_RenderTexture(
                self.raw(),
                texture.ptr.as_ptr(),
                src_rect_ptr,
                dest_rect_ptr,
            )
        };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Perform a scaled copy using the 9-grid algorithm to the current rendering target at subpixel precision.
    ///
    /// The pixels in the texture are split into a 3x3 grid, using the different corner sizes for each corner,
    /// and the sides and center making up the remaining pixels. The corners are then scaled using `scale` and
    /// fit into the corners of the destination rectangle. The sides and center are then stretched into place
    /// to cover the remaining destination rectangle.
    pub fn render_texture_9_grid(
        &mut self,
        texture: &Texture<T>,
        src_rect: Option<RectF32>,
        left_width: f32,
        right_width: f32,
        top_height: f32,
        bottom_height: f32,
        scale: f32,
        dest_rect: Option<RectF32>,
    ) -> Result<(), Error> {
        let src_rect_ptr = src_rect
            .as_ref()
            .map(RectF32::as_raw)
            .unwrap_or(core::ptr::null());
        let dest_rect_ptr = dest_rect
            .as_ref()
            .map(RectF32::as_raw)
            .unwrap_or(core::ptr::null());
        let result = unsafe {
            sys::SDL_RenderTexture9Grid(
                self.raw(),
                texture.raw(),
                src_rect_ptr,
                left_width,
                right_width,
                top_height,
                bottom_height,
                scale,
                dest_rect_ptr,
            )
        };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Tile a portion of the texture to the current rendering target at subpixel precision.
    ///
    /// The pixels in `srcrect` will be repeated as many times as needed to completely fill `dest_rect`.
    pub fn render_texture_tiled(
        &mut self,
        texture: &Texture<T>,
        src_rect: Option<RectF32>,
        scale: f32,
        dest_rect: Option<RectF32>,
    ) -> Result<(), Error> {
        let src_rect_ptr = src_rect
            .as_ref()
            .map(RectF32::as_raw)
            .unwrap_or(core::ptr::null());
        let dest_rect_ptr = dest_rect
            .as_ref()
            .map(RectF32::as_raw)
            .unwrap_or(core::ptr::null());
        let result = unsafe {
            sys::SDL_RenderTextureTiled(
                self.raw(),
                texture.raw(),
                src_rect_ptr,
                scale,
                dest_rect_ptr,
            )
        };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Copy a portion of the source texture to the current rendering target, with rotation and flipping,
    /// at subpixel precision.
    pub fn render_texture_rotated(
        &mut self,
        texture: &Texture<T>,
        src_rect: Option<RectF32>,
        dest_rect: Option<RectF32>,
        angle: f64,
        center: Option<PointF32>,
        flip: Option<FlipMode>,
    ) -> Result<(), Error> {
        let src_rect_ptr = src_rect
            .as_ref()
            .map(RectF32::as_raw)
            .unwrap_or(core::ptr::null());
        let dest_rect_ptr = dest_rect
            .as_ref()
            .map(RectF32::as_raw)
            .unwrap_or(core::ptr::null());
        let center_ptr = center
            .as_ref()
            .map(PointF32::as_raw)
            .unwrap_or(core::ptr::null());
        let flip_mode = flip
            .map(|f| f.to_ll())
            .unwrap_or(sys::SDL_FlipMode_SDL_FLIP_NONE);
        let result = unsafe {
            sys::SDL_RenderTextureRotated(
                self.raw(),
                texture.raw(),
                src_rect_ptr,
                dest_rect_ptr,
                angle,
                center_ptr,
                flip_mode,
            )
        };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Copy a portion of the source texture to the current rendering target, with affine transform, at subpixel precision.
    ///
    /// - `origin`: indicates where the top-left corner of `src_rect` should be mapped to, or `None` for the rendering
    /// target's origin.
    /// - `right`: indicates where the top-right corner of `src_rect` should be mapped to, or `None` for the rendering
    /// target's top-right corner.
    /// - `left`: indicates where the bottom-left corner of `src_rect` should be mapped to, or `None` for the rendering
    /// target's bottom-left corner.
    pub fn render_texture_affine(
        &mut self,
        texture: &Texture<T>,
        src_rect: Option<RectF32>,
        origin: Option<PointF32>,
        right: Option<PointF32>,
        down: Option<PointF32>,
    ) -> Result<(), Error> {
        let src_rect_ptr = src_rect
            .as_ref()
            .map(RectF32::as_raw)
            .unwrap_or(core::ptr::null());
        let origin_ptr = origin
            .as_ref()
            .map(PointF32::as_raw)
            .unwrap_or(core::ptr::null());
        let right_ptr = right
            .as_ref()
            .map(PointF32::as_raw)
            .unwrap_or(core::ptr::null());
        let down_ptr = down
            .as_ref()
            .map(PointF32::as_raw)
            .unwrap_or(core::ptr::null());
        let result = unsafe {
            sys::SDL_RenderTextureAffine(
                self.raw(),
                texture.raw(),
                src_rect_ptr,
                origin_ptr,
                right_ptr,
                down_ptr,
            )
        };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Render a list of triangles, optionally using a texture and indices into the vertex array.
    /// Color and alpha modulation is done per vertex ([`Renderer::color_mod`] and [`Texture::alpha_mod`] are ignored).
    pub fn render_geometry(
        &mut self,
        texture: Option<&Texture<T>>,
        vertices: &[Vertex],
        indices: &[i32],
    ) -> Result<(), Error> {
        let texture_ptr = texture.map(Texture::raw).unwrap_or(core::ptr::null_mut());
        let indices_ptr = if indices.is_empty() {
            core::ptr::null()
        } else {
            indices.as_ptr()
        };
        let result = unsafe {
            sys::SDL_RenderGeometry(
                self.raw(),
                texture_ptr,
                vertices.as_ptr() as *const sys::SDL_Vertex, // safe because the representation in-memory is the same
                i32::try_from(vertices.len())?,
                indices_ptr,
                i32::try_from(indices.len())?,
            )
        };
        if !result {
            return Err(Error::new());
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
        texture: Option<Texture<T>>,
    ) -> Result<Option<Texture<T>>, Error> {
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
                let result = unsafe { sys::SDL_SetRenderTarget(self.raw(), texture.ptr.as_ptr()) };
                if !result {
                    return Err(Error::new());
                }
            }
            _ => {
                let result = unsafe { sys::SDL_SetRenderTarget(self.raw(), core::ptr::null_mut()) };
                if !result {
                    return Err(Error::new());
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
    /// [`Renderer::replace_render_target`] afterwards, as textures by themselves do not have a concept of backbuffers.
    /// Calling [`Renderer::present`] while rendering to a texture will still update the screen with any current drawing that
    /// has been done _to the window itself_.
    pub fn present(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_RenderPresent(self.raw()) };
        if !result {
            return Err(Error::new());
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
            return Err(Error::new());
        }
        Ok(())
    }

    /// Force the rendering context to flush any pending commands and state.
    ///
    /// You do not need to (and in fact, shouldn't) call this function unless you are planning
    /// to call into OpenGL/Direct3D/Metal/whatever directly, in addition to using a `Renderer`.
    ///
    /// This is for a very-specific case: if you are using SDL's render API, and you plan to make
    /// OpenGL/D3D/whatever calls in addition to SDL render API calls. If this applies, you
    /// should call this function between calls to SDL's render API and the low-level API you're
    /// using in cooperation.
    ///
    /// In all other cases, you can ignore this function.
    ///
    /// This call makes SDL flush any pending rendering work it was queueing up to do later in a
    /// single batch, and marks any internal cached state as invalid, so it'll prepare all its
    /// state again later, from scratch.
    ///
    /// This means you do not need to save state in your rendering code to protect the `Renderer`.
    /// However, there lots of arbitrary pieces of Direct3D and OpenGL state that can confuse
    /// things; you should use your best judgment and be prepared to make changes if specific state
    /// needs to be protected.
    pub fn flush(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_FlushRenderer(self.raw()) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Returns a mutable pointer to the underlying raw `SDL_Renderer` used by this `Renderer`.
    #[inline]
    pub fn raw(&self) -> *mut sys::SDL_Renderer {
        self.internal.ptr.as_ptr()
    }
}

impl<T> Drop for Renderer<T> {
    fn drop(&mut self) {
        // If there's still a reference to the internal renderer,
        // we move the owner to the internal renderer so destroying it
        // becomes the internal renderer's responsibility.
        if Rc::strong_count(&self.internal) > 1 {
            let Some(owner) = self.owner.take() else {
                return;
            };
            let Ok(mut drop_owner) = self.internal.owner.try_borrow_mut() else {
                return;
            };
            let _ = drop_owner.insert(owner);
        }
    }
}

/// VSync behavior of a renderer.
///
/// When a renderer is created, vsync defaults to `RendererVSync::Disabled`.
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

// Describes how a renderer's logical size is mapped to its' output.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RenderLogicalPresentation {
    pub w: i32,
    pub h: i32,
    pub mode: RenderLogicalPresentationMode,
}

/// How the logical size is mapped to the output.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderLogicalPresentationMode {
    /// There is no logical size in effect
    Disabled = sys::SDL_RendererLogicalPresentation_SDL_LOGICAL_PRESENTATION_DISABLED,
    /// The rendered content is stretched to the output resolution.
    Stretch = sys::SDL_RendererLogicalPresentation_SDL_LOGICAL_PRESENTATION_STRETCH,
    /// The rendered content is fit to the largest dimension and the other dimension is letterboxed with black bars.
    Letterbox = sys::SDL_RendererLogicalPresentation_SDL_LOGICAL_PRESENTATION_LETTERBOX,
    /// The rendered content is fit to the smallest dimension and the other dimension extends beyond the output bounds.
    Overscan = sys::SDL_RendererLogicalPresentation_SDL_LOGICAL_PRESENTATION_OVERSCAN,
    /// The rendered content is scaled up by integer multiples to fit the output resolution.
    IntegerScale = sys::SDL_RendererLogicalPresentation_SDL_LOGICAL_PRESENTATION_INTEGER_SCALE,
}

impl RenderLogicalPresentationMode {
    /// SAFETY: `value` must be a valid variant of the enum.
    unsafe fn from_ll_unchecked(value: u32) -> Self {
        unsafe { core::mem::transmute(value) }
    }

    pub fn to_ll(&self) -> u32 {
        *self as u32
    }
}

/// Driver-specific representation of pixel data.
///
/// This struct holds a shared reference to its' parent (a raw [`sys::SDL_Renderer`])
/// via ref-count. A consequence of this is, to truly destroy the parent renderer
/// (equivalent to `SDL_DestroyRenderer`) and its' backbuffer, all `Texture`s created
/// by that renderer must be dropped.
pub struct Texture<T = Window> {
    _renderer: Rc<RendererInternal<T>>,
    ptr: NonNull<sys::SDL_Texture>,
}

impl<T> Texture<T> {
    /// Creates a texture for a rendering context.
    ///
    /// The contents of a texture when first created are not defined.
    pub fn new(
        renderer: &mut Renderer<T>,
        format: PixelFormat,
        access: TextureAccess,
        width: u32,
        height: u32,
    ) -> Result<Self, Error> {
        let format = format.to_ll();
        let access = access.to_ll();
        let ptr = NonNull::new(unsafe {
            sys::SDL_CreateTexture(
                renderer.raw(),
                format,
                access,
                width.try_into()?,
                height.try_into()?,
            )
        })
        .ok_or(Error::new())?;
        Ok(Self {
            _renderer: Rc::clone(&renderer.internal),
            ptr,
        })
    }

    #[inline]
    pub fn w(&self) -> i32 {
        unsafe { (*self.raw()).w }
    }

    #[inline]
    pub fn h(&self) -> i32 {
        unsafe { (*self.raw()).h }
    }

    #[inline]
    pub fn format(&self) -> PixelFormat {
        unsafe { PixelFormat::from_ll_unchecked((*self.raw()).format) }
    }

    /// Returns the size of a texture, as floating point values.
    pub fn size(&self) -> Result<(f32, f32), Error> {
        let mut w = 0.0;
        let mut h = 0.0;
        let result = unsafe { sys::SDL_GetTextureSize(self.raw(), &raw mut w, &raw mut h) };
        if !result {
            return Err(Error::new());
        }
        Ok((w, h))
    }

    /// Create a texture from an existing surface.
    ///
    /// The surface is not modified by this function.
    ///
    /// The [`TextureAccess`] hint for the created texture is [`TextureAccess::Static`].
    ///
    /// The pixel format of the created texture may be different from the pixel format of the surface.
    pub fn from_surface(renderer: &mut Renderer<T>, surface: &SurfaceRef) -> Result<Self, Error> {
        let ptr = NonNull::new(unsafe {
            sys::SDL_CreateTextureFromSurface(renderer.raw(), surface.raw() as *mut _)
        })
        .ok_or(Error::new())?;
        Ok(Texture {
            _renderer: Rc::clone(&renderer.internal),
            ptr,
        })
    }

    /// Returns the additional alpha value multiplied into render copy operations.
    pub fn alpha_mod(&self) -> Result<u8, Error> {
        let mut alpha = 0;
        let result = unsafe { sys::SDL_GetTextureAlphaMod(self.raw(), &raw mut alpha) };
        if !result {
            return Err(Error::new());
        }
        Ok(alpha)
    }

    /// Set an additional alpha value multiplied into render copy operations.
    ///
    /// When this texture is rendered, during the copy operation the source alpha value is modulated by
    /// this alpha value according to the following formula:
    ///
    /// `srcA = srcA * (alpha / 255)`
    ///
    /// Alpha modulation is not always supported by the renderer; it will return an `Error` if alpha
    /// modulation is not supported.
    pub fn set_alpha_mod(&mut self, alpha_mod: u8) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetTextureAlphaMod(self.raw(), alpha_mod) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Returns the additional alpha value multiplied into render copy operations.
    pub fn alpha_mod_f32(&self) -> Result<f32, Error> {
        let mut alpha = 0.0;
        let result = unsafe { sys::SDL_GetTextureAlphaModFloat(self.raw(), &raw mut alpha) };
        if !result {
            return Err(Error::new());
        }
        Ok(alpha)
    }

    /// Set an additional alpha value multiplied into render copy operations.
    ///
    /// When this texture is rendered, during the copy operation the source alpha value is modulated by
    /// this alpha value according to the following formula:
    ///
    /// `srcA = srcA * alpha`
    ///
    /// Alpha modulation is not always supported by the renderer; it will return an `Error` if alpha
    /// modulation is not supported.
    pub fn set_alpha_mod_f32(&mut self, alpha_mod: f32) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetTextureAlphaModFloat(self.raw(), alpha_mod) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Returns the blend mode used for texture copy operations.
    pub fn blend_mode(&self) -> Result<Option<BlendMode>, Error> {
        let mut blend_mode: sys::SDL_BlendMode = 0;
        let result: bool = unsafe { sys::SDL_GetTextureBlendMode(self.raw(), &raw mut blend_mode) };
        if !result {
            return Err(Error::new());
        }
        // ...
        BlendMode::try_from_ll(blend_mode)
    }

    /// Set the blend mode for a texture, used by [`Renderer::render_texture`].
    ///
    /// If the blend mode is not supported, the closest supported mode is chosen and this function
    /// returns an `Error`.
    pub fn set_blend_mode(&mut self, mode: BlendMode) -> Result<(), Error> {
        let mode = mode.to_ll();
        let result = unsafe { sys::SDL_SetTextureBlendMode(self.raw(), mode) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Returns the additional color value multiplied into render copy operations.
    pub fn color_mod(&self) -> Result<(u8, u8, u8), Error> {
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        let result =
            unsafe { sys::SDL_GetTextureColorMod(self.raw(), &raw mut r, &raw mut g, &raw mut b) };
        if !result {
            return Err(Error::new());
        }
        Ok((r, g, b))
    }

    /// Set an additional color value multiplied into render copy operations.
    ///
    /// When this texture is rendered, during the copy operation each source color channel is modulated
    /// by the appropriate color value according to the following formula:
    ///
    /// `srcC = srcC * (color / 255)`
    ///
    /// Color modulation is not always supported by the renderer; it will return an `Error` if color
    /// modulation is not supported.
    pub fn set_color_mod(&mut self, color_mod: (u8, u8, u8)) -> Result<(), Error> {
        let (r, g, b) = color_mod;
        let result = unsafe { sys::SDL_SetTextureColorMod(self.raw(), r, g, b) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Set an additional color value multiplied into render copy operations.
    ///
    /// When this texture is rendered, during the copy operation each source color channel is modulated
    /// by the appropriate color value according to the following formula:
    ///
    /// `srcC = srcC * color`
    ///
    /// Color modulation is not always supported by the renderer; it will return an `Error` if color
    /// modulation is not supported.
    pub fn set_color_mod_f32(&mut self, color_mod: (f32, f32, f32)) -> Result<(), Error> {
        let (r, g, b) = color_mod;
        let result = unsafe { sys::SDL_SetTextureColorModFloat(self.raw(), r, g, b) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Returns the scale mode used for texture scale operations.
    pub fn scale_mode(&self) -> Result<ScaleMode, Error> {
        let mut scale_mode: MaybeUninit<sys::SDL_ScaleMode> = MaybeUninit::uninit();
        unsafe {
            let result = sys::SDL_GetTextureScaleMode(self.raw(), scale_mode.as_mut_ptr());
            if !result {
                return Err(Error::new());
            }
            Ok(ScaleMode::from_ll_unchecked(scale_mode.assume_init()))
        }
    }

    /// Set the scale mode used for texture scale operations.
    ///
    /// The default texture scale mode is [`ScaleMode::Linear`].
    ///
    /// If the scale mode is not supported, the closest supported mode is chosen.
    pub fn set_scale_mode(&mut self, scale_mode: ScaleMode) -> Result<(), Error> {
        let scale_mode = scale_mode.to_ll();
        let result = unsafe { sys::SDL_SetTextureScaleMode(self.raw(), scale_mode) };
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Lock a portion of the texture for **write-only** pixel access.
    ///
    /// As an optimization, the pixels made available for editing don't necessarily contain the old texture data.
    /// This is a write-only operation, and if you need to keep a copy of the texture data you should do that at
    /// the application level.
    ///
    /// You must drop the lock to unlock the pixels and apply any changes.
    pub fn lock(&mut self, rect: Option<Rect>) -> Result<TextureLock<T>, Error> {
        TextureLock::new(self, rect)
    }

    /// SAFETY: texture must come directly from SDL and it *must* be owned by the caller.
    unsafe fn from_mut_ptr(renderer: &mut Renderer<T>, ptr: *mut sys::SDL_Texture) -> Self {
        Self {
            _renderer: Rc::clone(&renderer.internal),
            ptr: NonNull::new_unchecked(ptr),
        }
    }

    #[inline]
    fn raw(&self) -> *mut sys::SDL_Texture {
        self.ptr.as_ptr()
    }
}

impl<T> Drop for Texture<T> {
    fn drop(&mut self) {
        unsafe { sys::SDL_DestroyTexture(self.ptr.as_ptr()) };
    }
}

/// A texture that's locked for writing.
pub struct TextureLock<'a, T> {
    /// A pointer to the pixels array, owned by SDL
    pixels: &'a mut [u8],
    texture: &'a Texture<T>, // we need to store this to drop the lock
    pitch: i32,
}

impl<'a, T> TextureLock<'a, T> {
    fn new(texture: &'a mut Texture<T>, rect: Option<Rect>) -> Result<Self, Error> {
        unsafe {
            let mut pitch = 0;
            let mut pixels = core::ptr::null_mut();
            let rect = rect.unwrap_or({
                let w = u32::try_from(texture.w())?;
                let h = u32::try_from(texture.h())?;
                Rect::new(0, 0, w, h)
            });
            let result = sys::SDL_LockTexture(
                texture.raw(),
                rect.as_raw(),
                &raw mut pixels,
                &raw mut pitch,
            );
            if !result {
                return Err(Error::new());
            }
            // SDL gives us a contiguous buffer to write the pixels into.
            let size = texture
                .format()
                .details()?
                .byte_size_from_pitch_and_height(pitch as usize, rect.h());
            let pixels = core::slice::from_raw_parts_mut(pixels as *mut u8, size);
            Ok(Self {
                pixels,
                texture,
                pitch,
            })
        }
    }
}

impl<T> TextureLock<'_, T> {
    pub fn pitch(&self) -> usize {
        self.pitch as usize
    }

    pub fn pixels_mut(&mut self) -> &mut [u8] {
        // use the pitch to calculate the thing
        self.pixels
    }
}

impl<T> Drop for TextureLock<'_, T> {
    fn drop(&mut self) {
        unsafe { sys::SDL_UnlockTexture(self.texture.raw()) };
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

#[repr(transparent)]
pub struct Vertex(sys::SDL_Vertex);

impl Vertex {
    pub fn new(position: PointF32, color: ColorF32, tex_coord: PointF32) -> Self {
        Self(sys::SDL_Vertex {
            position: position.to_ll(),
            color: color.to_ll(),
            tex_coord: tex_coord.to_ll(),
        })
    }

    #[inline]
    pub fn position(&self) -> PointF32 {
        PointF32::new(self.0.position.x, self.0.position.y)
    }

    #[inline]
    pub fn color(&self) -> ColorF32 {
        ColorF32::new(
            self.0.color.r,
            self.0.color.g,
            self.0.color.b,
            self.0.color.a,
        )
    }

    #[inline]
    pub fn tex_coord(&self) -> PointF32 {
        PointF32::new(self.0.tex_coord.x, self.0.tex_coord.y)
    }

    #[inline]
    pub fn to_ll(&self) -> sys::SDL_Vertex {
        self.0
    }

    #[inline]
    pub fn raw(&self) -> *const sys::SDL_Vertex {
        &raw const self.0
    }
}

struct RendererInternal<T> {
    ptr: NonNull<sys::SDL_Renderer>,
    /// The owner of this renderer (a window or a surface).
    /// If the parent [`Renderer`] gets dropped before its' [`Texture`]s, then
    /// we move the owner to this struct. That's why we need the [`RefCell`].
    owner: RefCell<Option<T>>,
}

impl<T> Drop for RendererInternal<T> {
    fn drop(&mut self) {
        unsafe { sys::SDL_DestroyRenderer(self.ptr.as_ptr()) };
        // if the owner is Some it will get destroyed automatically :)
    }
}
