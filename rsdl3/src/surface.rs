use crate::blendmode::BlendMode;
use crate::init::VideoSubsystem;
use crate::iostream::IOStream;
#[allow(unused)]
use crate::pixels::PixelFormatDetails;
use crate::pixels::{Color, ColorF32, Colorspace, Palette, PixelFormat};
use crate::rect::Rect;
use crate::render::Renderer;
use crate::{sys, Error};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};

/// An owned collection of pixels used in software blitting.
///
/// Pixels are arranged in memory in rows, with the top row first. Each row occupies an amount
/// of memory given by the pitch (sometimes known as the row stride in non-SDL APIs).
///
/// Within each row, pixels are arranged from left to right until the width is reached. Each
/// pixel occupies a number of bits appropriate for its format, with most formats representing
/// each pixel as one or more whole bytes (in some indexed formats, instead multiple pixels
/// are packed into each byte), and a byte order given by the format. After encoding all
/// pixels, any remaining bytes to reach the pitch are used as padding to reach a desired
/// alignment, and have undefined contents.
///
/// When a surface holds YUV format data, the planes are assumed to be contiguous without padding
/// between them, e.g. a 32x32 surface in NV12 format with a pitch of 32 would consist of 32x32
/// bytes of Y plane followed by 32x16 bytes of UV plane.
pub struct Surface<'a> {
    video: VideoSubsystem,
    ptr: *mut sys::SDL_Surface,
    _marker: PhantomData<&'a ()>,
}

impl Surface<'static> {
    pub fn new(video: &VideoSubsystem, w: u32, h: u32, format: PixelFormat) -> Result<Self, Error> {
        let w = w.clamp(0, i32::MAX as u32) as i32;
        let h = h.clamp(0, i32::MAX as u32) as i32;
        let ptr = unsafe { sys::SDL_CreateSurface(w, h, format.to_ll()) };
        if ptr.is_null() {
            return Err(Error);
        }
        Ok(Self {
            video: video.clone(),
            ptr,
            _marker: PhantomData,
        })
    }

    #[cfg(feature = "image")]
    #[cfg_attr(docsrs, doc(cfg(feature = "image")))]
    /// Creates a new `Surface` by loading an image from the specified file path.
    pub fn from_image(video: &VideoSubsystem, path: &str) -> Result<Self, Error> {
        use alloc::ffi::CString;
        let path = CString::new(path)?;
        unsafe {
            let surface = sys::image::IMG_Load(path.as_ptr());
            if surface.is_null() {
                return Err(Error);
            }
            Ok(Self::from_mut_ptr(video, surface))
        }
    }

    /// Load a BMP image from a file.
    pub fn load_bmp(video: &VideoSubsystem, path: &str) -> Result<Self, Error> {
        use alloc::ffi::CString;
        let path = CString::new(path)?;
        unsafe {
            let surface = sys::SDL_LoadBMP(path.as_ptr());
            if surface.is_null() {
                return Err(Error);
            }
            Ok(Self::from_mut_ptr(video, surface))
        }
    }

    /// Creates a software `Renderer` from an existing `Surface`.
    ///
    /// The surface can later be borrowed by calling `Renderer::as_surface_ref` or `Renderer::as_surface_mut`.
    ///
    /// This is equivalent to [`Renderer::from_surface`].
    pub fn create_renderer(self) -> Result<Renderer, Error> {
        Renderer::from_surface(self)
    }
}

impl<'a> Surface<'a> {
    /// Load a BMP image from a seekable SDL data stream.
    pub fn load_bmp_from_io<'b>(video: &VideoSubsystem, src: IOStream<'b>) -> Result<Self, Error> {
        let ptr = unsafe { sys::SDL_LoadBMP_IO(src.raw(), false) };
        if ptr.is_null() {
            return Err(Error);
        }
        Ok(unsafe { Self::from_mut_ptr(video, ptr) })
    }

    /// Allocate a new surface with a specific pixel format and existing pixel data.
    ///
    /// Mutably borrows `pixels` for the lifetime of the returned `Surface`.
    pub fn from_pixels(
        video: &VideoSubsystem,
        format: PixelFormat,
        pixels: &'a mut [u8],
        width: u32,
        height: u32,
    ) -> Result<Surface<'a>, Error> {
        // we need to make sure we won't overflow the byte buffer...
        let details = format.details()?;
        let bytes_per_pixel = details.bytes_per_pixel();
        let total_bytes = usize::try_from(
            width
                .saturating_mul(height)
                .saturating_mul(bytes_per_pixel as u32), // cast ok because we're going from u8 to i32
        )?;
        if total_bytes > pixels.len() {
            return Err(Error::register(c"Invalid surface pixel parameters"));
        }
        let width = i32::try_from(width)?;
        let height = i32::try_from(height)?;
        let pitch = width.saturating_mul(bytes_per_pixel as i32);
        let ptr = unsafe {
            sys::SDL_CreateSurfaceFrom(
                width,
                height,
                format.to_ll(),
                pixels.as_mut_ptr() as *mut _,
                pitch,
            )
        };
        if ptr.is_null() {
            return Err(Error);
        }
        Ok(unsafe { Surface::from_mut_ptr(video, ptr) })
    }

    /// Copy an existing surface to a new surface of the specified format.
    ///
    /// This function is used to optimize images for faster *repeat* blitting. This is accomplished by converting
    /// the original and storing the result as a new surface. The new, optimized surface can then be used as the
    /// source for future blits, making them faster.
    ///
    /// If you are converting to an indexed surface and want to map colors to a palette, you can use
    /// [`Surface::convert_surface_and_colorspace`] instead.
    pub fn convert(&self, format: PixelFormat) -> Result<Surface<'a>, Error> {
        let ptr = unsafe { sys::SDL_ConvertSurface(self.ptr, format.to_ll()) };
        if ptr.is_null() {
            return Err(Error);
        }
        Ok(unsafe { Surface::from_mut_ptr(&self.video, ptr) })
    }

    /// Creates a new surface identical to the existing surface.
    ///
    /// If the original surface has alternate images, the new surface will have a reference to them as well.
    ///
    /// This method is equivalent to [`Surface::uplicate`] and [`VideoSubsystem::duplicate_surface`].
    #[inline]
    pub fn try_clone(&self) -> Result<Surface<'static>, Error> {
        self.duplicate()
    }

    /// Creates a new surface identical to the existing surface.
    ///
    /// If the original surface has alternate images, the new surface will have a reference to them as well.
    ///
    /// This method is equivalent to [`VideoSubsystem::duplicate_surface`].
    pub fn duplicate(&self) -> Result<Surface<'static>, Error> {
        self.deref().duplicate(&self.video)
    }

    /// SAFETY: ptr must be valid
    pub(crate) unsafe fn from_mut_ptr(video: &VideoSubsystem, ptr: *mut sys::SDL_Surface) -> Self {
        Self {
            video: video.clone(),
            ptr,
            _marker: PhantomData,
        }
    }
}

impl<'a> Drop for Surface<'a> {
    fn drop(&mut self) {
        unsafe { sys::SDL_DestroySurface(self.ptr) };
    }
}

impl<'a> Deref for Surface<'a> {
    type Target = SurfaceRef;

    fn deref(&self) -> &Self::Target {
        unsafe { SurfaceRef::from_ptr(self.ptr) }
    }
}

impl<'a> DerefMut for Surface<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { SurfaceRef::from_mut_ptr(self.ptr) }
    }
}

/// A zero-sized type that functions as a reference to an SDL surface.
///
/// This type is only exposed as a reference such that its' lifetime is bound to an owner.
///
/// Check out [`Surface`] for the owned version of this struct.
pub struct SurfaceRef {
    _inner: PhantomData<*const ()>, // !Send + !Sync
}

impl SurfaceRef {
    pub(crate) unsafe fn from_ptr<'a>(ptr: *const sys::SDL_Surface) -> &'a Self {
        &*(ptr as *const SurfaceRef)
    }

    pub(crate) unsafe fn from_mut_ptr<'a>(ptr: *mut sys::SDL_Surface) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }

    /// Save surface to a file.
    ///
    /// Surfaces with a 24-bit, 32-bit and paletted 8-bit format get saved in the BMP directly.
    /// Other RGB formats with 8-bit or higher get converted to a 24-bit surface or, if they
    /// have an alpha mask or a colorkey, to a 32-bit surface before they are saved. YUV and
    /// paletted 1-bit and 4-bit formats are not supported.
    pub fn save_bmp(&self, path: &str) -> Result<(), Error> {
        use alloc::ffi::CString;
        let path = CString::new(path)?;
        let result = unsafe { sys::SDL_SaveBMP(self.raw(), path.as_ptr()) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Save a surface to a seekable SDL data stream in BMP format.
    ///
    /// Surfaces with a 24-bit, 32-bit and paletted 8-bit format get saved in the BMP directly.
    /// Other RGB formats with 8-bit or higher get converted to a 24-bit surface or, if they
    /// have an alpha mask or a colorkey, to a 32-bit surface before they are saved. YUV and
    /// paletted 1-bit and 4-bit formats are not supported.
    pub fn save_bmp_into_iostream(&self, stream: &mut IOStream) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SaveBMP_IO(self.raw(), stream.raw(), false) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Creates a new surface identical to the existing surface.
    /// If the original surface has alternate images, the new surface will have a reference to them as well.
    ///
    /// This function takes a `VideoSubsystem` parameter due to lifetime requirements: the
    /// returned surface cannot outlive the subsystem and `SurfaceRef` can't access it on
    /// its' own.
    pub fn duplicate(&self, video: &VideoSubsystem) -> Result<Surface<'static>, Error> {
        let ptr = unsafe { sys::SDL_DuplicateSurface(self.raw()) };
        if ptr.is_null() {
            return Err(Error);
        }
        Ok(unsafe { Surface::from_mut_ptr(video, ptr) })
    }

    /// Creates a new surface identical to the existing surface, scaled to the desired size.
    ///
    /// This function takes a `VideoSubsystem` parameter due to lifetime requirements: the
    /// returned surface cannot outlive the subsystem and `SurfaceRef` can't access it on
    /// its' own.
    pub fn scale(
        &self,
        video: &VideoSubsystem,
        width: u32,
        height: u32,
        scale_mode: ScaleMode,
    ) -> Result<Surface<'static>, Error> {
        let width = i32::try_from(width)?;
        let height = i32::try_from(height)?;
        let ptr = unsafe { sys::SDL_ScaleSurface(self.raw(), width, height, scale_mode.to_ll()) };
        if ptr.is_null() {
            return Err(Error);
        }
        Ok(unsafe { Surface::from_mut_ptr(video, ptr) })
    }

    /// Returns the additional alpha value used in blit operations.
    pub fn alpha_mod(&self) -> Result<u8, Error> {
        let mut alpha_mod: u8 = 0;
        let result =
            unsafe { sys::SDL_GetSurfaceAlphaMod(self.raw() as *mut _, &raw mut alpha_mod) };
        if !result {
            return Err(Error);
        }
        Ok(alpha_mod)
    }

    /// Set an additional alpha value used in blit operations.
    ///
    /// When this surface is blitted, during the blit operation the source alpha value is modulated by
    /// this alpha value according to the following formula:
    ///
    /// `srcA = srcA * (alpha / 255)`
    pub fn set_alpha_mod(&mut self, alpha_mod: u8) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetSurfaceAlphaMod(self.raw(), alpha_mod) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Returns the blend mode used for blit operations.
    pub fn blend_mode(&self) -> Result<Option<BlendMode>, Error> {
        let mut blend_mode = 0;
        let result =
            unsafe { sys::SDL_GetSurfaceBlendMode(self.raw() as *mut _, &raw mut blend_mode) };
        if !result {
            return Err(Error);
        }
        BlendMode::try_from_ll(blend_mode)
    }

    /// Set the blend mode used for blit operations.
    ///
    /// To copy a surface to another surface (or texture) without blending with the existing data, the
    /// blendmode of the SOURCE surface should be set to `None`.
    pub fn set_blend_mode(&mut self, blend_mode: Option<BlendMode>) -> Result<(), Error> {
        let blend_mode = BlendMode::option_to_ll(blend_mode);
        let result = unsafe { sys::SDL_SetSurfaceBlendMode(self.raw(), blend_mode) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Returns the clipping rectangle for a surface.
    ///
    /// When `self` is the destination of a blit, only the area within the clip rectangle is drawn into.
    pub fn clip_rect(&self) -> Result<Rect, Error> {
        let mut rect = MaybeUninit::uninit();
        let rect = unsafe {
            let result = sys::SDL_GetSurfaceClipRect(self.raw() as *mut _, rect.as_mut_ptr());
            if !result {
                return Err(Error);
            }
            rect.assume_init()
        };
        Ok(Rect::from_ll(rect))
    }

    /// Set the clipping rectangle for a surface.
    ///
    /// When `self` is the destination of a blit, only the area within the clip rectangle is drawn into.
    ///
    /// Note that blits are automatically clipped to the edges of the source and destination surfaces.
    pub fn set_clip_rect(&mut self, rect: Option<Rect>) -> Result<(), Error> {
        let clip_rect = rect.map(Rect::to_ll);
        let clip_rect_ptr = clip_rect
            .as_ref()
            .map_or(core::ptr::null(), core::ptr::from_ref);
        let result = unsafe { sys::SDL_SetSurfaceClipRect(self.raw(), clip_rect_ptr) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Get the color key (transparent pixel) for a surface.
    ///
    /// The color key is a pixel of the format used by the surface, as generated by
    /// [`crate::pixels::PixelFormatDetails::map_rgb`]
    ///
    /// If the surface doesn't have color key enabled this function returns an `Error`.
    pub fn color_key(&self) -> Result<u32, Error> {
        let mut color_key = 0;
        let result =
            unsafe { sys::SDL_GetSurfaceColorKey(self.raw() as *mut _, &raw mut color_key) };
        if !result {
            return Err(Error);
        }
        Ok(color_key)
    }

    /// Set the color key (transparent pixel) in a surface.
    ///
    /// The color key defines a pixel value that will be treated as transparent in a blit.
    /// For example, one can use this to specify that cyan pixels should be considered transparent,
    /// and therefore not rendered.
    ///
    /// `color_key` is a pixel of the format used by the surface, as generated by
    /// [`crate::pixels::PixelFormatDetails::map_rgb`].
    pub fn set_color_key(&mut self, color_key: Option<u32>) -> Result<(), Error> {
        let result = match color_key {
            Some(color_key) => unsafe { sys::SDL_SetSurfaceColorKey(self.raw(), true, color_key) },
            None => unsafe { sys::SDL_SetSurfaceColorKey(self.raw(), false, 0) },
        };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Returns the additional color value multiplied into blit operations.
    pub fn color_mod(&self) -> Result<(u8, u8, u8), Error> {
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        let result = unsafe {
            sys::SDL_GetSurfaceColorMod(self.raw() as *mut _, &raw mut r, &raw mut g, &raw mut b)
        };
        if !result {
            return Err(Error);
        }
        Ok((r, g, b))
    }

    /// Set an additional color value multiplied into blit operations.
    ///
    /// When this surface is blitted, during the blit operation each source color channel is modulated
    /// by the appropriate color value according to the following formula:
    ///
    /// `srcC = srcC * (color / 255)`
    pub fn set_color_mod(&mut self, r: u8, g: u8, b: u8) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetSurfaceColorMod(self.raw(), r, g, b) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    pub fn colorspace(&self) -> Colorspace {
        let result = unsafe { sys::SDL_GetSurfaceColorspace(self.raw()) };
        Colorspace(result)
    }

    /// Performs a fast blit from the source surface to the destination surface with clipping.
    ///
    /// If either `src_rect` or `dest_rect` are `None`, the entire surface (`self` or `dest`) is copied while
    /// ensuring clipping to the `clip_rect` of `dest_rect`.
    ///
    /// The blit semantics for surfaces with and without blending and colorkey are defined as follows:
    ///
    /// Check [`sys::SDL_BlitSurface`] for more details on blit semantics.
    pub fn blit(
        &mut self,
        src_rect: Option<Rect>,
        dest: &mut SurfaceRef,
        dest_rect: Option<Rect>,
    ) -> Result<(), Error> {
        let src_rect = src_rect.map(Rect::to_ll);
        let src_rect_ptr = src_rect
            .as_ref()
            .map_or(core::ptr::null(), core::ptr::from_ref);

        // SDL actually ignores the width and height of the dest rectangle.
        let dest_rect = dest_rect.map(Rect::to_ll);
        let dest_rect_ptr = dest_rect
            .as_ref()
            .map_or(core::ptr::null(), core::ptr::from_ref);

        let result = unsafe {
            sys::SDL_BlitSurface(
                self.raw() as *mut _,
                src_rect_ptr,
                dest.raw(),
                dest_rect_ptr,
            )
        };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Perform a scaled blit to a destination surface, which may be of a different format.
    pub fn blit_scaled(
        &mut self,
        src_rect: Option<Rect>,
        dest: &mut SurfaceRef,
        dest_rect: Option<Rect>,
        scale_mode: ScaleMode,
    ) -> Result<(), Error> {
        let src_rect = src_rect.map(Rect::to_ll);
        let src_rect_ptr = src_rect
            .as_ref()
            .map_or(core::ptr::null(), core::ptr::from_ref);

        let dest_rect = dest_rect.map(Rect::to_ll);
        let dest_rect_ptr = dest_rect
            .as_ref()
            .map_or(core::ptr::null(), core::ptr::from_ref);

        let result = unsafe {
            sys::SDL_BlitSurfaceScaled(
                self.raw(),
                src_rect_ptr,
                dest.raw(),
                dest_rect_ptr,
                scale_mode.to_ll(),
            )
        };

        if !result {
            return Err(Error);
        }

        Ok(())
    }

    /// Perform a scaled blit using the 9-grid algorithm to a destination surface, which may be
    /// of a different format.
    ///
    /// The pixels in the source surface are split into a 3x3 grid, using the different corner
    /// sizes for each corner, and the sides and center making up the remaining pixels. The corners
    /// are then scaled using `scale` and fit into the corners of the destination rectangle. The
    /// sides and center are then stretched into place to cover the remaining destination rectangle.
    pub fn blit_9_grid(
        &mut self,
        src_rect: Option<Rect>,
        left_width: u32,
        right_width: u32,
        top_height: u32,
        bottom_height: u32,
        scale: f32,
        scale_mode: ScaleMode,
        dest: &mut SurfaceRef,
        dest_rect: Option<Rect>,
    ) -> Result<(), Error> {
        let src_rect = src_rect.map(Rect::to_ll);
        let src_rect_ptr = src_rect
            .as_ref()
            .map_or(core::ptr::null(), core::ptr::from_ref);

        let dest_rect = dest_rect.map(Rect::to_ll);
        let dest_rect_ptr = dest_rect
            .as_ref()
            .map_or(core::ptr::null(), core::ptr::from_ref);

        let result = unsafe {
            sys::SDL_BlitSurface9Grid(
                self.raw(),
                src_rect_ptr,
                left_width.try_into()?,
                right_width.try_into()?,
                top_height.try_into()?,
                bottom_height.try_into()?,
                scale,
                scale_mode.to_ll(),
                dest.raw(),
                dest_rect_ptr,
            )
        };

        if !result {
            return Err(Error);
        }

        Ok(())
    }

    /// Perform a tiled blit to a destination surface, which may be of a different format.
    ///
    /// The pixels in `src_rect` will be repeated as many times as needed to completely fill `dest_rect`.
    pub fn blit_tiled(
        &self,
        src_rect: Option<Rect>,
        dest: &mut SurfaceRef,
        dest_rect: Option<Rect>,
    ) -> Result<(), Error> {
        let src_rect = src_rect.map(Rect::to_ll);
        let src_rect_ptr = src_rect
            .as_ref()
            .map_or(core::ptr::null(), core::ptr::from_ref);

        let dest_rect = dest_rect.map(Rect::to_ll);
        let dest_rect_ptr = dest_rect
            .as_ref()
            .map_or(core::ptr::null(), core::ptr::from_ref);

        let result = unsafe {
            sys::SDL_BlitSurfaceTiled(self.raw(), src_rect_ptr, dest.raw(), dest_rect_ptr)
        };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Perform a scaled and tiled blit to a destination surface, which may be of a different format.
    ///
    /// The pixels in `src_rect` will be scaled and repeated as many times as needed to completely
    /// fill `dest_Rect`.
    pub fn blit_tiled_with_scale(
        &self,
        src_rect: Option<Rect>,
        dest: &mut SurfaceRef,
        scale: f32,
        scale_mode: ScaleMode,
        dest_rect: Option<Rect>,
    ) -> Result<(), Error> {
        let src_rect = src_rect.map(Rect::to_ll);
        let src_rect_ptr = src_rect
            .as_ref()
            .map_or(core::ptr::null(), core::ptr::from_ref);

        let dest_rect = dest_rect.map(Rect::to_ll);
        let dest_rect_ptr = dest_rect
            .as_ref()
            .map_or(core::ptr::null(), core::ptr::from_ref);

        let result = unsafe {
            sys::SDL_BlitSurfaceTiledWithScale(
                self.raw(),
                src_rect_ptr,
                scale,
                scale_mode.to_ll(),
                dest.raw(),
                dest_rect_ptr,
            )
        };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Perform a fast fill of a rectangle with a specific color.
    ///
    /// `color` should be a pixel of the format used by the surface, and can be generated by
    /// [`crate::pixels::PixelFormatDetails::map_rgb`] or
    /// [`crate::pixels::PixelFormatDetails::map_rgba`].
    /// If the color value contains an\n alpha component then the destination is simply filled with that alpha
    /// information, no blending takes place.
    ///
    /// If there is a clip rectangle set on the destination (set via [`SurfaceRef::set_clip_rect`]), then this
    /// function will fill based on the intersection of the clip rectangle and `rect`.
    pub fn fill_rect(&mut self, rect: Option<Rect>, color: u32) -> Result<(), Error> {
        let rect = rect.map(Rect::to_ll);
        let rect_ptr = rect.as_ref().map_or(core::ptr::null(), core::ptr::from_ref);
        let result = unsafe { sys::SDL_FillSurfaceRect(self.raw(), rect_ptr, color) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Perform a fast fill of a set of rectangles with a specific color.
    ///
    /// `color` should be a pixel of the format used by the surface, and can be generated by
    /// [`PixelFormatDetails::map_rgb`] or [`PixelFormatDetails::map_rgba`]. If the color value contains
    /// an alpha component then the destination is simply filled with that alpha information,
    /// no blending takes place.
    ///
    /// If there is a clip rectangle set on the destination (set via [`Surface::set_clip_rect`],
    /// then this function will fill based on the intersection of the clip rectangle and `rect`.
    pub fn fill_rects(&mut self, rects: &[Rect], color: u32) -> Result<(), Error> {
        use core::ffi::c_int;
        let count = c_int::try_from(rects.len())?;
        let result = unsafe {
            sys::SDL_FillSurfaceRects(self.raw(), rects.as_ptr() as *const _, count, color)
        };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Flip a surface vertically or horizontally.
    pub fn flip(&mut self, mode: Option<FlipMode>) -> Result<(), Error> {
        let result = unsafe {
            sys::SDL_FlipSurface(
                self.raw(),
                mode.map(|f| f.to_ll())
                    .unwrap_or(sys::SDL_FlipMode_SDL_FLIP_NONE),
            )
        };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Clear a surface with a specific color, with floating point precision.
    ///
    /// This function handles all surface formats, and ignores any clip rectangle.
    ///
    /// If the surface is YUV, the color is assumed to be in the sRGB colorspace, otherwise the
    /// color is assumed to be in the colorspace of the suface.
    pub fn clear(&mut self, color: Color) -> Result<(), Error> {
        let color: ColorF32 = color.into();
        let result = unsafe {
            sys::SDL_ClearSurface(self.raw(), color.r(), color.g(), color.b(), color.a())
        };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Writes a single pixel to a surface.
    ///
    /// This function prioritizes correctness over speed: it is suitable for unit tests, but is
    /// not intended for use in a game engine.
    ///
    /// Like [`PixelFormatDetails::map_rgba`], this uses the entire 0..255 range when converting color
    /// components from pixel formats with less than 8 bits per RGB component.
    pub fn write_pixel(&mut self, x: u32, y: u32, color: Color) -> Result<(), Error> {
        let x = i32::try_from(x)?;
        let y = i32::try_from(y)?;
        let result = unsafe {
            sys::SDL_WriteSurfacePixel(self.raw(), x, y, color.r(), color.g(), color.b(), color.a())
        };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Writes a single pixel to a surface.
    ///
    /// This function prioritizes correctness over speed: it is suitable for unit tests, but is
    /// not intended for use in a game engine.
    pub fn write_pixel_float(&mut self, x: u32, y: u32, color: ColorF32) -> Result<(), Error> {
        let x = i32::try_from(x)?;
        let y = i32::try_from(y)?;
        let result = unsafe {
            sys::SDL_WriteSurfacePixelFloat(
                self.raw(),
                x,
                y,
                color.r(),
                color.g(),
                color.b(),
                color.a(),
            )
        };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Retrieves a single pixel from a surface.
    ///
    /// This function prioritizes correctness over speed: it is suitable for unit tests,
    /// but is not intended for use in a game engine.
    ///
    /// Like [`crate::pixels::PixelFormatDetails::rgba`], this uses the entire 0..255
    /// range when converting color components from pixel formats with less than 8 bits
    /// per RGB component.
    pub fn read_pixel(&self, x: u32, y: u32) -> Result<Color, Error> {
        let x = i32::try_from(x)?;
        let y = i32::try_from(y)?;
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        let mut a = 0;
        let result = unsafe {
            sys::SDL_ReadSurfacePixel(
                self.raw(),
                x,
                y,
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

    /// Retrieves a single pixel from a surface.
    ///
    /// This function prioritizes correctness over speed: it is suitable for unit tests,
    /// but is not intended for use in a game engine.
    ///
    /// Like [`crate::pixels::PixelFormatDetails::rgba`], this uses the entire 0..255
    /// range when converting color components from pixel formats with less than 8 bits
    /// per RGB component.
    pub fn read_pixel_float(&self, x: u32, y: u32) -> Result<ColorF32, Error> {
        let x = i32::try_from(x)?;
        let y = i32::try_from(y)?;
        let mut r = 0.0;
        let mut g = 0.0;
        let mut b = 0.0;
        let mut a = 0.0;
        let result = unsafe {
            sys::SDL_ReadSurfacePixelFloat(
                self.raw(),
                x,
                y,
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

    /// Set the palette used by a surface.
    ///
    /// A single palette can be shared with many surfaces.
    pub fn set_palette(&mut self, palette: &Palette) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetSurfacePalette(self.raw(), palette.raw()) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Returns the palette used by the surface.
    pub fn palette(&self) -> Option<Palette> {
        let result = unsafe { sys::SDL_GetSurfacePalette(self.raw()) };
        if result.is_null() {
            return None;
        }
        Some(unsafe { Palette::from_mut_ptr(result) })
    }

    /// Returns whether the surface has a color key.
    pub fn has_color_key(&self) -> bool {
        unsafe { sys::SDL_SurfaceHasColorKey(self.raw()) }
    }

    /// Returns whether the surface is RLE enabled.
    pub fn has_rle(&self) -> bool {
        unsafe { sys::SDL_SurfaceHasRLE(self.raw()) }
    }

    /// Returns whether the surface is RLE enabled.
    pub fn set_rle(&self, has_rle: bool) -> Result<(), Error> {
        let result = unsafe { sys::SDL_SetSurfaceRLE(self.raw(), has_rle) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Map an RGB triple to an opaque pixel value for a surface.
    ///
    /// This function maps the RGB color value to the specified pixel format and returns the
    /// pixel value best approximating the given RGB color value for\n the given pixel format.
    ///
    /// If the surface has a palette, the index of the closest matching color in the palette
    /// will be returned.
    ///
    /// If the surface pixel format has an alpha component it will be returned as all 1 bits
    /// (fully opaque).
    ///
    /// If the pixel format bpp (color depth) is less than 32-bpp then the unused upper bits
    /// of the return value can safely be ignored (e.g., with a 16-bpp format the return
    /// value can be assigned to a Uint16, and similarly a Uint8\n for an 8-bpp format).
    pub fn map_rgb(&self, rgb: (u8, u8, u8)) -> u32 {
        let (r, g, b) = rgb;
        unsafe { sys::SDL_MapSurfaceRGB(self.raw(), r, g, b) }
    }

    /// Map an RGBA quadruple to a pixel value for a surface.
    ///
    /// This function maps the RGBA color value to the specified pixel format and returns
    /// the pixel value best approximating the given RGBA color value for the given pixel
    /// format
    ///
    /// If the surface pixel format has no alpha component the alpha value will be ignored
    /// (as it will be in formats with a palette).
    ///
    /// If the surface has a palette, the index of the closest matching color in the palette
    /// will be returned.
    ///
    /// If the pixel format bpp (color depth) is less than 32-bpp then the unused upper
    /// bits of the return value can safely be ignored (e.g., with a 16-bpp format the
    /// return value can be assigned to a Uint16, and similarly a Uint8 for an 8-bpp format).
    pub fn map_rgba(&self, rgba: (u8, u8, u8, u8)) -> u32 {
        let (r, g, b, a) = rgba;
        unsafe { sys::SDL_MapSurfaceRGBA(self.raw(), r, g, b, a) }
    }

    /// Premultiply the alpha in a surface.
    ///
    /// This is safe to use with src == dst, but not for other overlapping areas.
    pub fn premultiply_alpha(&mut self, linear: bool) -> Result<(), Error> {
        let result = unsafe { sys::SDL_PremultiplySurfaceAlpha(self.raw(), linear) };
        if !result {
            return Err(Error);
        }
        Ok(())
    }

    /// Creates a `SurfaceLock`, which can be used to directly access a surface's pixels.
    ///
    /// This is equivalent to [`SurfaceLock::new`].
    pub fn lock<'a>(&'a mut self) -> Result<SurfaceLock<'a>, Error> {
        SurfaceLock::new(self)
    }

    /// The format of the surface.
    pub fn format(&self) -> PixelFormat {
        unsafe {
            let format = (*self.raw()).format;
            PixelFormat::from_ll_unchecked(format)
        }
    }

    #[inline]
    pub fn raw(&self) -> *mut sys::SDL_Surface {
        self as *const Self as *mut Self as *mut () as *mut sys::SDL_Surface
    }
}

/// Allows reading and writing a surface's pixels, using the surface's pixel format.
pub struct SurfaceLock<'a>(&'a mut SurfaceRef);

impl<'a> SurfaceLock<'a> {
    /// Creates a `SurfaceLock`.
    fn new(surface: &'a mut SurfaceRef) -> Result<Self, Error> {
        let result = unsafe { sys::SDL_LockSurface(surface.raw()) };
        if !result {
            return Err(Error);
        }
        Ok(Self(surface))
    }

    /// Returns a slice with the surface's underlying bytes.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let height = (*self.raw()).h;
            let pitch = (*self.raw()).pitch;
            let length = (height * pitch) as usize;
            let pixels = (*self.raw()).pixels;
            if pixels.is_null() {
                return &[];
            }
            core::slice::from_raw_parts(pixels as *const u8, length)
        }
    }

    /// Returns a mutable slice with the surface's underlying bytes.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            let height = (*self.raw()).h;
            let pitch = (*self.raw()).pitch;
            let length = (height * pitch) as usize;
            let pixels = (*self.raw()).pixels;
            if pixels.is_null() {
                return &mut [];
            }
            core::slice::from_raw_parts_mut(pixels as *mut u8, length)
        }
    }
}

impl Deref for SurfaceLock<'_> {
    type Target = SurfaceRef;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl DerefMut for SurfaceLock<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<'a> Drop for SurfaceLock<'a> {
    fn drop(&mut self) {
        unsafe { sys::SDL_UnlockSurface(self.raw()) };
    }
}

/// The scaling mode.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u32)]
pub enum ScaleMode {
    Nearest = sys::SDL_ScaleMode_SDL_SCALEMODE_LINEAR,
    Linear = sys::SDL_ScaleMode_SDL_SCALEMODE_NEAREST,
}

impl ScaleMode {
    /// Converts a raw `SDL_ScaleMode` into a `ScaleMode`.
    pub fn try_from_ll(value: sys::SDL_ScaleMode) -> Result<Self, Error> {
        Ok(match value {
            sys::SDL_ScaleMode_SDL_SCALEMODE_NEAREST => Self::Nearest,
            sys::SDL_ScaleMode_SDL_SCALEMODE_LINEAR => Self::Linear,
            _ => return Err(Error::register(c"Unknown scale mode.")),
        })
    }

    /// Converts a raw `ScaleMode` into a raw `sys::SDL_ScaleMode`.
    pub fn to_ll(&self) -> sys::SDL_ScaleMode {
        *self as u32
    }
}

/// The flip mode.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FlipMode {
    Horizontal = sys::SDL_FlipMode_SDL_FLIP_HORIZONTAL,
    Vertical = sys::SDL_FlipMode_SDL_FLIP_VERTICAL,
}

impl FlipMode {
    /// Converts a raw `SDL_FlipMode` into a `FlipMode`.
    ///
    /// If the `SDL_FlipMode` is `SDL_FLIP_NONE`, this function will return `None`.
    pub fn from_ll(value: sys::SDL_FlipMode) -> Option<Self> {
        match value {
            sys::SDL_FlipMode_SDL_FLIP_VERTICAL => Some(Self::Vertical),
            sys::SDL_FlipMode_SDL_FLIP_HORIZONTAL => Some(Self::Horizontal),
            _ => None,
        }
    }

    /// Converts a `FlipMode` into a raw `sys::SDL_FlipMode`.
    #[inline]
    pub fn to_ll(&self) -> sys::SDL_FlipMode {
        *self as u32
    }
}
