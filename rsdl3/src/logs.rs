use crate::sys;
use alloc::{borrow::ToOwned, ffi::CString, string::String};
use core::fmt::Arguments;

#[macro_export]
macro_rules! log {
    ($fmt:expr) => {
        $crate::logs::log(format_args!($fmt));
    };
    ($fmt:expr, $($args:tt),*) => {
        $crate::logs::log(format_args!($fmt, $($args,)*));
    };
}

#[macro_export]
macro_rules! log_critical {
    ($cat:expr, $fmt:expr) => {
        $crate::logs::log_critical($cat, format_args!($fmt));
    };
    ($cat:expr, $fmt:expr, $($args:tt),*) => {
        $crate::logs::log_critical($cat, format_args!($fmt, $($args,)*));
    };
}

#[macro_export]
macro_rules! log_debug {
    ($cat:expr, $fmt:expr) => {
        $crate::logs::log_debug($cat, format_args!($fmt));
    };
    ($cat:expr, $fmt:expr, $($args:tt),*) => {
        $crate::logs::log_debug($cat, format_args!($fmt, $($args,)*));
    };
}

#[macro_export]
macro_rules! log_info {
    ($cat:expr, $fmt:expr) => {
        $crate::logs::log_info($cat, format_args!($fmt));
    };
    ($cat:expr, $fmt:expr, $($args:tt),*) => {
        $crate::logs::log_info($cat, format_args!($fmt, $($args,)*));
    };
}

#[macro_export]
macro_rules! log_message {
    ($cat:expr, $fmt:expr) => {
        $crate::logs::log_message($cat, format_args!($fmt));
    };
    ($cat:expr, $fmt:expr, $($args:tt),*) => {
        $crate::logs::log_message($cat, format_args!($fmt, $($args,)*));
    };
}

#[macro_export]
macro_rules! log_error {
    ($cat:expr, $fmt:expr) => {
        $crate::logs::log_error($cat, format_args!($fmt));
    };
    ($cat:expr, $fmt:expr, $($args:tt),*) => {
        $crate::logs::log_error($cat, format_args!($fmt, $($args,)*));
    };
}

/// Log a message with [`LogCategory::Application`] and [`LogPriority::INFO`].
///
/// This will panic if `message` contains an interior null byte.
pub fn log(args: Arguments) {
    let message = args_to_c_string(args);
    unsafe {
        sys::SDL_Log(message.as_ptr());
    }
}

/// Reset all priorities to default.
pub fn reset_log_priorities() {
    unsafe { sys::SDL_ResetLogPriorities() };
}

/// Set the priority of a particular log category.
pub fn set_log_priority(category: LogCategory, priority: LogPriority) {
    unsafe {
        sys::SDL_SetLogPriority(category.to_ll() as i32, priority.to_ll());
    }
}

pub fn log_priority(category: LogCategory) -> LogPriority {
    LogPriority(unsafe { sys::SDL_GetLogPriority(category.to_ll() as i32) })
}

/// Log a message with [`LogPriority::CRITICAL`].
///
/// This will panic if `message` contains an interior null byte.
pub fn log_critical(category: LogCategory, args: Arguments) {
    let message = args_to_c_string(args);
    unsafe {
        sys::SDL_LogCritical(category.to_ll() as i32, message.as_ptr());
    }
}

/// Log a message with [`LogPriority::CRITICAL`].
///
/// This will panic if `message` contains an interior null byte.
pub fn log_debug(category: LogCategory, args: Arguments) {
    log_category(category, args, sys::SDL_LogDebug)
}

/// Log a message with [`LogPriority::CRITICAL`].
///
/// This will panic if `message` contains an interior null byte.
pub fn log_error(category: LogCategory, args: Arguments) {
    log_category(category, args, sys::SDL_LogError)
}

/// Log a message with [`LogPriority::CRITICAL`].
///
/// This will panic if `message` contains an interior null byte.
pub fn log_info(category: LogCategory, args: Arguments) {
    log_category(category, args, sys::SDL_LogInfo)
}

/// Log a message with [`LogPriority::CRITICAL`].
///
/// This will panic if `message` contains an interior null byte.
pub fn log_message(category: LogCategory, priority: LogPriority, args: Arguments) {
    let message = args_to_c_string(args);
    unsafe { sys::SDL_LogMessage(category.to_ll() as i32, priority.0, message.as_ptr()) };
}

#[inline]
fn log_category(
    category: LogCategory,
    args: Arguments,
    cb: unsafe extern "C" fn(i32, *const i8, ...),
) {
    let message = args_to_c_string(args);
    unsafe { cb(category.to_ll() as i32, message.as_ptr()) };
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

impl LogCategory {
    #[inline]
    fn to_ll(&self) -> u32 {
        *self as u32
    }
}

fn args_to_c_string(args: Arguments) -> CString {
    use core::fmt::Write;
    let mut buf = String::new();
    buf.write_fmt(args).unwrap();
    CString::new(buf.to_owned().replace("%", "%%")).unwrap()
}
