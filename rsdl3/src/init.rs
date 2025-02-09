#![allow(warnings)]

use crate::Error;
use alloc::string::String;
use alloc::sync::Arc;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use sdl3_sys as sys;

static IS_SDL_INITIALIZED: AtomicBool = AtomicBool::new(false);
const INITIALIZED: bool = true;
const UNINITIALIZED: bool = false;

#[derive(Clone)]
pub struct Sdl(Arc<SdlDrop>, PhantomData<*const ()>);

pub struct AudioSubsystem(pub(crate) Arc<Subsystem<{ sys::init::SDL_INIT_AUDIO }>>);

pub struct CameraSubsystem(pub(crate) Arc<Subsystem<{ sys::init::SDL_INIT_CAMERA }>>);

pub struct EventsSubsystem(pub(crate) Arc<Subsystem<{ sys::init::SDL_INIT_EVENTS }>>);

pub struct GamepadSubsystem(pub(crate) Arc<Subsystem<{ sys::init::SDL_INIT_GAMEPAD }>>);

pub struct HapticSubsystem(pub(crate) Arc<Subsystem<{ sys::init::SDL_INIT_HAPTIC }>>);

pub struct JoystickSubsystem(pub(crate) Arc<Subsystem<{ sys::init::SDL_INIT_JOYSTICK }>>);

pub struct VideoSubsystem(pub(crate) Arc<Subsystem<{ sys::init::SDL_INIT_VIDEO }>>);

pub struct SensorSubsystem(pub(crate) Arc<Subsystem<{ sys::init::SDL_INIT_SENSOR }>>);

impl Sdl {
    /// SAFETY: This must be called from the main thread.
    pub unsafe fn init() -> Result<Self, Error> {
        let res = IS_SDL_INITIALIZED.compare_exchange(
            UNINITIALIZED,
            INITIALIZED,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
        if res.is_err() {
            return Err(Error(String::from("SDL is already initialized.")));
        }

        let result = unsafe { sys::init::SDL_Init(0) };
        if !result {
            let _ = IS_SDL_INITIALIZED.compare_exchange(
                INITIALIZED,
                UNINITIALIZED,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
            return Err(Error::from_sdl());
        }

        Ok(Self(Arc::new(SdlDrop), PhantomData))
    }

    pub fn audio(&self) -> Result<AudioSubsystem, Error> {
        Subsystem::init(&self.0).map(Arc::new).map(AudioSubsystem)
    }

    pub fn camera(&self) -> Result<CameraSubsystem, Error> {
        Subsystem::init(&self.0).map(Arc::new).map(CameraSubsystem)
    }

    pub fn events(&self) -> Result<EventsSubsystem, Error> {
        Subsystem::init(&self.0).map(Arc::new).map(EventsSubsystem)
    }

    pub fn gamepad(&self) -> Result<GamepadSubsystem, Error> {
        Subsystem::init(&self.0).map(Arc::new).map(GamepadSubsystem)
    }

    pub fn haptic(&self) -> Result<HapticSubsystem, Error> {
        Subsystem::init(&self.0).map(Arc::new).map(HapticSubsystem)
    }

    pub fn joystick(&self) -> Result<JoystickSubsystem, Error> {
        Subsystem::init(&self.0).map(Arc::new).map(JoystickSubsystem)
    }

    pub fn video(&self) -> Result<VideoSubsystem, Error> {
        Subsystem::init(&self.0).map(Arc::new).map(VideoSubsystem)
    }

    pub fn sensor(&self) -> Result<SensorSubsystem, Error> {
        Subsystem::init(&self.0).map(Arc::new).map(SensorSubsystem)
    }
}

pub(crate) struct Subsystem<const INIT_FLAG: u32>(Arc<SdlDrop>, PhantomData<*const ()>);

impl<const INIT_FLAG: u32> Subsystem<INIT_FLAG> {
    fn init(sdl: &Arc<SdlDrop>) -> Result<Self, Error> {
        // Subsystems are refcounted internally by SDL.
        // If you create two instances of the same subsystem with this method, SDL will increase
        // the refcount.
        // Once Drop gets called (calling SDL_QuitSubSystem) the refcount is decremented.
        // So it doesn't matter if a system has already been initialized by Sdl.
        // This will just become a separate reference to that same subsystem.
        let result = unsafe { sys::init::SDL_InitSubSystem(INIT_FLAG) };
        if !result {
            return Err(Error::from_sdl());
        }
        Ok(Self(Arc::clone(sdl), PhantomData))
    }
}

impl<const INIT_FLAG: u32> Drop for Subsystem<INIT_FLAG> {
    fn drop(&mut self) {
        // This call matches the SDL_InitSubSystem from this instance.
        // SDL refcounts subsystems internally so this should be safe.
        unsafe { sys::init::SDL_QuitSubSystem(INIT_FLAG) };
    }
}

struct SdlDrop;

impl Drop for SdlDrop {
    fn drop(&mut self) {
        unsafe { sys::init::SDL_Quit() };
        IS_SDL_INITIALIZED.store(true, Ordering::Relaxed);
    }
}
