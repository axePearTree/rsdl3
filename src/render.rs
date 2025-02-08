use crate::pixels::PixelFormat;
use crate::rect::RectF32;
use crate::surface::{Surface, SurfaceOwned};
use crate::video::Window;
use crate::{sys, Error};
use alloc::ffi::CString;
use alloc::rc::{Rc, Weak};
use alloc::string::String;
use core::ops::{Deref, DerefMut};

// The order of fields must be preserved so the drop order is correct.
// Also, the window must not be exposed mutably while the Renderer exists.
pub struct WindowRenderer {
    renderer: Renderer,
    window: Window,
}

impl WindowRenderer {
    pub(crate) fn new(window: Window, driver: Option<&str>) -> Result<Self, Error> {
        unsafe {
            let driver = match driver {
                Some(driver) => Some(CString::new(driver)?),
                None => None,
            };
            let driver = driver.map(|s| s.as_ptr()).unwrap_or(core::ptr::null());
            let ptr = sys::render::SDL_CreateRenderer(window.as_mut_ptr(), driver);
            if ptr.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(Self {
                renderer: Renderer {
                    ptr: Rc::new(ptr),
                    target: None,
                },
                window,
            })
        }
    }

    pub fn as_window_ref(&self) -> &Window {
        &self.window
    }

    pub fn into_window(self) -> Window {
        self.window
    }
}

impl Deref for WindowRenderer {
    type Target = Renderer;

    fn deref(&self) -> &Self::Target {
        &self.renderer
    }
}

impl DerefMut for WindowRenderer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.renderer
    }
}

// The order of fields must be preserved so the drop order is correct.
pub struct SoftwareRenderer {
    renderer: Renderer,
    surface: SurfaceOwned,
}

impl SoftwareRenderer {
    pub fn new(mut surface: SurfaceOwned) -> Result<Self, Error> {
        unsafe {
            let ptr = sys::render::SDL_CreateSoftwareRenderer(surface.as_mut_ptr());
            if ptr.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(Self {
                renderer: Renderer {
                    ptr: Rc::new(ptr),
                    target: None,
                },
                surface,
            })
        }
    }

    pub fn as_surface_ref(&self) -> &Surface {
        &self.surface
    }

    pub fn into_surface(self) -> SurfaceOwned {
        self.surface
    }
}

impl Deref for SoftwareRenderer {
    type Target = Renderer;

    fn deref(&self) -> &Self::Target {
        &self.renderer
    }
}

impl DerefMut for SoftwareRenderer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.renderer
    }
}

// SAFETY:
// This struct should only be exposed as a reference bound to its' owner (either a WindowRenderer
// or a SoftareRenderer) so the lifetime is sound.
// The Rc MUST NOT BE CLONED AND HANDED OUT.
// We use it to hand out Weak references to Textures so we can check whether or not
// the parent renderer is alive, at runtime.
pub struct Renderer {
    ptr: Rc<*mut sys::render::SDL_Renderer>,
    target: Option<Texture>,
}

impl Renderer {
    pub fn create_texture(
        &mut self,
        format: PixelFormat,
        access: TextureAccess,
        width: u32,
        height: u32,
    ) -> Result<Texture, Error> {
        let format = format.to_ll();
        let access = access.to_ll();
        let ptr = unsafe {
            sys::render::SDL_CreateTexture(
                *self.ptr,
                format,
                access,
                width.try_into()?,
                height.try_into()?,
            )
        };
        if ptr.is_null() {
            return Err(Error::from_sdl());
        }
        Ok(Texture {
            parent: Rc::downgrade(&self.ptr),
            ptr,
        })
    }

    pub fn create_texture_from_surface(&mut self, surface: &mut Surface) -> Result<Texture, Error> {
        let ptr =
            unsafe { sys::render::SDL_CreateTextureFromSurface(*self.ptr, surface.as_mut_ptr()) };
        if ptr.is_null() {
            return Err(Error::from_sdl());
        }
        Ok(Texture {
            parent: Rc::downgrade(&self.ptr),
            ptr,
        })
    }

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

        let result = unsafe {
            sys::render::SDL_RenderTexture(*self.ptr, texture.ptr, src_rect_ptr, dest_rect_ptr)
        };

        if !result {
            return Err(Error::from_sdl());
        }

        Ok(())
    }

    pub fn set_render_target(&mut self, texture: Texture) -> Result<(), Error> {
        self.validate_texture(&texture)?;

        let result = unsafe { sys::render::SDL_SetRenderTarget(*self.ptr, texture.ptr) };
        self.target = Some(texture);

        if !result {
            return Err(Error::from_sdl());
        }

        Ok(())
    }

    pub fn take_render_target(&mut self) -> Result<Option<Texture>, Error> {
        let result = unsafe { sys::render::SDL_SetRenderTarget(*self.ptr, core::ptr::null_mut()) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(self.target.take())
    }

    pub fn present(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::render::SDL_RenderPresent(*self.ptr) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    fn validate_texture(&self, texture: &Texture) -> Result<(), Error> {
        match texture.parent.upgrade() {
            Some(ref parent) if Rc::ptr_eq(parent, &self.ptr) => Ok(()),
            None => {
                return Err(Error::new("Texture's renderer has already been destroyed."));
            }
            _ => {
                return Err(Error::new("Texture does not belong to this renderer."));
            }
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let ptr = *self.ptr;
        unsafe { sys::render::SDL_DestroyRenderer(ptr) }
    }
}

pub struct Texture {
    parent: Weak<*mut sys::render::SDL_Renderer>,
    /// SAFETY: this pointer can only be used if the parent Weak pointer is upgradeable.
    ptr: *mut sys::render::SDL_Texture,
}

impl Drop for Texture {
    fn drop(&mut self) {
        // SAFETY:
        if let Some(_) = self.parent.upgrade() {
            unsafe { sys::render::SDL_DestroyTexture(self.ptr) };
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TextureAccess(sys::render::SDL_TextureAccess);

impl TextureAccess {
    /// Changes rarely, not lockable
    pub const STATIC: Self = Self(sys::render::SDL_TextureAccess::STATIC);
    /// Changes frequently, lockable
    pub const STREAMING: Self = Self(sys::render::SDL_TextureAccess::STREAMING);
    /// Texture can be used as a render target
    pub const TARGET: Self = Self(sys::render::SDL_TextureAccess::TARGET);

    pub fn to_ll(self) -> sys::render::SDL_TextureAccess {
        self.0
    }
}
