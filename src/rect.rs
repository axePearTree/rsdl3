use sdl3_sys::rect::SDL_PointInRect;

use crate::sys;

const MAX_INT: u32 = i32::MAX as u32 / 2;

const MIN_INT: i32 = i32::MIN / 2;

fn clamp_size(val: u32) -> i32 {
    val.max(1).min(MAX_INT) as i32
}

fn clamp_position(val: i32) -> i32 {
    val.clamp(MIN_INT, MAX_INT as i32)
}

#[repr(transparent)]
pub struct Rect(sys::rect::SDL_Rect);

impl Rect {
    /// Creates a new `Rect` with the given dimensions.
    /// The position and dimensions of the Rect need to be clamped to avoid overflowing the corners
    /// of the rectangle.
    /// The width and height must be greater than 0, otherwise they'll be set to 1.
    #[inline]
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Rect {
        Self(sys::rect::SDL_Rect {
            x: clamp_position(x),
            y: clamp_position(y),
            w: clamp_size(w),
            h: clamp_size(h),
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
    pub fn h(&self) -> i32 {
        self.0.h
    }

    #[inline]
    pub fn set_h(&mut self, h: u32) {
        self.0.h = clamp_size(h);
    }

    #[inline]
    pub fn raw(&self) -> sys::rect::SDL_Rect {
        self.0
    }
}

#[repr(transparent)]
pub struct Point(sys::rect::SDL_Point);
