use crate::init::VideoSubsystem;
use crate::pixels::{Color, ColorF32, PixelFormat};
use crate::rect::Rect;
use crate::{sys, Error};
use alloc::sync::Arc;
use core::marker::PhantomData;
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
            _video: VideoSubsystem(Arc::clone(&video.0)),
            inner: Surface(ptr),
        })
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
pub struct Surface(*mut sys::surface::SDL_Surface);

impl Surface {
    pub(crate) unsafe fn new(ptr: *mut sys::surface::SDL_Surface) -> Self {
        Self(ptr)
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
            return Err(Error::from_sdl())
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

    pub fn format(&self) -> PixelFormat {
        let format = unsafe { (*self.0).format };
        PixelFormat::from_ll(format)
    }

    pub fn as_ptr(&self) -> *const sys::surface::SDL_Surface {
        self.0 as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut sys::surface::SDL_Surface {
        self.0
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ScaleMode(sys::surface::SDL_ScaleMode);

impl ScaleMode {
    pub const NEAREST: Self = Self(sys::surface::SDL_ScaleMode::NEAREST);
    pub const LINEAR: Self = Self(sys::surface::SDL_ScaleMode::LINEAR);

    pub fn to_ll(&self) -> sys::surface::SDL_ScaleMode {
        self.0
    }
}
