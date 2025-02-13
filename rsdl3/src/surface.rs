use crate::init::VideoSubsystem;
use crate::pixels::{Color, ColorF32, PixelFormat};
use crate::rect::Rect;
use crate::{sys, Error};
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};

pub struct SurfaceOwned {
    _video: VideoSubsystem,
    inner: Surface,
}

impl SurfaceOwned {
    pub(crate) fn new(
        video: &VideoSubsystem,
        w: u32,
        h: u32,
        format: PixelFormat,
    ) -> Result<Self, Error> {
        let w = w.clamp(0, i32::MAX as u32) as i32;
        let h = h.clamp(0, i32::MAX as u32) as i32;
        let ptr = unsafe { sys::surface::SDL_CreateSurface(w, h, format.to_ll()) };
        if ptr.is_null() {
            return Err(Error::from_sdl());
        }
        Ok(Self {
            _video: video.clone(),
            inner: Surface(ptr),
        })
    }

    /// SAFETY: ptr must be valid
    pub(crate) unsafe fn from_ptr(
        video: &VideoSubsystem,
        ptr: *mut sys::surface::SDL_Surface,
    ) -> Self {
        Self {
            _video: video.clone(),
            inner: Surface::new(ptr),
        }
    }

    pub fn convert(self, format: PixelFormat) -> Result<SurfaceOwned, Error> {
        let ptr = unsafe { sys::surface::SDL_ConvertSurface(self.0, format.to_ll()) };
        if ptr.is_null() {
            return Err(Error::from_sdl());
        }
        Ok(unsafe { SurfaceOwned::from_ptr(&self._video, ptr) })
    }

    #[inline]
    pub fn as_ref(&self) -> &Surface {
        &self.inner
    }

    #[inline]
    pub fn as_mut(&mut self) -> &mut Surface {
        &mut self.inner
    }
}

impl Deref for SurfaceOwned {
    type Target = Surface;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SurfaceOwned {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Drop for SurfaceOwned {
    fn drop(&mut self) {
        unsafe { sys::surface::SDL_DestroySurface(self.0) };
    }
}

// SAFETY:
// We only ever hand out this struct via derefs. This object can't be constructed outside of this
// module; so it's always exposed as a reference whose lifetime matches that of its' owner.
// Attempts to reassining the value of a &mut Surface won't work because Surfaces can't be moved
// out of.
pub struct Surface(*mut sys::surface::SDL_Surface);

impl Surface {
    pub(crate) unsafe fn new(ptr: *mut sys::surface::SDL_Surface) -> Self {
        Self(ptr)
    }

    pub fn add_alternate_image(&mut self, other: &mut Surface) -> Result<(), Error> {
        let result = unsafe { sys::surface::SDL_AddSurfaceAlternateImage(self.0, other.0) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn alpha_mod(&self) -> Result<u8, Error> {
        let mut alpha_mod: u8 = 0;
        let result = unsafe { sys::surface::SDL_GetSurfaceAlphaMod(self.0, &raw mut alpha_mod) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(alpha_mod)
    }

    pub fn set_alpha_mod(&mut self, alpha_mod: u8) -> Result<(), Error> {
        let result = unsafe { sys::surface::SDL_SetSurfaceAlphaMod(self.0, alpha_mod) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn blend_mode(&self) -> Result<Option<BlendMode>, Error> {
        let mut blend_mode = 0;
        let result = unsafe { sys::surface::SDL_GetSurfaceBlendMode(self.0, &raw mut blend_mode) };
        if !result {
            return Err(Error::from_sdl());
        }
        BlendMode::try_from_ll(blend_mode)
    }

    pub fn set_blend_mode(&mut self, blend_mode: BlendMode) -> Result<(), Error> {
        let result = unsafe { sys::surface::SDL_SetSurfaceBlendMode(self.0, blend_mode.to_ll()) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn clip_rect(&self) -> Result<Rect, Error> {
        let mut rect = MaybeUninit::uninit();
        let rect = unsafe {
            let result = sys::surface::SDL_GetSurfaceClipRect(self.0, rect.as_mut_ptr());
            if !result {
                return Err(Error::from_sdl());
            }
            rect.assume_init()
        };
        Ok(Rect::from_ll(rect))
    }

    pub fn set_clip_rect(&mut self, rect: Option<Rect>) -> Result<(), Error> {
        let clip_rect = rect.map(Rect::to_ll);
        let clip_rect_ptr = clip_rect
            .as_ref()
            .map_or(core::ptr::null(), core::ptr::from_ref);
        let result = unsafe { sys::surface::SDL_SetSurfaceClipRect(self.0, clip_rect_ptr) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn color_key(&self) -> Result<u32, Error> {
        let mut color_key = 0;
        let result = unsafe { sys::surface::SDL_GetSurfaceColorKey(self.0, &raw mut color_key) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(color_key)
    }

    pub fn set_color_key(&mut self, color_key: Option<u32>) -> Result<(), Error> {
        let result = match color_key {
            Some(color_key) => unsafe {
                sys::surface::SDL_SetSurfaceColorKey(self.0, true, color_key)
            },
            None => unsafe { sys::surface::SDL_SetSurfaceColorKey(self.0, false, 0) },
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn color_mod(&self) -> Result<(u8, u8, u8), Error> {
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        let result = unsafe {
            sys::surface::SDL_GetSurfaceColorMod(self.0, &raw mut r, &raw mut g, &raw mut b)
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok((r, g, b))
    }

    pub fn set_color_mod(&mut self, r: u8, g: u8, b: u8) -> Result<(), Error> {
        let result = unsafe { sys::surface::SDL_SetSurfaceColorMod(self.0, r, g, b) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    /// This function takes a mutable reference to the [`Surface`] to mimic the parameters of
    /// [`sys::surface::SDL_BlitSurface`]. It doesn't actually mutate the surface's contents.
    pub fn blit(
        &mut self,
        src_rect: Option<Rect>,
        dest: &mut Surface,
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

        let result =
            unsafe { sys::surface::SDL_BlitSurface(self.0, src_rect_ptr, dest.0, dest_rect_ptr) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    /// This function takes a mutable reference to the [`Surface`] to mimic the parameters of
    /// [`sys::surface::SDL_BlitSurface`]. It doesn't actually mutate the surface's contents.
    pub fn blit_scaled(
        &mut self,
        src_rect: Option<Rect>,
        dest: &mut Surface,
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
            sys::surface::SDL_BlitSurfaceScaled(
                self.0,
                src_rect_ptr,
                dest.0,
                dest_rect_ptr,
                scale_mode.to_ll(),
            )
        };

        if !result {
            return Err(Error::from_sdl());
        }

        Ok(())
    }

    pub fn blit_9_grid(
        &mut self,
        src_rect: Option<Rect>,
        left_width: u32,
        right_width: u32,
        top_height: u32,
        bottom_height: u32,
        scale: f32,
        scale_mode: ScaleMode,
        dest: &mut Surface,
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
            sys::surface::SDL_BlitSurface9Grid(
                self.0,
                src_rect_ptr,
                left_width.try_into()?,
                right_width.try_into()?,
                top_height.try_into()?,
                bottom_height.try_into()?,
                scale,
                scale_mode.to_ll(),
                dest.0,
                dest_rect_ptr,
            )
        };

        if !result {
            return Err(Error::from_sdl());
        }

        Ok(())
    }

    pub fn blit_tiled(
        &mut self,
        src_rect: Option<Rect>,
        dest: &mut Surface,
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
            sys::surface::SDL_BlitSurfaceTiled(self.0, src_rect_ptr, dest.0, dest_rect_ptr)
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn blit_tiled_with_scale(
        &mut self,
        src_rect: Option<Rect>,
        dest: &mut Surface,
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
            sys::surface::SDL_BlitSurfaceTiledWithScale(
                self.0,
                src_rect_ptr,
                scale,
                scale_mode.to_ll(),
                dest.0,
                dest_rect_ptr,
            )
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn flip(&mut self, mode: Option<FlipMode>) -> Result<(), Error> {
        let result = unsafe {
            sys::surface::SDL_FlipSurface(
                self.0,
                mode.map(|f| f.to_ll())
                    .unwrap_or(sys::surface::SDL_FlipMode::NONE),
            )
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn clear(&mut self, color: Color) -> Result<(), Error> {
        let color: ColorF32 = color.into();
        let result = unsafe {
            sys::surface::SDL_ClearSurface(self.0, color.r(), color.g(), color.b(), color.a())
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn lock<'a>(&'a mut self) -> Result<SurfaceLock<'a>, Error> {
        let result = unsafe { sys::surface::SDL_LockSurface(self.0) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(SurfaceLock(self))
    }

    pub fn format(&self) -> PixelFormat {
        unsafe {
            let format = (*self.0).format;
            PixelFormat::from_ll_unchecked(format)
        }
    }

    pub fn as_ptr(&self) -> *const sys::surface::SDL_Surface {
        self.0 as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut sys::surface::SDL_Surface {
        self.0
    }
}

pub struct SurfaceLock<'a>(&'a mut Surface);

impl<'a> SurfaceLock<'a> {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let ptr = (*self.0 .0).pixels;
            match self.pixels_length(self.0 .0) {
                Some(length) => core::slice::from_raw_parts(ptr as *const u8, length),
                None => &[],
            }
        }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            let ptr = (*self.0 .0).pixels;
            match self.pixels_length(self.0 .0) {
                Some(length) => core::slice::from_raw_parts_mut(ptr as *mut u8, length),
                None => &mut [],
            }
        }
    }

    unsafe fn pixels_length(&self, ptr: *const sys::surface::SDL_Surface) -> Option<usize> {
        if (*ptr).pixels.is_null() {
            return None;
        }
        Some(((*ptr).h * (*ptr).pitch) as usize)
    }
}

impl Deref for SurfaceLock<'_> {
    type Target = Surface;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Drop for SurfaceLock<'a> {
    fn drop(&mut self) {
        unsafe { sys::surface::SDL_UnlockSurface(self.0 .0) };
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(i32)]
pub enum ScaleMode {
    Nearest = sys::surface::SDL_ScaleMode::LINEAR.0,
    Linear = sys::surface::SDL_ScaleMode::NEAREST.0,
}

impl ScaleMode {
    pub fn try_from_ll(value: sys::surface::SDL_ScaleMode) -> Result<Self, Error> {
        Ok(match value {
            sys::surface::SDL_ScaleMode::NEAREST => Self::Nearest,
            sys::surface::SDL_ScaleMode::LINEAR => Self::Linear,
            _ => return Err(Error::new("Invalid SDL scale mode.")),
        })
    }

    pub fn to_ll(&self) -> sys::surface::SDL_ScaleMode {
        sys::surface::SDL_ScaleMode(*self as i32)
    }
}

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FlipMode {
    Horizontal = sys::surface::SDL_FlipMode::HORIZONTAL.0,
    Vertical = sys::surface::SDL_FlipMode::VERTICAL.0,
}

impl FlipMode {
    pub fn from_ll(value: sys::surface::SDL_FlipMode) -> Option<Self> {
        match value {
            sys::surface::SDL_FlipMode::VERTICAL => Some(Self::Vertical),
            sys::surface::SDL_FlipMode::HORIZONTAL => Some(Self::Horizontal),
            _ => None,
        }
    }

    #[inline]
    pub fn to_ll(&self) -> sys::surface::SDL_FlipMode {
        sys::surface::SDL_FlipMode(*self as i32)
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlendMode {
    Blend = sys::blendmode::SDL_BLENDMODE_BLEND,
    BlendPremultiplied = sys::blendmode::SDL_BLENDMODE_BLEND_PREMULTIPLIED,
    Add = sys::blendmode::SDL_BLENDMODE_ADD,
    AddPremultipled = sys::blendmode::SDL_BLENDMODE_ADD_PREMULTIPLIED,
    Mod = sys::blendmode::SDL_BLENDMODE_MOD,
    Mul = sys::blendmode::SDL_BLENDMODE_MUL,
    Invalid = sys::blendmode::SDL_BLENDMODE_INVALID,
}

impl BlendMode {
    pub fn try_from_ll(value: u32) -> Result<Option<Self>, Error> {
        match value {
            sys::blendmode::SDL_BLENDMODE_BLEND => Ok(Some(Self::Blend)),
            sys::blendmode::SDL_BLENDMODE_BLEND_PREMULTIPLIED => Ok(Some(Self::BlendPremultiplied)),
            sys::blendmode::SDL_BLENDMODE_ADD => Ok(Some(Self::Add)),
            sys::blendmode::SDL_BLENDMODE_ADD_PREMULTIPLIED => Ok(Some(Self::AddPremultipled)),
            sys::blendmode::SDL_BLENDMODE_MOD => Ok(Some(Self::Mod)),
            sys::blendmode::SDL_BLENDMODE_MUL => Ok(Some(Self::Mul)),
            sys::blendmode::SDL_BLENDMODE_INVALID => Ok(Some(Self::Invalid)),
            sys::blendmode::SDL_BLENDMODE_NONE => Ok(None),
            _ => Err(Error::new("Unknown blend mode.")),
        }
    }

    #[inline]
    pub fn to_ll(&self) -> u32 {
        *self as u32
    }
}
