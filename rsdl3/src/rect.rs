use crate::sys;

const MAX_INT: u32 = (i32::MAX / 2) as u32;

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

    /// Calculate the intersection of a rectangle and line segment. Returns true if there is an
    /// intersection, false otherwise.
    ///
    /// This function is used to clip a line segment to a rectangle. A line segment contained entirely
    /// within the rectangle or that does not intersect will remain unchanged. A line segment that
    /// crosses the rectangle at either or both ends will be clipped to the boundary of the rectangle
    /// and the new coordinates saved in `x1`, `y1`, `x2`, and/or `y2` as necessary.
    ///
    /// * `x1` a referece to the starting X-coordinate of the line.
    /// * `y1` a reference to the starting Y-coordinate of the line.
    /// * `x2` a pointer to the ending X-coordinate of the line.
    /// * `y2` a pointer to the ending Y-coordinate of the line.
    #[inline]
    pub fn line_intersection(
        &self,
        x1: &mut i32,
        y1: &mut i32,
        x2: &mut i32,
        y2: &mut i32,
    ) -> bool {
        unsafe {
            sys::SDL_GetRectAndLineIntersection(
                self.as_raw(),
                x1 as *mut i32,
                y1 as *mut i32,
                x2 as *mut i32,
                y2 as *mut i32,
            )
        }
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

    /// Calculate the intersection of a rectangle and line segment. Returns true if there is an
    /// intersection, false otherwise.
    ///
    /// This function is used to clip a line segment to a rectangle. A line segment contained entirely
    /// within the rectangle or that does not intersect will remain unchanged. A line segment that
    /// crosses the rectangle at either or both ends will be clipped to the boundary of the rectangle
    /// and the new coordinates saved in `x1`, `y1`, `x2`, and/or `y2` as necessary.
    ///
    /// * `x1` a referece to the starting X-coordinate of the line.
    /// * `y1` a reference to the starting Y-coordinate of the line.
    /// * `x2` a pointer to the ending X-coordinate of the line.
    /// * `y2` a pointer to the ending Y-coordinate of the line.
    #[inline]
    pub fn line_intersection(
        &self,
        x1: &mut f32,
        y1: &mut f32,
        x2: &mut f32,
        y2: &mut f32,
    ) -> bool {
        unsafe {
            sys::SDL_GetRectAndLineIntersectionFloat(
                self.as_raw(),
                x1 as *mut f32,
                y1 as *mut f32,
                x2 as *mut f32,
                y2 as *mut f32,
            )
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_bounds_dont_overflow() {
        let x = (MAX_INT + 1) as i32;
        let y = (MAX_INT + 1) as i32;
        let w = MAX_INT + 1;
        let h = MAX_INT + 1;
        assert!(x.overflowing_add(w as i32).1);
        assert!(y.overflowing_add(h as i32).1);
        let rect = Rect::new(x, y, w, h);
        assert_eq!(rect.x(), MAX_INT as i32);
        assert_eq!(rect.y(), MAX_INT as i32);
        assert_eq!(rect.w(), MAX_INT);
        assert_eq!(rect.h(), MAX_INT);
        assert!(!rect.x().overflowing_add(rect.w() as i32).1);
        assert!(!rect.y().overflowing_add(rect.h() as i32).1);

        let rect = Rect::new(MIN_INT - 1, MIN_INT - 1, MAX_INT, MAX_INT);
        assert_eq!(rect.x(), MIN_INT);
        assert_eq!(rect.y(), MIN_INT);
        assert_eq!(rect.w(), MAX_INT);
        assert_eq!(rect.h(), MAX_INT);
        assert!(!rect.x().overflowing_add(rect.w() as i32).1);
    }
}
