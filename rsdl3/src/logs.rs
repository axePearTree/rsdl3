use crate::{sys, SdlDrop};
use alloc::{rc::Rc, vec::Vec};
use core::{cell::RefCell, ffi::CStr, fmt::Arguments};

#[derive(Clone)]
pub struct Logger {
    pub(crate) internal: Rc<InternalLogger>,
}

impl Logger {
    pub fn priority(&self, category: LogCategory) -> LogPriority {
        // SAFETY: safe to call because SDL is guaranteed to be alive via `internal`.
        LogPriority(unsafe { sys::SDL_GetLogPriority(category as u32 as i32) })
    }

    pub fn set_priority(&mut self, category: LogCategory, priority: impl Into<LogPriority>) {
        let priority: LogPriority = priority.into();
        // SAFETY: safe to call because SDL is guaranteed to be alive via `internal`.
        unsafe {
            sys::SDL_SetLogPriority(category as u32 as i32, priority.0);
        }
    }

    /// Log a message with [`LogCategory::Application`] and [`LogPriority::INFO`].
    // TODO: we should just bite the bullet and return an error here instead of failing silently.
    pub fn log<'a>(&self, args: Arguments<'a>) {
        let buffer = &mut *self.internal.buffer.borrow_mut();
        buffer.clear();
        match buffer::write_to_cstr(buffer, args) {
            Some(buffer) => {
                // SAFETY: safe to call because SDL is guaranteed to be alive via `internal`.
                unsafe { sys::SDL_Log(buffer.as_ptr()) };
            }
            None => {}
        }
    }
}

#[derive(Clone)]
pub(crate) struct InternalLogger {
    _sdl: Rc<SdlDrop>,
    buffer: RefCell<Vec<u8>>,
}

impl InternalLogger {
    pub(crate) fn new(drop: &Rc<SdlDrop>) -> Self {
        Self {
            _sdl: Rc::clone(drop),
            buffer: RefCell::new(Vec::with_capacity(1024)),
        }
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct LogPriority(u32);

impl LogPriority {
    pub const INVALID: Self = Self(sys::SDL_LogPriority_SDL_LOG_PRIORITY_INVALID);
    pub const TRACE: Self = Self(sys::SDL_LogPriority_SDL_LOG_PRIORITY_TRACE);
    pub const VERBOSE: Self = Self(sys::SDL_LogPriority_SDL_LOG_PRIORITY_VERBOSE);
    pub const DEBUG: Self = Self(sys::SDL_LogPriority_SDL_LOG_PRIORITY_DEBUG);
    pub const INFO: Self = Self(sys::SDL_LogPriority_SDL_LOG_PRIORITY_INFO);
    pub const WARN: Self = Self(sys::SDL_LogPriority_SDL_LOG_PRIORITY_WARN);
    pub const ERROR: Self = Self(sys::SDL_LogPriority_SDL_LOG_PRIORITY_ERROR);
    pub const CRITICAL: Self = Self(sys::SDL_LogPriority_SDL_LOG_PRIORITY_CRITICAL);
    pub const COUNT: Self = Self(sys::SDL_LogPriority_SDL_LOG_PRIORITY_COUNT);

    #[inline]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[inline]
    pub fn to_ll(&self) -> u32 {
        self.0
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum LogCategory {
    Application = sys::SDL_LogCategory_SDL_LOG_CATEGORY_APPLICATION,
    Error = sys::SDL_LogCategory_SDL_LOG_CATEGORY_ERROR,
    Assert = sys::SDL_LogCategory_SDL_LOG_CATEGORY_ASSERT,
    System = sys::SDL_LogCategory_SDL_LOG_CATEGORY_SYSTEM,
    Audio = sys::SDL_LogCategory_SDL_LOG_CATEGORY_AUDIO,
    Video = sys::SDL_LogCategory_SDL_LOG_CATEGORY_VIDEO,
    Render = sys::SDL_LogCategory_SDL_LOG_CATEGORY_RENDER,
    Input = sys::SDL_LogCategory_SDL_LOG_CATEGORY_INPUT,
    Test = sys::SDL_LogCategory_SDL_LOG_CATEGORY_TEST,
    Gpu = sys::SDL_LogCategory_SDL_LOG_CATEGORY_GPU,
    Reserved2 = sys::SDL_LogCategory_SDL_LOG_CATEGORY_RESERVED2,
    Reserved3 = sys::SDL_LogCategory_SDL_LOG_CATEGORY_RESERVED3,
    Reserved4 = sys::SDL_LogCategory_SDL_LOG_CATEGORY_RESERVED4,
    Reserved5 = sys::SDL_LogCategory_SDL_LOG_CATEGORY_RESERVED5,
    Reserved6 = sys::SDL_LogCategory_SDL_LOG_CATEGORY_RESERVED6,
    Reserved7 = sys::SDL_LogCategory_SDL_LOG_CATEGORY_RESERVED7,
    Reserved8 = sys::SDL_LogCategory_SDL_LOG_CATEGORY_RESERVED8,
    Reserved9 = sys::SDL_LogCategory_SDL_LOG_CATEGORY_RESERVED9,
    Reserved10 = sys::SDL_LogCategory_SDL_LOG_CATEGORY_RESERVED10,
    Custom = sys::SDL_LogCategory_SDL_LOG_CATEGORY_CUSTOM,
}

mod buffer {
    use alloc::vec::Vec;
    use core::ffi::CStr;
    use core::fmt::{Arguments, Write};

    pub fn write_to_cstr<'a, 'b>(buf: &'a mut Vec<u8>, args: Arguments<'b>) -> Option<&'a CStr> {
        LogBufferWriter(buf).write_fmt(args).ok()?;
        // SAFETY: By now we've made sure there are no nul-terminators inside the string.
        Some(unsafe { CStr::from_bytes_with_nul_unchecked(buf) })
    }

    struct LogBufferWriter<'a>(&'a mut Vec<u8>);

    impl core::fmt::Write for LogBufferWriter<'_> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            LogBufferStrWriter(self.0).write_str(s)
        }

        fn write_fmt(&mut self, args: Arguments<'_>) -> core::fmt::Result {
            LogBufferStrWriter(self.0).write_fmt(args)?;
            self.0.push(b'\0');
            Ok(())
        }
    }

    struct LogBufferStrWriter<'a>(&'a mut Vec<u8>);

    impl core::fmt::Write for LogBufferStrWriter<'_> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            for byte in s.bytes() {
                if byte == b'\0' {
                    return Err(core::fmt::Error);
                }
            }
            self.0.extend_from_slice(s.as_bytes());
            Ok(())
        }
    }
}
