use core::{ffi::CStr, marker::PhantomData};

use alloc::string::String;

use crate::{sys, Error};

/// A structure that represents a color as RGBA components.
///
/// The bits of this structure can be directly reinterpreted as an integer-packed color
/// which uses the [`PixelFormat::Abgr8888`] format (on little-endian systems) or the
/// [`PixelFormat::Rgba8888`] format (on big-endian systems).
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Color(sys::SDL_Color);

impl Color {
    #[inline]
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(sys::SDL_Color { r, g, b, a })
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

    pub fn to_ll(&self) -> sys::SDL_Color {
        self.0
    }
}

impl From<Color> for (u8, u8, u8) {
    fn from(value: Color) -> Self {
        (value.0.r, value.0.g, value.0.b)
    }
}

impl From<Color> for (u8, u8, u8, u8) {
    fn from(value: Color) -> Self {
        (value.0.r, value.0.g, value.0.b, value.0.a)
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from((r, g, b, a): (u8, u8, u8, u8)) -> Self {
        Self::new(r, g, b, a)
    }
}

/// A structure that represents a color as `f32` RGBA components.
///
/// The bits of this structure can be directly reinterpreted as a float-packed color which uses the
/// [`PixelFormat::Rgba128Float`] format.
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct ColorF32(sys::SDL_FColor);

impl ColorF32 {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(sys::SDL_FColor { r, g, b, a })
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

    pub fn to_ll(&self) -> sys::SDL_FColor {
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

impl From<ColorF32> for (f32, f32, f32) {
    fn from(value: ColorF32) -> Self {
        (value.0.r, value.0.g, value.0.b)
    }
}

impl From<ColorF32> for (f32, f32, f32, f32) {
    fn from(value: ColorF32) -> Self {
        (value.0.r, value.0.g, value.0.b, value.0.a)
    }
}

impl From<(f32, f32, f32, f32)> for ColorF32 {
    fn from((r, g, b, a): (f32, f32, f32, f32)) -> Self {
        Self::new(r, g, b, a)
    }
}
/// Pixel format.
///
/// SDL's pixel formats have the following naming convention:
///
/// - Names with a list of components and a single bit count, such as `Rgb24` and `Abgr32`,
/// define a platform-independent encoding into bytes in the order specified. For example,
/// in `Rgb24` data, each pixel is encoded in 3 bytes (red, green, blue) in that order, and
/// in `Abgr32` data, each pixel is encoded in 4 bytes alpha, blue, green, red) in that order.
/// Use these names if the property of a format that is important to you is the order of
/// the bytes in memory or on disk.
///
/// - Names with a bit count per component, such as `Argb8888` and `Xrgb1555`, are "packed"
/// into an appropriately-sized integer in the platform's native endianness. For example,
/// `Argb8888` is a sequence of 32-bit integers; in each integer, the most significant bits
/// are alpha, and the least significant bits are blue. On a little-endian CPU such as x86,
/// the least significant bits of each integer are arranged first in memory, but on a big-endian
/// CPU such as s390x, the most significant bits are arranged first. Use these names if the
/// property of a format that is important to you is the meaning of each bit position within
/// a native-endianness integer.
///
/// - In indexed formats such as `Index4Lsb`, each pixel is represented by encoding an index into
/// the palette into the indicated number of bits, with multiple pixels packed into each byte
/// if appropriate. In LSB formats, the first (leftmost) pixel is stored in the least-significant
/// bits of the byte; in MSB formats, it's stored in the most-significant bits. `Index8` does not
/// need LSB/MSB variants, because each pixel exactly fills one byte.
///
/// The 32-bit byte-array encodings such as `Rgba32` are aliases for the appropriate 8888 encoding
/// for the current platform. For example, `Rgba32` is an alias for `Abgr8888` on little-endian CPUs
/// like x86, or an alias for `Rgba8888` on big-endian CPUs.
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PixelFormat {
    Unknown = sys::SDL_PixelFormat_SDL_PIXELFORMAT_UNKNOWN,
    Index1Lsb = sys::SDL_PixelFormat_SDL_PIXELFORMAT_INDEX1LSB,
    Index1Msb = sys::SDL_PixelFormat_SDL_PIXELFORMAT_INDEX1MSB,
    Index2Lsb = sys::SDL_PixelFormat_SDL_PIXELFORMAT_INDEX2LSB,
    Index2Msb = sys::SDL_PixelFormat_SDL_PIXELFORMAT_INDEX2MSB,
    Index4Lsb = sys::SDL_PixelFormat_SDL_PIXELFORMAT_INDEX4LSB,
    Index4Msb = sys::SDL_PixelFormat_SDL_PIXELFORMAT_INDEX4MSB,
    Index8 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_INDEX8,
    Rgb332 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_RGB332,
    Xrgb4444 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_XRGB4444,
    Xbgr4444 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_XBGR4444,
    Xrgb1555 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_XRGB1555,
    Xbgr1555 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_XBGR1555,
    Argb4444 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_ARGB4444,
    Rgba4444 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_RGBA4444,
    Abgr4444 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_ABGR4444,
    Bgra4444 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_BGRA4444,
    Argb1555 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_ARGB1555,
    Rgba5551 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_RGBA5551,
    Abgr1555 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_ABGR1555,
    Bgra5551 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_BGRA5551,
    Rgb565 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_RGB565,
    Bgr565 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_BGR565,
    Rgb24 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_RGB24,
    Bgr24 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_BGR24,
    Xrgb8888 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_XRGB8888,
    Rgbx8888 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_RGBX8888,
    Xbgr8888 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_XBGR8888,
    Bgrx8888 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_BGRX8888,
    Argb8888 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_ARGB8888,
    Rgba8888 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_RGBA8888,
    Abgr8888 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_ABGR8888,
    Bgra8888 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_BGRA8888,
    Xrgb2101010 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_XRGB2101010,
    Xbgr2101010 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_XBGR2101010,
    Argb2101010 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_ARGB2101010,
    Abgr2101010 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_ABGR2101010,
    Rgb48 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_RGB48,
    Bgr48 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_BGR48,
    Rgba64 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_RGBA64,
    Argb64 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_ARGB64,
    Bgra64 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_BGRA64,
    Abgr64 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_ABGR64,
    Rgb48Float = sys::SDL_PixelFormat_SDL_PIXELFORMAT_RGB48_FLOAT,
    Bgr48Float = sys::SDL_PixelFormat_SDL_PIXELFORMAT_BGR48_FLOAT,
    Rgba64Float = sys::SDL_PixelFormat_SDL_PIXELFORMAT_RGBA64_FLOAT,
    Argb64Float = sys::SDL_PixelFormat_SDL_PIXELFORMAT_ARGB64_FLOAT,
    Bgra64Float = sys::SDL_PixelFormat_SDL_PIXELFORMAT_BGRA64_FLOAT,
    Abgr64Float = sys::SDL_PixelFormat_SDL_PIXELFORMAT_ABGR64_FLOAT,
    Rgb96Float = sys::SDL_PixelFormat_SDL_PIXELFORMAT_RGB96_FLOAT,
    Bgr96Float = sys::SDL_PixelFormat_SDL_PIXELFORMAT_BGR96_FLOAT,
    Rgba128Float = sys::SDL_PixelFormat_SDL_PIXELFORMAT_RGBA128_FLOAT,
    Argb128Float = sys::SDL_PixelFormat_SDL_PIXELFORMAT_ARGB128_FLOAT,
    Bgra128Float = sys::SDL_PixelFormat_SDL_PIXELFORMAT_BGRA128_FLOAT,
    Abgr128Float = sys::SDL_PixelFormat_SDL_PIXELFORMAT_ABGR128_FLOAT,
    /// Planar mode: Y + V + U  (3 planes)
    Yv12 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_YV12,
    /// Planar mode: Y + U + V  (3 planes)
    Iyuv = sys::SDL_PixelFormat_SDL_PIXELFORMAT_IYUV,
    /// Packed mode: Y0+U0+Y1+V0 (1 plane)
    Yuy2 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_YUY2,
    /// Packed mode: U0+Y0+V0+Y1 (1 plane)
    Uyvy = sys::SDL_PixelFormat_SDL_PIXELFORMAT_UYVY,
    /// Packed mode: Y0+V0+Y1+U0 (1 plane)
    Yvyu = sys::SDL_PixelFormat_SDL_PIXELFORMAT_YVYU,
    /// Planar mode: Y + U/V interleaved  (2 planes)
    Nv12 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_NV12,
    /// Planar mode: Y + V/U interleaved  (2 planes)
    Nv21 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_NV21,
    /// Planar mode: Y + U/V interleaved  (2 planes)
    P010 = sys::SDL_PixelFormat_SDL_PIXELFORMAT_P010,
    /// Android video texture format
    ExternalOes = sys::SDL_PixelFormat_SDL_PIXELFORMAT_EXTERNAL_OES,
}

impl PixelFormat {
    /// Attempts to convert from a low-level SDL pixel format to PixelFormat
    /// It assumes the internal pixel format is valid since it comes from SDL!
    pub(crate) unsafe fn from_ll_unchecked(format: sys::SDL_PixelFormat) -> Self {
        // Since we're using repr(i32) and the values match exactly,
        // we can safely transmute the integer value
        let format_val = format;
        unsafe { core::mem::transmute(format_val) }
    }

    #[inline]
    pub fn to_ll(&self) -> sys::SDL_PixelFormat {
        *self as u32
    }

    pub fn details(&self) -> Result<&PixelFormatDetails, Error> {
        let details = unsafe { sys::SDL_GetPixelFormatDetails(self.to_ll()) };
        if details.is_null() {
            return Err(Error::new());
        }
        Ok(unsafe { PixelFormatDetails::from_ptr(details) })
    }

    pub fn masks(&self) -> Result<PixelFormatRgbaMask, Error> {
        let mut bpp = 0;
        let mut r_mask = 0;
        let mut g_mask = 0;
        let mut b_mask = 0;
        let mut a_mask = 0;
        let result = unsafe {
            sys::SDL_GetMasksForPixelFormat(
                self.to_ll(),
                &raw mut bpp,
                &raw mut r_mask,
                &raw mut g_mask,
                &raw mut b_mask,
                &raw mut a_mask,
            )
        };
        if !result {
            return Err(Error::new());
        }
        Ok(PixelFormatRgbaMask {
            bpp,
            r_mask,
            g_mask,
            b_mask,
            a_mask,
        })
    }

    pub fn name(&self) -> String {
        unsafe {
            let ptr = sys::SDL_GetPixelFormatName(self.to_ll());
            let c_str = CStr::from_ptr(ptr);
            String::from_utf8_lossy(c_str.to_bytes()).into_owned()
        }
    }
}

/// Zero-sized struct equivalent to `SDL_PixelFormatDetails`.
// This struct is zero-sized.
// We cast *SDL_PixelFormatDetails to &PixelFormatDetails.
// It can't be constructed outside of this crate and it's only exposed as a reference.
pub struct PixelFormatDetails {
    // !Send + !Sync
    _inner: PhantomData<*const ()>,
}

impl PixelFormatDetails {
    pub(crate) unsafe fn from_ptr<'a>(ptr: *const sys::SDL_PixelFormatDetails) -> &'a Self {
        &*(ptr as *const Self)
    }

    pub fn raw(&self) -> *const sys::SDL_PixelFormatDetails {
        self as *const PixelFormatDetails as *const _
    }

    /// Map an RGB triple to an opaque pixel value for a given pixel format.
    ///
    /// This function maps the RGB color value to the specified pixel format and returns the
    /// pixel value best approximating the given RGB color value for the given pixel format.
    ///
    /// If the format has a palette (8-bit) the index of the closest matching color in the palette
    /// will be returned.
    ///
    /// If the specified pixel format has an alpha component it will be returned as all 1 bits
    /// (fully opaque).
    ///
    /// If the pixel format bpp (color depth) is less than 32-bpp then the unused upper bits of
    /// the return value can safely be ignored (e.g., with a 16-bpp format the return value can
    /// be assigned to a Uint16, and similarly a Uint8 for an 8-bpp format).
    pub fn map_rgb(&self, palette: Option<&Palette>, r: u8, g: u8, b: u8) -> u32 {
        let palette = palette
            .map(|p| p.ptr as *const _)
            .unwrap_or(core::ptr::null());
        unsafe { sys::SDL_MapRGB(self.raw(), palette, r, g, b) }
    }

    /// Map an RGBA quadruple to a pixel value for a given pixel format.
    ///
    /// This function maps the RGBA color value to the specified pixel format and returns the
    /// pixel value best approximating the given RGBA color value for the given pixel format.
    ///
    /// If the specified pixel format has no alpha component the alpha value will be ignored
    /// (as it will be in formats with a palette).
    ///
    /// If the format has a palette (8-bit) the index of the closest matching color in the
    /// palette will be returned.
    ///
    /// If the pixel format bpp (color depth) is less than 32-bpp then the unused upper bits
    /// of the return value can safely be ignored (e.g., with a 16-bpp format the return value
    /// can be assigned to a Uint16, and similarly a Uint8 for an 8-bpp format).
    pub fn map_rgba(&self, palette: Option<&Palette>, r: u8, g: u8, b: u8, a: u8) -> u32 {
        let palette = palette
            .map(|p| p.ptr as *const _)
            .unwrap_or(core::ptr::null());
        unsafe { sys::SDL_MapRGBA(self.raw(), palette, r, g, b, a) }
    }

    /// Get RGB values from a pixel in the specified format.
    ///
    /// This function uses the entire 8-bit [0..255] range when converting color components
    /// from pixel formats with less than 8-bits per RGB component (e.g., a completely white
    /// pixel in 16-bit RGB565 format would return [0xff, 0xff, 0xff] not [0xf8, 0xfc, 0xf8]).
    pub fn rgb(&self, pixel: u32, palette: Option<&Palette>) -> (u8, u8, u8) {
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        let palette = palette
            .map(|p| p.ptr as *const _)
            .unwrap_or(core::ptr::null());
        unsafe {
            sys::SDL_GetRGB(
                pixel,
                self.raw(),
                palette,
                &raw mut r,
                &raw mut g,
                &raw mut b,
            )
        };
        (r, g, b)
    }

    /// Get RGBA values from a pixel in the specified format.
    ///
    /// This function uses the entire 8-bit [0..255] range when converting color components
    /// from pixel formats with less than 8-bits per RGB component (e.g., a completely white
    /// pixel in 16-bit RGB565 format would return [0xff, 0xff, 0xff] not [0xf8, 0xfc, 0xf8]).
    ///
    /// If the surface has no alpha component, the alpha will be returned as 0xff (100% opaque).
    pub fn rgba(&self, pixel: u32, palette: Option<&Palette>) -> (u8, u8, u8, u8) {
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        let mut a = 0;
        let palette = palette
            .map(|p| p.ptr as *const _)
            .unwrap_or(core::ptr::null());
        unsafe {
            sys::SDL_GetRGBA(
                pixel,
                self.raw(),
                palette,
                &raw mut r,
                &raw mut g,
                &raw mut b,
                &raw mut a,
            )
        };
        (r, g, b, a)
    }

    #[inline]
    pub fn format(&self) -> PixelFormat {
        unsafe { PixelFormat::from_ll_unchecked((*self.raw()).format) }
    }

    #[inline]
    pub fn bits_per_pixel(&self) -> u8 {
        unsafe { (*self.raw()).bits_per_pixel }
    }

    #[inline]
    pub fn bytes_per_pixel(&self) -> u8 {
        unsafe { (*self.raw()).bytes_per_pixel }
    }

    #[inline]
    pub fn byte_size_from_pitch_and_height(&self, pitch: usize, height: u32) -> usize {
        let height = height as usize;
        match self.format() {
            PixelFormat::Yv12 | PixelFormat::Iyuv => {
                // YUV is 4:2:0.
                // `pitch` is the width of the Y component, and
                // `height` is the height of the Y component.
                // U and V have half the width and height of Y.
                pitch * height + 2 * (pitch / 2 * height / 2)
            }
            _ => pitch * height,
        }
    }

    #[inline]
    pub fn padding(&self) -> [u8; 2] {
        unsafe { (*self.raw()).padding }
    }

    #[inline]
    pub fn r_mask(&self) -> u32 {
        unsafe { (*self.raw()).Rmask }
    }

    #[inline]
    pub fn g_mask(&self) -> u32 {
        unsafe { (*self.raw()).Gmask }
    }

    #[inline]
    pub fn b_mask(&self) -> u32 {
        unsafe { (*self.raw()).Bmask }
    }

    #[inline]
    pub fn a_mask(&self) -> u32 {
        unsafe { (*self.raw()).Amask }
    }

    #[inline]
    pub fn r_bits(&self) -> u8 {
        unsafe { (*self.raw()).Rbits }
    }

    #[inline]
    pub fn g_bits(&self) -> u8 {
        unsafe { (*self.raw()).Gbits }
    }

    #[inline]
    pub fn b_bits(&self) -> u8 {
        unsafe { (*self.raw()).Bbits }
    }

    #[inline]
    pub fn a_bits(&self) -> u8 {
        unsafe { (*self.raw()).Abits }
    }

    #[inline]
    pub fn r_shift(&self) -> u8 {
        unsafe { (*self.raw()).Rshift }
    }

    #[inline]
    pub fn g_shift(&self) -> u8 {
        unsafe { (*self.raw()).Gshift }
    }

    #[inline]
    pub fn b_shift(&self) -> u8 {
        unsafe { (*self.raw()).Bshift }
    }

    #[inline]
    pub fn a_shift(&self) -> u8 {
        unsafe { (*self.raw()).Ashift }
    }
}

// TODO: once we start supporting Surface color palettes there's a chance we'll
// have to add a lifetime parameter to this so we can ACTUALLY have exclusive access to the palette.
/// A set of indexed colors representing a palette.
///
/// The ownership of the underlying palette is not necessarily unique. SDL uses refcounting internally.
///
/// This means that two different Palette objects can point to the same underlying SDL_Palette. It
/// also means that dropping the palette will not necessarily destroy the underlying SDL_Palette.
pub struct Palette {
    ptr: *mut sys::SDL_Palette,
}

impl Palette {
    pub fn new(num_colors: usize) -> Result<Self, Error> {
        let result = unsafe { sys::SDL_CreatePalette(num_colors as i32) };
        if result.is_null() {
            return Err(Error::new());
        }
        Ok(Self { ptr: result })
    }

    pub(crate) unsafe fn from_mut_ptr(ptr: *mut sys::SDL_Palette) -> Self {
        Self { ptr }
    }

    /// Set a range of colors in a palette.
    pub fn set_colors(&mut self, colors: &[Color], at_index: usize) -> Result<(), Error> {
        let colors_ptr = colors.as_ptr() as *const sys::SDL_Color;
        let result = unsafe {
            sys::SDL_SetPaletteColors(
                self.ptr,
                colors_ptr,
                i32::try_from(at_index)?,
                i32::try_from(colors.len())?,
            )
        };
        // SDL will return an error if the array doesn't have enough room for the color OR if the
        // at_index is invalid. That being said... it's an empty error.
        if !result {
            return Err(Error::new());
        }
        Ok(())
    }

    /// Returns a slice with this palette's colors.
    pub fn colors(&self) -> &[Color] {
        unsafe {
            let len = (*self.ptr).ncolors as usize;
            let colors = (*self.ptr).colors;
            core::slice::from_raw_parts(colors as *const Color, len)
        }
    }

    pub fn raw(&self) -> *mut sys::SDL_Palette {
        self.ptr
    }
}

impl Drop for Palette {
    fn drop(&mut self) {
        unsafe { sys::SDL_DestroyPalette(self.ptr) };
    }
}

/// A bits-per-pixel value and RGBA masks.
///
/// This is used as a return value for [`PixelFormat::masks`].
#[derive(Copy, Clone, Debug)]
pub struct PixelFormatRgbaMask {
    pub bpp: i32,
    pub r_mask: u32,
    pub g_mask: u32,
    pub b_mask: u32,
    pub a_mask: u32,
}

/// Colorspace definitions.
///
/// Since similar colorspaces may vary in their details (matrix, transfer function, etc.), this is not
/// an exhaustive list, but rather a representative sample of the kinds of colorspaces supported in SDL.
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Colorspace(sys::SDL_Colorspace);

impl Colorspace {
    pub const UNKNOWN: Self = Self(sys::SDL_Colorspace_SDL_COLORSPACE_UNKNOWN);
    pub const SRGB: Self = Self(sys::SDL_Colorspace_SDL_COLORSPACE_SRGB);
    pub const SRGB_LINEAR: Self = Self(sys::SDL_Colorspace_SDL_COLORSPACE_SRGB_LINEAR);
    pub const HDR10: Self = Self(sys::SDL_Colorspace_SDL_COLORSPACE_HDR10);
    pub const JPEG: Self = Self(sys::SDL_Colorspace_SDL_COLORSPACE_JPEG);
    pub const BT601_LIMITED: Self = Self(sys::SDL_Colorspace_SDL_COLORSPACE_BT601_LIMITED);
    pub const BT601_FULL: Self = Self(sys::SDL_Colorspace_SDL_COLORSPACE_BT601_FULL);
    pub const BT2020_LIMITED: Self = Self(sys::SDL_Colorspace_SDL_COLORSPACE_BT2020_LIMITED);
    pub const BT2020_FULL: Self = Self(sys::SDL_Colorspace_SDL_COLORSPACE_BT2020_FULL);
    pub const RGB_DEFAULT: Self = Self(sys::SDL_Colorspace_SDL_COLORSPACE_RGB_DEFAULT);
    pub const YUV_DEFAULT: Self = Self(sys::SDL_Colorspace_SDL_COLORSPACE_YUV_DEFAULT);

    #[inline]
    pub(crate) fn from_ll(ll: sys::SDL_Colorspace) -> Self {
        Self(ll)
    }

    #[inline]
    pub fn to_ll(&self) -> sys::SDL_Colorspace {
        self.0
    }
}
