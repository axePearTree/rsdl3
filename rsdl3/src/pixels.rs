use crate::{sys, Error};

#[derive(Copy, Clone, Debug)]
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

    pub fn to_ll(&self) -> sys::pixels::SDL_Color {
        self.0
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct ColorF32(sys::pixels::SDL_FColor);

impl ColorF32 {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(sys::pixels::SDL_FColor { r, g, b, a })
    }

    #[inline]
    pub fn r(&self) -> f32 {
        self.0.r
    }

    #[inline]
    pub fn g(&self) -> f32 {
        self.0.g
    }

    #[inline]
    pub fn b(&self) -> f32 {
        self.0.b
    }

    #[inline]
    pub fn a(&self) -> f32 {
        self.0.a
    }

    #[inline]
    pub fn set_r(&mut self, r: f32) {
        self.0.r = r;
    }

    #[inline]
    pub fn set_g(&mut self, g: f32) {
        self.0.g = g;
    }

    #[inline]
    pub fn set_b(&mut self, b: f32) {
        self.0.b = b;
    }

    #[inline]
    pub fn set_a(&mut self, a: f32) {
        self.0.a = a;
    }

    pub fn to_ll(&self) -> sys::pixels::SDL_FColor {
        self.0
    }
}

impl From<Color> for ColorF32 {
    fn from(value: Color) -> Self {
        let r = value.r() as f32 / 255.0;
        let g = value.g() as f32 / 255.0;
        let b = value.b() as f32 / 255.0;
        let a = value.a() as f32 / 255.0;
        Self::new(r, g, b, a)
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum PixelFormat {
    Unknown = sys::pixels::SDL_PixelFormat::UNKNOWN.0,
    Index1Lsb = sys::pixels::SDL_PixelFormat::INDEX1LSB.0,
    Index1Msb = sys::pixels::SDL_PixelFormat::INDEX1MSB.0,
    Index2Lsb = sys::pixels::SDL_PixelFormat::INDEX2LSB.0,
    Index2Msb = sys::pixels::SDL_PixelFormat::INDEX2MSB.0,
    Index4Lsb = sys::pixels::SDL_PixelFormat::INDEX4LSB.0,
    Index4Msb = sys::pixels::SDL_PixelFormat::INDEX4MSB.0,
    Index8 = sys::pixels::SDL_PixelFormat::INDEX8.0,
    Rgb332 = sys::pixels::SDL_PixelFormat::RGB332.0,
    Xrgb4444 = sys::pixels::SDL_PixelFormat::XRGB4444.0,
    Xbgr4444 = sys::pixels::SDL_PixelFormat::XBGR4444.0,
    Xrgb1555 = sys::pixels::SDL_PixelFormat::XRGB1555.0,
    Xbgr1555 = sys::pixels::SDL_PixelFormat::XBGR1555.0,
    Argb4444 = sys::pixels::SDL_PixelFormat::ARGB4444.0,
    Rgba4444 = sys::pixels::SDL_PixelFormat::RGBA4444.0,
    Abgr4444 = sys::pixels::SDL_PixelFormat::ABGR4444.0,
    Bgra4444 = sys::pixels::SDL_PixelFormat::BGRA4444.0,
    Argb1555 = sys::pixels::SDL_PixelFormat::ARGB1555.0,
    Rgba5551 = sys::pixels::SDL_PixelFormat::RGBA5551.0,
    Abgr1555 = sys::pixels::SDL_PixelFormat::ABGR1555.0,
    Bgra5551 = sys::pixels::SDL_PixelFormat::BGRA5551.0,
    Rgb565 = sys::pixels::SDL_PixelFormat::RGB565.0,
    Bgr565 = sys::pixels::SDL_PixelFormat::BGR565.0,
    Rgb24 = sys::pixels::SDL_PixelFormat::RGB24.0,
    Bgr24 = sys::pixels::SDL_PixelFormat::BGR24.0,
    Xrgb8888 = sys::pixels::SDL_PixelFormat::XRGB8888.0,
    Rgbx8888 = sys::pixels::SDL_PixelFormat::RGBX8888.0,
    Xbgr8888 = sys::pixels::SDL_PixelFormat::XBGR8888.0,
    Bgrx8888 = sys::pixels::SDL_PixelFormat::BGRX8888.0,
    Argb8888 = sys::pixels::SDL_PixelFormat::ARGB8888.0,
    Rgba8888 = sys::pixels::SDL_PixelFormat::RGBA8888.0,
    Abgr8888 = sys::pixels::SDL_PixelFormat::ABGR8888.0,
    Bgra8888 = sys::pixels::SDL_PixelFormat::BGRA8888.0,
    Xrgb2101010 = sys::pixels::SDL_PixelFormat::XRGB2101010.0,
    Xbgr2101010 = sys::pixels::SDL_PixelFormat::XBGR2101010.0,
    Argb2101010 = sys::pixels::SDL_PixelFormat::ARGB2101010.0,
    Abgr2101010 = sys::pixels::SDL_PixelFormat::ABGR2101010.0,
    Rgb48 = sys::pixels::SDL_PixelFormat::RGB48.0,
    Bgr48 = sys::pixels::SDL_PixelFormat::BGR48.0,
    Rgba64 = sys::pixels::SDL_PixelFormat::RGBA64.0,
    Argb64 = sys::pixels::SDL_PixelFormat::ARGB64.0,
    Bgra64 = sys::pixels::SDL_PixelFormat::BGRA64.0,
    Abgr64 = sys::pixels::SDL_PixelFormat::ABGR64.0,
    Rgb48Float = sys::pixels::SDL_PixelFormat::RGB48_FLOAT.0,
    Bgr48Float = sys::pixels::SDL_PixelFormat::BGR48_FLOAT.0,
    Rgba64Float = sys::pixels::SDL_PixelFormat::RGBA64_FLOAT.0,
    Argb64Float = sys::pixels::SDL_PixelFormat::ARGB64_FLOAT.0,
    Bgra64Float = sys::pixels::SDL_PixelFormat::BGRA64_FLOAT.0,
    Abgr64Float = sys::pixels::SDL_PixelFormat::ABGR64_FLOAT.0,
    Rgb96Float = sys::pixels::SDL_PixelFormat::RGB96_FLOAT.0,
    Bgr96Float = sys::pixels::SDL_PixelFormat::BGR96_FLOAT.0,
    Rgba128Float = sys::pixels::SDL_PixelFormat::RGBA128_FLOAT.0,
    Argb128Float = sys::pixels::SDL_PixelFormat::ARGB128_FLOAT.0,
    Bgra128Float = sys::pixels::SDL_PixelFormat::BGRA128_FLOAT.0,
    Abgr128Float = sys::pixels::SDL_PixelFormat::ABGR128_FLOAT.0,
    /// Planar mode: Y + V + U  (3 planes)
    Yv12 = sys::pixels::SDL_PixelFormat::YV12.0,
    /// Planar mode: Y + U + V  (3 planes)
    Iyuv = sys::pixels::SDL_PixelFormat::IYUV.0,
    /// Packed mode: Y0+U0+Y1+V0 (1 plane)
    Yuy2 = sys::pixels::SDL_PixelFormat::YUY2.0,
    /// Packed mode: U0+Y0+V0+Y1 (1 plane)
    Uyvy = sys::pixels::SDL_PixelFormat::UYVY.0,
    /// Packed mode: Y0+V0+Y1+U0 (1 plane)
    Yvyu = sys::pixels::SDL_PixelFormat::YVYU.0,
    /// Planar mode: Y + U/V interleaved  (2 planes)
    Nv12 = sys::pixels::SDL_PixelFormat::NV12.0,
    /// Planar mode: Y + V/U interleaved  (2 planes)
    Nv21 = sys::pixels::SDL_PixelFormat::NV21.0,
    /// Planar mode: Y + U/V interleaved  (2 planes)
    P010 = sys::pixels::SDL_PixelFormat::P010.0,
    /// Android video texture format
    ExternalOes = sys::pixels::SDL_PixelFormat::EXTERNAL_OES.0,
}

impl PixelFormat {
    /// Attempts to convert from a low-level SDL pixel format to PixelFormat
    /// It assumes the internal pixel format is valid since it comes from SDL!
    pub(crate) unsafe fn from_ll_unchecked(format: sys::pixels::SDL_PixelFormat) -> Self {
        // Since we're using repr(i32) and the values match exactly,
        // we can safely transmute the integer value
        let format_val = format.0 as i32;
        unsafe { core::mem::transmute(format_val) }
    }

    #[inline]
    pub fn to_ll(&self) -> sys::pixels::SDL_PixelFormat {
        sys::pixels::SDL_PixelFormat(*self as i32)
    }
}
