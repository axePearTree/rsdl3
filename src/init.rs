#![allow(warnings)]

use alloc::borrow::ToOwned;
use alloc::rc::{Rc, Weak};
use alloc::string::String;
use core::cell::RefCell;
use core::ffi::CStr;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};

static IS_SDL_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug)]
pub struct Error(pub(crate) String);

impl Error {
    pub fn from_sdl() -> Self {
        unsafe {
            let err = sdl3_sys::error::SDL_GetError();
            Error(CStr::from_ptr(err as *const _).to_str().unwrap().to_owned())
        }
    }
}

/// An entry point for SDL subsystem initialization functions.
pub struct Sdl {
    inner: Rc<RefCell<SdlInner>>,
}

impl Sdl {
    /// Safety: Must be called from the main thread.
    pub unsafe fn init() -> Result<Self, Error> {
        let inner = SdlInner::init()?;
        Ok(Self {
            inner: Rc::new(RefCell::new(inner))
        })
    }

    pub fn video(&self) -> Result<VideoSubsystem, Error> {
        match self.inner.borrow().video.0.upgrade() {
            Some(video) => {
                // subsystem is already initialized
                return Ok(VideoSubsystem(Rc::clone(&video)));
            }
            _ => {}
        }
        let subsystem = Rc::new(Subsystem::init(&self.inner)?);
        self.inner.borrow_mut().video = VideoSubsystemWeak(Rc::downgrade(&subsystem));
        Ok(VideoSubsystem(subsystem))
    }

    pub fn audio(&self) -> Result<AudioSubsystem, Error> {
        match self.inner.borrow().audio.0.upgrade() {
            Some(audio) => {
                // subsystem is already initialized
                return Ok(AudioSubsystem(Rc::clone(&audio)));
            }
            _ => {}
        }
        let subsystem = Rc::new(Subsystem::init(&self.inner)?);
        self.inner.borrow_mut().audio = AudioSubsystemWeak(Rc::downgrade(&subsystem));
        Ok(AudioSubsystem(subsystem))
    }
}

pub struct VideoSubsystem(Rc<Subsystem<{ sdl3_sys::init::SDL_INIT_VIDEO }>>);

pub struct AudioSubsystem(Rc<Subsystem<{ sdl3_sys::init::SDL_INIT_AUDIO }>>);

// This struct keeps track of which subsystems are currently alive via Weak refcount.
// Initializing a subsystem more than once will just return a new referece to the subsystem.
struct SdlInner {
    video: VideoSubsystemWeak,
    audio: AudioSubsystemWeak,
}

impl SdlInner {
    unsafe fn init() -> Result<Self, Error> {
        if IS_SDL_INITIALIZED.load(Ordering::Acquire) {
            return Err(Error(String::from("SDL is already initialized.")));
        }
        IS_SDL_INITIALIZED.store(true, Ordering::Release);
        let result = sdl3_sys::init::SDL_Init(0);
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(SdlInner {
            video: VideoSubsystemWeak(Weak::new()),
            audio: AudioSubsystemWeak(Weak::new()),
        })
    }
}

impl Drop for SdlInner {
    fn drop(&mut self) {
        if !IS_SDL_INITIALIZED.load(Ordering::Acquire) {
            return;
        }
        unsafe { sdl3_sys::init::SDL_Quit() };
        IS_SDL_INITIALIZED.store(false, Ordering::Release);
    }
}

struct VideoSubsystemWeak(Weak<Subsystem<{ sdl3_sys::init::SDL_INIT_VIDEO }>>);

struct AudioSubsystemWeak(Weak<Subsystem<{ sdl3_sys::init::SDL_INIT_AUDIO }>>);

#[derive(Clone)]
struct Subsystem<const FLAG: u32 = 0> {
    _sdl: Rc<RefCell<SdlInner>>,
    _m: PhantomData<&'static mut ()>,
}

impl<const FLAG: u32> Subsystem<FLAG> {
    fn init(sdl: &Rc<RefCell<SdlInner>>) -> Result<Self, Error> {
        unsafe {
            let result = sdl3_sys::init::SDL_InitSubSystem(FLAG);
            if !result {
                return Err(Error::from_sdl());
            }
        }
        Ok(Self {
            _sdl: Rc::clone(&sdl),
            _m: PhantomData,
        })
    }
}

impl<const FLAG: u32> Drop for Subsystem<FLAG> {
    fn drop(&mut self) {
        // If a subsystem is alive then SdlInner is alive.
        unsafe { sdl3_sys::init::SDL_QuitSubSystem(FLAG) }
    }
}
