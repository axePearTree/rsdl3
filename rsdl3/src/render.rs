use crate::pixels::{Color, PixelFormat};
use crate::rect::RectF32;
use crate::surface::{Surface, SurfaceRef};
use crate::video::{Window, WindowRef};
use crate::{sys, Error};
use alloc::ffi::CString;
use alloc::rc::{Rc, Weak};

/// A structure representing rendering state.
pub struct Renderer {
    context: RendererContext,
    target: Option<Texture>,
    /// This ptr is used with the sole purpose of handing out Weak references.
    ptr: Rc<*mut sys::SDL_Renderer>,
}

enum RendererContext {
    Window(Window),
    Software(Surface),
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe { sys::SDL_DestroyRenderer(*self.ptr) };
    }
}

impl Renderer {
    /// Creates a `Renderer` from an existing `Window` using the specified `driver`.
    ///
    /// The `driver` name can be obtained by calling [`VideoSubsystem::render_driver`] using the driver's index.
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
                return Err(Error::from_sdl());
            }
            Ok(Self {
                context: RendererContext::Window(window),
                ptr: Rc::new(ptr),
                target: None,
            })
        }
    }

    /// Creates a software `Renderer` from an existing `Surface`.
    ///
    /// The surface can later be borrowed by calling `Renderer::as_surface_ref` or `Renderer::as_surface_mut`.
    pub fn from_surface(mut surface: Surface) -> Result<Self, Error> {
        unsafe {
            let ptr = sys::SDL_CreateSoftwareRenderer(surface.as_mut_ptr());
            if ptr.is_null() {
                return Err(Error::from_sdl());
            }
            Ok(Self {
                context: RendererContext::Software(surface),
                ptr: Rc::new(ptr),
                target: None,
            })
        }
    }

    pub fn as_window_ref(&self) -> Option<&WindowRef> {
        match &self.context {
            RendererContext::Window(window) => Some(window),
            RendererContext::Software(_) => None,
        }
    }

    pub fn as_window_mut(&mut self) -> Option<&mut WindowRef> {
        match &mut self.context {
            RendererContext::Window(window) => Some(window),
            RendererContext::Software(_) => None,
        }
    }

    pub fn as_surface_ref(&self) -> Option<&SurfaceRef> {
        match &self.context {
            RendererContext::Software(surface) => Some(&*surface),
            RendererContext::Window(_) => None,
        }
    }

    pub fn as_surface_mut(&mut self) -> Option<&mut SurfaceRef> {
        match &mut self.context {
            RendererContext::Software(surface) => Some(&mut *surface),
            RendererContext::Window(_) => None,
        }
    }

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
            sys::SDL_CreateTexture(
                self.as_mut_ptr(),
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
            renderer: Rc::downgrade(&self.ptr),
            ptr,
        })
    }

    pub fn create_texture_from_surface(&mut self, surface: &SurfaceRef) -> Result<Texture, Error> {
        let ptr = unsafe {
            sys::SDL_CreateTextureFromSurface(self.as_mut_ptr(), surface.as_ptr() as *mut _)
        };
        if ptr.is_null() {
            return Err(Error::from_sdl());
        }
        Ok(Texture {
            renderer: Rc::downgrade(&self.ptr),
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
            sys::SDL_RenderTexture(self.as_mut_ptr(), texture.ptr, src_rect_ptr, dest_rect_ptr)
        };

        if !result {
            return Err(Error::from_sdl());
        }

        Ok(())
    }

    pub fn render_draw_color(&self) -> Result<Color, Error> {
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        let mut a = 0;
        let result = unsafe {
            sys::SDL_GetRenderDrawColor(
                self.as_ptr() as *mut _,
                &raw mut r,
                &raw mut g,
                &raw mut b,
                &raw mut a,
            )
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(Color::new(r, g, b, a))
    }

    pub fn set_draw_color(&mut self, color: Color) -> Result<(), Error> {
        let result = unsafe {
            sys::SDL_SetRenderDrawColor(
                self.as_mut_ptr(),
                color.r(),
                color.g(),
                color.b(),
                color.a(),
            )
        };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    /// Returns the previously used texture if there was one.
    pub fn set_render_target(
        &mut self,
        texture: Option<Texture>,
    ) -> Result<Option<Texture>, Error> {
        match texture {
            Some(texture) => {
                self.validate_texture(&texture)?;
                let result = unsafe { sys::SDL_SetRenderTarget(self.as_mut_ptr(), texture.ptr) };
                if !result {
                    return Err(Error::from_sdl());
                }
                Ok(self.target.replace(texture))
            }
            _ => {
                let result =
                    unsafe { sys::SDL_SetRenderTarget(self.as_mut_ptr(), core::ptr::null_mut()) };
                if !result {
                    return Err(Error::from_sdl());
                }
                Ok(self.target.take())
            }
        }
    }

    pub fn present(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_RenderPresent(self.as_mut_ptr()) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::SDL_RenderClear(self.as_mut_ptr()) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn as_ptr(&self) -> *const sys::SDL_Renderer {
        *self.ptr
    }

    pub fn as_mut_ptr(&mut self) -> *mut sys::SDL_Renderer {
        *self.ptr
    }

    fn validate_texture(&self, texture: &Texture) -> Result<(), Error> {
        if texture.renderer.weak_count() == 0 {
            return Err(Error::RendererAlreadyDestroyed);
        }
        if !Weak::ptr_eq(&texture.renderer, &Rc::downgrade(&self.ptr)) {
            return Err(Error::TextureFromDifferentRenderer);
        }
        Ok(())
    }
}

pub struct Texture {
    /// Tells us whether or not the backing Renderer is alive via Weak::strong_count.
    /// This must *never* be upgraded to an Rc.
    renderer: Weak<*mut sys::SDL_Renderer>,
    ptr: *mut sys::SDL_Texture,
}

impl Texture {
    pub fn from_surface(renderer: &mut Renderer, surface: &SurfaceRef) -> Result<Self, Error> {
        renderer.create_texture_from_surface(surface)
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        if self.renderer.strong_count() > 0 {
            unsafe { sys::SDL_DestroyTexture(self.ptr) };
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TextureAccess(sys::SDL_TextureAccess);

impl TextureAccess {
    /// Changes rarely, not lockable
    pub const STATIC: Self = Self(sys::SDL_TextureAccess_SDL_TEXTUREACCESS_STATIC);
    /// Changes frequently, lockable
    pub const STREAMING: Self = Self(sys::SDL_TextureAccess_SDL_TEXTUREACCESS_STREAMING);
    /// Texture can be used as a render target
    pub const TARGET: Self = Self(sys::SDL_TextureAccess_SDL_TEXTUREACCESS_TARGET);

    pub fn to_ll(self) -> sys::SDL_TextureAccess {
        self.0
    }
}
