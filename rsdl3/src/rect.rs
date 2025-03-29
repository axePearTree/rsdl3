use crate::sys;

const MAX_INT: u32 = i32::MAX as u32 / 2;

const MIN_INT: i32 = i32::MIN / 2;

fn clamp_size(val: u32) -> i32 {
    val.max(1).min(MAX_INT) as i32
}

fn clamp_position(val: i32) -> i32 {
    val.clamp(MIN_INT, MAX_INT as i32)
}

fn clamp_size_i32(val: i32) -> i32 {
    val.max(1).min(MAX_INT as i32)
}

fn clamp_position_f32(val: f32) -> f32 {
    val.clamp(MIN_INT as f32, MAX_INT as f32)
}

// SAFETY: must be transparent
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Rect(pub(crate) sys::SDL_Rect);

impl Rect {
    /// Creates a new `Rect` with the given dimensions.
    /// The position and dimensions of the Rect need to be clamped to avoid overflowing the corners
    /// of the rectangle.
    /// The width and height must be greater than 0, otherwise they'll be set to 1.
    #[inline]
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self(sys::SDL_Rect {
            x: clamp_position(x),
            y: clamp_position(y),
            w: clamp_size(w),
            h: clamp_size(h),
        })
    }

    /// Creates a new `Rect` with the given dimensions from an existing [`sys::SDL_Rect`].
    /// The position and dimensions of the Rect need to be clamped to avoid overflowing the corners
    /// of the rectangle.
    /// The width and height must be greater than 0, otherwise they'll be set to 1.
    #[inline]
    pub fn from_ll(rect: sys::SDL_Rect) -> Self {
        Self(sys::SDL_Rect {
            x: clamp_position(rect.x),
            y: clamp_position(rect.y),
            w: clamp_size_i32(rect.w),
            h: clamp_size_i32(rect.h),
        })
    }

    #[inline]
    pub fn x(&self) -> i32 {
        self.0.x
    }

    #[inline]
    pub fn set_x(&mut self, x: i32) {
        self.0.x = clamp_position(x);
    }

    #[inline]
    pub fn y(&self) -> i32 {
        self.0.y
    }

    #[inline]
    pub fn set_y(&mut self, y: i32) {
        self.0.y = clamp_position(y);
    }

    #[inline]
    pub fn w(&self) -> u32 {
        self.0.w as u32
    }

    #[inline]
    pub fn set_w(&mut self, w: u32) {
        self.0.w = clamp_size(w);
    }

    #[inline]
    pub fn h(&self) -> u32 {
        self.0.h as u32
    }

    #[inline]
    pub fn set_h(&mut self, h: u32) {
        self.0.h = clamp_size(h);
    }

    #[inline]
    pub fn to_ll(self) -> sys::SDL_Rect {
        self.0
    }

    pub(crate) fn as_raw(&self) -> *const sys::SDL_Rect {
        self as *const Self as *const sys::SDL_Rect
    }
}

// SAFETY: must be transparent
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct RectF32(sys::SDL_FRect);

impl RectF32 {
    /// Creates a new `Rect` with the given dimensions.
    /// The position and dimensions of the Rect need to be clamped to avoid overflowing the corners
    /// of the rectangle.
    /// The width and height must be greater than 0, otherwise they'll be set to 1.
    #[inline]
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        let rect = sys::SDL_Rect {
            x: clamp_position(x as i32),
            y: clamp_position(y as i32),
            w: clamp_size(w.max(0.0) as u32),
            h: clamp_size(h.max(0.0) as u32),
        };
        Self(sys::SDL_FRect {
            x: rect.x as f32,
            y: rect.y as f32,
            w: rect.w as f32,
            h: rect.h as f32,
        })
    }

    /// Creates a new `Rect` with the given dimensions from an existing [`sys::SDL_Rect`].
    /// The position and dimensions of the Rect need to be clamped to avoid overflowing the corners
    /// of the rectangle.
    /// The width and height must be greater than 0, otherwise they'll be set to 1.
    #[inline]
    pub fn from_ll(rect: sys::SDL_FRect) -> Self {
        Self(sys::SDL_FRect {
            x: clamp_position(rect.x as i32) as f32,
            y: clamp_position(rect.y as i32) as f32,
            w: clamp_size_i32(rect.w as i32) as f32,
            h: clamp_size_i32(rect.h as i32) as f32,
        })
    }

    #[inline]
    pub fn x(&self) -> f32 {
        self.0.x
    }

    #[inline]
    pub fn set_x(&mut self, x: f32) {
        self.0.x = clamp_position(x as i32) as f32;
    }

    #[inline]
    pub fn y(&self) -> f32 {
        self.0.y
    }

    #[inline]
    pub fn set_y(&mut self, y: f32) {
        self.0.y = clamp_position(y as i32) as f32;
    }

    #[inline]
    pub fn w(&self) -> f32 {
        self.0.w
    }

    #[inline]
    pub fn set_w(&mut self, w: f32) {
        self.0.w = clamp_size(w.max(0.0) as u32) as f32;
    }

    #[inline]
    pub fn h(&self) -> f32 {
        self.0.h
    }

    #[inline]
    pub fn set_h(&mut self, h: f32) {
        self.0.h = clamp_size(h.max(0.0) as u32) as f32;
    }

    #[inline]
    pub fn to_ll(self) -> sys::SDL_FRect {
        self.0
    }

    pub(crate) fn as_raw(&self) -> *const sys::SDL_FRect {
        self as *const Self as *const sys::SDL_FRect
    }
}

impl From<Rect> for RectF32 {
    fn from(value: Rect) -> Self {
        RectF32::new(
            value.x() as f32,
            value.y() as f32,
            value.w() as f32,
            value.h() as f32,
        )
    }
}

// SAFETY: must be transparent
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Point(sys::SDL_Point);

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self(sys::SDL_Point {
            x: clamp_position(x),
            y: clamp_position(y),
        })
    }

    #[inline]
    pub fn x(&self) -> i32 {
        self.0.x
    }

    #[inline]
    pub fn set_x(&mut self, x: i32) {
        self.0.x = clamp_position(x);
    }

    #[inline]
    pub fn y(&self) -> i32 {
        self.0.y
    }

    #[inline]
    pub fn set_y(&mut self, y: i32) {
        self.0.y = clamp_position(y);
    }

    #[inline]
    pub fn to_ll(&self) -> sys::SDL_Point {
        self.0
    }
}

impl Default for Point {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

// SAFETY: must be transparent
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct PointF32(sys::SDL_FPoint);

impl PointF32 {
    pub fn new(x: f32, y: f32) -> Self {
        Self(sys::SDL_FPoint {
            x: clamp_position_f32(x),
            y: clamp_position_f32(y),
        })
    }

    #[inline]
    pub fn x(&self) -> f32 {
        self.0.x
    }

    #[inline]
    pub fn set_x(&mut self, x: f32) {
        self.0.x = clamp_position_f32(x);
    }

    #[inline]
    pub fn y(&self) -> f32 {
        self.0.y
    }

    #[inline]
    pub fn set_y(&mut self, y: f32) {
        self.0.y = clamp_position_f32(y);
    }

    #[inline]
    pub fn to_ll(&self) -> sys::SDL_FPoint {
        self.0
    }

    pub(crate) fn as_raw(&self) -> *const sys::SDL_FPoint {
        self as *const Self as *const sys::SDL_FPoint
    }
}

impl Default for PointF32 {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}
