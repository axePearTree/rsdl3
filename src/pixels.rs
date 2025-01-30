use crate::sys;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Color(sys::pixels::SDL_Color);

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(sys::pixels::SDL_Color { r, g, b, a })
    }

    #[inline]
    pub fn r(&self) -> u8 {
        self.0.r
    }

    #[inline]
    pub fn g(&self) -> u8 {
        self.0.g
    }

    #[inline]
    pub fn b(&self) -> u8 {
        self.0.b
    }

    #[inline]
    pub fn a(&self) -> u8 {
        self.0.a
    }

    #[inline]
    pub fn set_r(&mut self, r: u8) {
        self.0.r = r;
    }

    #[inline]
    pub fn set_g(&mut self, g: u8) {
        self.0.g = g;
    }

    #[inline]
    pub fn set_b(&mut self, b: u8) {
        self.0.b = b;
    }

    #[inline]
    pub fn set_a(&mut self, a: u8) {
        self.0.a = a;
    }

    pub fn raw(&self) -> sys::pixels::SDL_Color {
        self.0
    }
}

pub struct PixelFormat(sys::pixels::SDL_PixelFormat);

impl PixelFormat {
    pub fn raw(&self) -> sys::pixels::SDL_PixelFormat {
        self.0
    }
}

impl PixelFormat {
    pub const UNKNOWN: Self = Self(sys::pixels::SDL_PixelFormat::UNKNOWN);
    pub const INDEX1LSB: Self = Self(sys::pixels::SDL_PixelFormat::INDEX1LSB);
    pub const INDEX1MSB: Self = Self(sys::pixels::SDL_PixelFormat::INDEX1MSB);
    pub const INDEX2LSB: Self = Self(sys::pixels::SDL_PixelFormat::INDEX2LSB);
    pub const INDEX2MSB: Self = Self(sys::pixels::SDL_PixelFormat::INDEX2MSB);
    pub const INDEX4LSB: Self = Self(sys::pixels::SDL_PixelFormat::INDEX4LSB);
    pub const INDEX4MSB: Self = Self(sys::pixels::SDL_PixelFormat::INDEX4MSB);
    pub const INDEX8: Self = Self(sys::pixels::SDL_PixelFormat::INDEX8);
    pub const RGB332: Self = Self(sys::pixels::SDL_PixelFormat::RGB332);
    pub const XRGB4444: Self = Self(sys::pixels::SDL_PixelFormat::XRGB4444);
    pub const XBGR4444: Self = Self(sys::pixels::SDL_PixelFormat::XBGR4444);
    pub const XRGB1555: Self = Self(sys::pixels::SDL_PixelFormat::XRGB1555);
    pub const XBGR1555: Self = Self(sys::pixels::SDL_PixelFormat::XBGR1555);
    pub const ARGB4444: Self = Self(sys::pixels::SDL_PixelFormat::ARGB4444);
    pub const RGBA4444: Self = Self(sys::pixels::SDL_PixelFormat::RGBA4444);
    pub const ABGR4444: Self = Self(sys::pixels::SDL_PixelFormat::ABGR4444);
    pub const BGRA4444: Self = Self(sys::pixels::SDL_PixelFormat::BGRA4444);
    pub const ARGB1555: Self = Self(sys::pixels::SDL_PixelFormat::ARGB1555);
    pub const RGBA5551: Self = Self(sys::pixels::SDL_PixelFormat::RGBA5551);
    pub const ABGR1555: Self = Self(sys::pixels::SDL_PixelFormat::ABGR1555);
    pub const BGRA5551: Self = Self(sys::pixels::SDL_PixelFormat::BGRA5551);
    pub const RGB565: Self = Self(sys::pixels::SDL_PixelFormat::RGB565);
    pub const BGR565: Self = Self(sys::pixels::SDL_PixelFormat::BGR565);
    pub const RGB24: Self = Self(sys::pixels::SDL_PixelFormat::RGB24);
    pub const BGR24: Self = Self(sys::pixels::SDL_PixelFormat::BGR24);
    pub const XRGB8888: Self = Self(sys::pixels::SDL_PixelFormat::XRGB8888);
    pub const RGBX8888: Self = Self(sys::pixels::SDL_PixelFormat::RGBX8888);
    pub const XBGR8888: Self = Self(sys::pixels::SDL_PixelFormat::XBGR8888);
    pub const BGRX8888: Self = Self(sys::pixels::SDL_PixelFormat::BGRX8888);
    pub const ARGB8888: Self = Self(sys::pixels::SDL_PixelFormat::ARGB8888);
    pub const RGBA8888: Self = Self(sys::pixels::SDL_PixelFormat::RGBA8888);
    pub const ABGR8888: Self = Self(sys::pixels::SDL_PixelFormat::ABGR8888);
    pub const BGRA8888: Self = Self(sys::pixels::SDL_PixelFormat::BGRA8888);
    pub const XRGB2101010: Self = Self(sys::pixels::SDL_PixelFormat::XRGB2101010);
    pub const XBGR2101010: Self = Self(sys::pixels::SDL_PixelFormat::XBGR2101010);
    pub const ARGB2101010: Self = Self(sys::pixels::SDL_PixelFormat::ARGB2101010);
    pub const ABGR2101010: Self = Self(sys::pixels::SDL_PixelFormat::ABGR2101010);
    pub const RGB48: Self = Self(sys::pixels::SDL_PixelFormat::RGB48);
    pub const BGR48: Self = Self(sys::pixels::SDL_PixelFormat::BGR48);
    pub const RGBA64: Self = Self(sys::pixels::SDL_PixelFormat::RGBA64);
    pub const ARGB64: Self = Self(sys::pixels::SDL_PixelFormat::ARGB64);
    pub const BGRA64: Self = Self(sys::pixels::SDL_PixelFormat::BGRA64);
    pub const ABGR64: Self = Self(sys::pixels::SDL_PixelFormat::ABGR64);
    pub const RGB48_FLOAT: Self = Self(sys::pixels::SDL_PixelFormat::RGB48_FLOAT);
    pub const BGR48_FLOAT: Self = Self(sys::pixels::SDL_PixelFormat::BGR48_FLOAT);
    pub const RGBA64_FLOAT: Self = Self(sys::pixels::SDL_PixelFormat::RGBA64_FLOAT);
    pub const ARGB64_FLOAT: Self = Self(sys::pixels::SDL_PixelFormat::ARGB64_FLOAT);
    pub const BGRA64_FLOAT: Self = Self(sys::pixels::SDL_PixelFormat::BGRA64_FLOAT);
    pub const ABGR64_FLOAT: Self = Self(sys::pixels::SDL_PixelFormat::ABGR64_FLOAT);
    pub const RGB96_FLOAT: Self = Self(sys::pixels::SDL_PixelFormat::RGB96_FLOAT);
    pub const BGR96_FLOAT: Self = Self(sys::pixels::SDL_PixelFormat::BGR96_FLOAT);
    pub const RGBA128_FLOAT: Self = Self(sys::pixels::SDL_PixelFormat::RGBA128_FLOAT);
    pub const ARGB128_FLOAT: Self = Self(sys::pixels::SDL_PixelFormat::ARGB128_FLOAT);
    pub const BGRA128_FLOAT: Self = Self(sys::pixels::SDL_PixelFormat::BGRA128_FLOAT);
    pub const ABGR128_FLOAT: Self = Self(sys::pixels::SDL_PixelFormat::ABGR128_FLOAT);
    /// Planar mode: Y + V + U  (3 planes)
    pub const YV12: Self = Self(sys::pixels::SDL_PixelFormat::YV12);
    /// Planar mode: Y + U + V  (3 planes)
    pub const IYUV: Self = Self(sys::pixels::SDL_PixelFormat::IYUV);
    /// Packed mode: Y0+U0+Y1+V0 (1 plane)
    pub const YUY2: Self = Self(sys::pixels::SDL_PixelFormat::YUY2);
    /// Packed mode: U0+Y0+V0+Y1 (1 plane)
    pub const UYVY: Self = Self(sys::pixels::SDL_PixelFormat::UYVY);
    /// Packed mode: Y0+V0+Y1+U0 (1 plane)
    pub const YVYU: Self = Self(sys::pixels::SDL_PixelFormat::YVYU);
    /// Planar mode: Y + U/V interleaved  (2 planes)
    pub const NV12: Self = Self(sys::pixels::SDL_PixelFormat::NV12);
    /// Planar mode: Y + V/U interleaved  (2 planes)
    pub const NV21: Self = Self(sys::pixels::SDL_PixelFormat::NV21);
    /// Planar mode: Y + U/V interleaved  (2 planes)
    pub const P010: Self = Self(sys::pixels::SDL_PixelFormat::P010);
    /// Android video texture format
    pub const EXTERNAL_OES: Self = Self(sys::pixels::SDL_PixelFormat::EXTERNAL_OES);
}
