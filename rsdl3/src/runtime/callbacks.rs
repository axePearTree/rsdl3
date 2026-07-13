use alloc::boxed::Box;
use core::ffi::{c_char, c_int, c_void};
pub use rsdl3_macros::application;

use crate::{events::Event, runtime::Args, Error};

pub enum ControlFlow {
    Continue,
    Success,
}

impl ControlFlow {
    const fn to_ll(self) -> crate::sys::SDL_AppResult {
        match self {
            Self::Continue => crate::sys::SDL_AppResult_SDL_APP_CONTINUE,
            Self::Success => crate::sys::SDL_AppResult_SDL_APP_SUCCESS,
        }
    }

    const fn from_ll(value: crate::sys::SDL_AppResult) -> Self {
        match value {
            crate::sys::SDL_AppResult_SDL_APP_SUCCESS => Self::Success,
            _ => Self::Continue,
        }
    }
}

fn to_ll(result: Result<ControlFlow, Error>) -> crate::sys::SDL_AppResult {
    match result {
        Ok(control_flow) => control_flow.to_ll(),
        Err(_) => crate::sys::SDL_AppResult_SDL_APP_FAILURE,
    }
}

fn from_ll(value: crate::sys::SDL_AppResult) -> Result<ControlFlow, Error> {
    match value {
        crate::sys::SDL_AppResult_SDL_APP_FAILURE => Err(Error::new()),
        _ => Ok(ControlFlow::from_ll(value)),
    }
}

pub trait Callbacks: Sized + 'static {
    fn init(_args: Args) -> Result<Self, Error>;

    fn iterate(&mut self) -> Result<ControlFlow, Error> {
        Ok(ControlFlow::Continue)
    }

    fn event(&mut self, _event: Event) -> Result<ControlFlow, Error> {
        Ok(ControlFlow::Continue)
    }

    fn quit(self, _result: Result<ControlFlow, Error>) {}
}

pub fn init_callbacks<T: Callbacks>(
    appstate: *mut *mut c_void,
    argc: c_int,
    argv: *mut *mut c_char,
) -> crate::sys::SDL_AppResult {
    match T::init(Args::from_raw(argc, argv)) {
        Ok(app) => {
            unsafe {
                *appstate = Box::into_raw(Box::new(app)).cast();
            }
            ControlFlow::Continue.to_ll()
        }
        Err(_) => crate::sys::SDL_AppResult_SDL_APP_FAILURE,
    }
}

pub unsafe fn iterate_callbacks<T: Callbacks>(appstate: *mut c_void) -> crate::sys::SDL_AppResult {
    let Some(app) = (unsafe { appstate.cast::<T>().as_mut() }) else {
        return crate::sys::SDL_AppResult_SDL_APP_FAILURE;
    };
    to_ll(app.iterate())
}

pub unsafe fn event_callbacks<T: Callbacks>(
    appstate: *mut c_void,
    event: *mut crate::sys::SDL_Event,
) -> crate::sys::SDL_AppResult {
    let Some(app) = (unsafe { appstate.cast::<T>().as_mut() }) else {
        return crate::sys::SDL_AppResult_SDL_APP_FAILURE;
    };
    let Some(event) = (unsafe { event.as_ref() }) else {
        return crate::sys::SDL_AppResult_SDL_APP_FAILURE;
    };
    to_ll(app.event(Event(*event)))
}

pub unsafe fn quit_callbacks<T: Callbacks>(
    appstate: *mut c_void,
    result: crate::sys::SDL_AppResult,
) {
    if appstate.is_null() {
        return;
    }

    unsafe { Box::from_raw(appstate.cast::<T>()) }.quit(from_ll(result));
}
