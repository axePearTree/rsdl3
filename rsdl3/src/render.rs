use crate::pixels::{Color, PixelFormat};
use crate::rect::RectF32;
use crate::surface::{Surface, SurfaceRef};
use crate::video::{Window, WindowRef};
use crate::{sys, Error};
use alloc::ffi::CString;
use alloc::rc::{Rc, Weak};
use core::cell::RefCell;

pub struct Renderer(Rc<RendererInner>);

impl Renderer {
    pub fn try_from_window(mut window: Window, driver: Option<&str>) -> Result<Self, Error> {
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
            let inner = RendererInner {
                context: RendererContext::Window(window),
                ptr,
                target: RefCell::new(None),
            };
            Ok(Self(Rc::new(inner)))
        }
    }

    pub fn try_from_surface(mut surface: Surface) -> Result<Self, Error> {
        unsafe {
            let ptr = sys::render::SDL_CreateSoftwareRenderer(surface.as_mut_ptr());
            if ptr.is_null() {
                return Err(Error::from_sdl());
            }
            let inner = RendererInner {
                context: RendererContext::Software(surface),
                ptr,
                target: RefCell::new(None),
            };
            Ok(Self(Rc::new(inner)))
        }
    }

    pub fn as_window_ref(&self) -> Option<&WindowRef> {
        match &self.0.context {
            RendererContext::Window(window) => Some(window),
            RendererContext::Software(_) => None,
        }
    }

    pub fn as_window_mut(&mut self) -> Option<&mut WindowRef> {
        let inner = Rc::get_mut(&mut self.0)?;
        match &mut inner.context {
            RendererContext::Window(window) => Some(window),
            RendererContext::Software(_) => None,
        }
    }

    pub fn as_surface_ref(&self) -> Option<&SurfaceRef> {
        match &self.0.context {
            RendererContext::Software(surface) => Some(&*surface),
            RendererContext::Window(_) => None,
        }
    }

    pub fn as_surface_mut(&mut self) -> Option<&mut SurfaceRef> {
        let inner = Rc::get_mut(&mut self.0)?;
        match &mut inner.context {
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
            sys::render::SDL_CreateTexture(
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
            inner: Rc::downgrade(&self.0),
            ptr,
        })
    }

    pub fn create_texture_from_surface(
        &mut self,
        surface: &mut SurfaceRef,
    ) -> Result<Texture, Error> {
        let ptr = unsafe {
            sys::render::SDL_CreateTextureFromSurface(self.as_mut_ptr(), surface.as_mut_ptr())
        };
        if ptr.is_null() {
            return Err(Error::from_sdl());
        }
        Ok(Texture {
            inner: Rc::downgrade(&self.0),
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
            sys::render::SDL_RenderTexture(
                self.as_mut_ptr(),
                texture.ptr,
                src_rect_ptr,
                dest_rect_ptr,
            )
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
            sys::render::SDL_GetRenderDrawColor(
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

    pub fn set_render_draw_color(&mut self, color: Color) -> Result<(), Error> {
        let result = unsafe {
            sys::render::SDL_SetRenderDrawColor(
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
                let result =
                    unsafe { sys::render::SDL_SetRenderTarget(self.as_mut_ptr(), texture.ptr) };
                if !result {
                    return Err(Error::from_sdl());
                }
                Ok(self.0.target.borrow_mut().replace(texture))
            }
            _ => {
                let result = unsafe {
                    sys::render::SDL_SetRenderTarget(self.as_mut_ptr(), core::ptr::null_mut())
                };
                if !result {
                    return Err(Error::from_sdl());
                }
                Ok(self.0.target.borrow_mut().take())
            }
        }
    }

    pub fn present(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::render::SDL_RenderPresent(self.as_mut_ptr()) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::render::SDL_RenderClear(self.as_mut_ptr()) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(())
    }

    pub fn as_ptr(&self) -> *const sys::render::SDL_Renderer {
        self.0.ptr
    }

    pub fn as_mut_ptr(&mut self) -> *mut sys::render::SDL_Renderer {
        self.0.ptr
    }

    fn validate_texture(&self, texture: &Texture) -> Result<(), Error> {
        if texture.inner.weak_count() == 0 {
            return Err(Error::new("Texture's renderer has already been destroyed."));
        }
        if !Weak::ptr_eq(&texture.inner, &Rc::downgrade(&self.0)) {
            return Err(Error::new("Texture does not belong to this renderer."));
        }
        Ok(())
    }
}

pub struct Texture {
    inner: Weak<RendererInner>,
    ptr: *mut sys::render::SDL_Texture,
}

impl Drop for Texture {
    fn drop(&mut self) {
        if self.inner.weak_count() > 0 {
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

enum RendererContext {
    Window(Window),
    Software(Surface),
}

struct RendererInner {
    context: RendererContext,
    target: RefCell<Option<Texture>>,
    ptr: *mut sys::render::SDL_Renderer,
}

impl Drop for RendererInner {
    fn drop(&mut self) {
        unsafe { sys::render::SDL_DestroyRenderer(self.ptr) };
    }
}
