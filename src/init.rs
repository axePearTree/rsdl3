#![allow(warnings)]

use crate::Error;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use core::cell::{Cell, RefCell};
use core::ffi::CStr;
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};
use sdl3_sys as sys;

static IS_SDL_INITIALIZED: AtomicBool = AtomicBool::new(false);

const LOCKED: bool = true;
const UNLOCKED: bool = false;

pub struct Sdl {
    drop: Arc<SdlDrop>,
    video: Arc<AtomicBool>,
    _marker: PhantomData<*const ()>,
}

impl Sdl {
    /// SAFETY: Must be called from the main thread.
    pub unsafe fn init() -> Result<Self, Error> {
        let drop = SdlDrop::new()?;
        Ok(Self {
            drop: Arc::new(drop),
            video: Arc::new(AtomicBool::new(false)),
            _marker: PhantomData,
        })
    }

    pub fn video(&self) -> Result<VideoSubsystem, Error> {
        Subsystem::init(&self.drop, &self.video).map(VideoSubsystem)
    }
}

pub struct VideoSubsystem(Subsystem<{ sys::init::SDL_INIT_VIDEO }>);

struct SdlDrop;

impl SdlDrop {
    /// SAFETY: Must be called from the main thread.
    unsafe fn new() -> Result<Self, Error> {
        let cmp = IS_SDL_INITIALIZED.compare_exchange(
            UNLOCKED,
            LOCKED,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );

        if cmp.is_err() {
            return Err(Error(String::from("SDL is already initialized")));
        }

        if !sys::init::SDL_Init(0) {
            IS_SDL_INITIALIZED.store(false, Ordering::Relaxed);
            return Err(Error::from_sdl());
        }

        Ok(Self)
    }
}

impl Drop for SdlDrop {
    fn drop(&mut self) {
        unsafe { sys::init::SDL_Quit() };
        IS_SDL_INITIALIZED.store(false, Ordering::Relaxed);
    }
}

struct Subsystem<const INIT_FLAG: u32> {
    flag: Arc<AtomicBool>,
    drop: Arc<SdlDrop>,
    _marker: PhantomData<*const ()>,
}

impl<const INIT_FLAG: u32> Subsystem<INIT_FLAG> {
    fn init(drop: &Arc<SdlDrop>, flag: &Arc<AtomicBool>) -> Result<Self, Error> {
        let cmp = flag.compare_exchange(UNLOCKED, LOCKED, Ordering::Relaxed, Ordering::Relaxed);
        if cmp.is_err() {
            return Err(Error(String::from("Subsystem is already initialized")));
        }
        let ret = unsafe { sys::init::SDL_InitSubSystem(INIT_FLAG) };
        if !ret {
            flag.store(false, Ordering::Relaxed);
            return Err(Error::from_sdl());
        }
        Ok(Self {
            flag: Arc::clone(&flag),
            drop: Arc::clone(&drop),
            _marker: PhantomData,
        })
    }
}

impl<const INIT_FLAG: u32> Drop for Subsystem<INIT_FLAG> {
    fn drop(&mut self) {
        unsafe { sys::init::SDL_QuitSubSystem(INIT_FLAG) };
        self.flag.store(false, Ordering::Relaxed);
    }
}
