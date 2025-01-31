use crate::{init::VideoSubsystem, pixels::{Color, ColorF32}, sys, Error};
use core::{marker::PhantomData, ops::{Deref, DerefMut}};

pub struct OwnedSurface {
    _video: VideoSubsystem,
    inner: Surface,
}

impl Deref for OwnedSurface {
    type Target = Surface;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for OwnedSurface {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct SurfaceRef<'a> {
    inner: Surface,
    _m: PhantomData<&'a *const ()>,
}

impl<'a> SurfaceRef<'a> {
    /// SAFETY:
    /// Gotta make sure the lifetime of this object matches that of its' owner.
    pub(crate) unsafe fn from_mut_ptr(ptr: *mut sys::surface::SDL_Surface) -> Self {
        Self {
            inner: Surface(ptr),
            _m: PhantomData,
        }
    }
}

impl<'a> Deref for SurfaceRef<'a> {
    type Target = Surface;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct SurfaceMut<'a> {
    inner: Surface,
    _m: PhantomData<&'a *const ()>,
}

impl<'a> SurfaceMut<'a> {
    /// SAFETY:
    /// Gotta make sure the lifetime of this object matches that of its' owner.
    pub(crate) unsafe fn from_mut_ptr(ptr: *mut sys::surface::SDL_Surface) -> Self {
        Self {
            inner: Surface(ptr),
            _m: PhantomData,
        }
    }
}

impl<'a> Deref for SurfaceMut<'a> {
    type Target = Surface;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> DerefMut for SurfaceMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// SAFETY:
// We only ever hand out this struct via derefs. This object can't be constructed outside of this
// module; so it's always exposed as a reference whose lifetime matches that of its' owner.
pub struct Surface(*mut sys::surface::SDL_Surface);

impl Surface {
    pub fn clear(&mut self, color: Color) -> Result<(), Error> {
        let color: ColorF32 = color.into();
        let result = unsafe {
            // sys::surface::SDL_ClearSurface(self.ptr, color.r(), color.g(), color.b(), color.a())
            sys::surface::SDL_ClearSurface(self.0, color.r(), color.g(), color.b(), color.a())
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }
}
