use crate::events::EventPump;
use crate::sys;
use crate::Error;
use alloc::rc::{Rc, Weak};
use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};

static IS_SDL_INITIALIZED: AtomicBool = AtomicBool::new(false);
const INITIALIZED: bool = true;
const UNINITIALIZED: bool = false;

pub struct Sdl {
    pub(crate) drop: Rc<SdlDrop>,
    audio: Weak<Subsystem<{ sys::SDL_INIT_AUDIO }>>,
    camera: Weak<Subsystem<{ sys::SDL_INIT_CAMERA }>>,
    events: Weak<Subsystem<{ sys::SDL_INIT_EVENTS }>>,
    gamepad: Weak<Subsystem<{ sys::SDL_INIT_GAMEPAD }>>,
    haptic: Weak<Subsystem<{ sys::SDL_INIT_HAPTIC }>>,
    joystick: Weak<Subsystem<{ sys::SDL_INIT_JOYSTICK }>>,
    video: Weak<Subsystem<{ sys::SDL_INIT_VIDEO }>>,
    sensor: Weak<Subsystem<{ sys::SDL_INIT_SENSOR }>>,
    event_pump: Weak<RefCell<EventPump>>,
}

#[allow(unused)]
#[derive(Clone)]
pub struct AudioSubsystem(pub(crate) Rc<Subsystem<{ sys::SDL_INIT_AUDIO }>>);

#[allow(unused)]
#[derive(Clone)]
pub struct CameraSubsystem(pub(crate) Rc<Subsystem<{ sys::SDL_INIT_CAMERA }>>);

#[allow(unused)]
#[derive(Clone)]
pub struct EventsSubsystem {
    pub(crate) subsystem: Rc<Subsystem<{ sys::SDL_INIT_EVENTS }>>,
    pub(crate) event_pump: Rc<RefCell<EventPump>>,
}

#[allow(unused)]
#[derive(Clone)]
pub struct GamepadSubsystem(pub(crate) Rc<Subsystem<{ sys::SDL_INIT_GAMEPAD }>>);

#[allow(unused)]
#[derive(Clone)]
pub struct HapticSubsystem(pub(crate) Rc<Subsystem<{ sys::SDL_INIT_HAPTIC }>>);

#[allow(unused)]
#[derive(Clone)]
pub struct JoystickSubsystem(pub(crate) Rc<Subsystem<{ sys::SDL_INIT_JOYSTICK }>>);

#[allow(unused)]
#[derive(Clone)]
pub struct VideoSubsystem(pub(crate) Rc<Subsystem<{ sys::SDL_INIT_VIDEO }>>);

#[allow(unused)]
#[derive(Clone)]
pub struct SensorSubsystem(pub(crate) Rc<Subsystem<{ sys::SDL_INIT_SENSOR }>>);

impl Sdl {
    /// Initializes SDL.
    /// Will return an [`Error`] if SDL can't be initialized or if SDL is *already* initialized.
    /// SAFETY:
    /// Must be called from the main thread.
    pub unsafe fn init() -> Result<Self, Error> {
        Ok(Self {
            audio: Weak::new(),
            camera: Weak::new(),
            gamepad: Weak::new(),
            events: Weak::new(),
            haptic: Weak::new(),
            joystick: Weak::new(),
            video: Weak::new(),
            sensor: Weak::new(),
            drop: Rc::new(SdlDrop::init()?),
            event_pump: Weak::new(),
        })
    }

    /// Returns a unique instance of the `AudioSubsystem`.
    /// The subsystem will be initialized if it hasn't been yet.
    pub fn audio(&mut self) -> Result<AudioSubsystem, Error> {
        Self::get_or_init(&mut self.audio, &self.drop).map(AudioSubsystem)
    }

    /// Returns a unique instance of the `CameraSubsystem`.
    /// The subsystem will be initialized if it hasn't been yet.
    pub fn camera(&mut self) -> Result<CameraSubsystem, Error> {
        Self::get_or_init(&mut self.camera, &self.drop).map(CameraSubsystem)
    }

    /// Returns a unique instance of the `EventsSubsystem`.
    /// The subsystem will be initialized if it hasn't been yet.
    ///
    /// No custom events will be registered. You can register custom events by calling
    /// [`Sdl::events_custom`].
    pub fn events(&mut self) -> Result<EventsSubsystem, Error> {
        let subsystem = Self::get_or_init(&mut self.events, &self.drop)?;
        let event_pump = match self.event_pump.upgrade() {
            Some(event_pump) => event_pump,
            None => {
                let event_pump = Rc::new(RefCell::new(EventPump));
                self.event_pump = Rc::downgrade(&event_pump);
                event_pump
            }
        };
        Ok(EventsSubsystem {
            subsystem,
            event_pump,
        })
    }

    ///
    /// Returns a unique instance of the `GamepadSubsystem`.
    /// The subsystem will be initialized if it hasn't been yet.
    pub fn gamepad(&mut self) -> Result<GamepadSubsystem, Error> {
        Self::get_or_init(&mut self.gamepad, &self.drop).map(GamepadSubsystem)
    }

    /// Returns a unique instance of the `HapticSubsystem`.
    /// The subsystem will be initialized if it hasn't been yet.
    pub fn haptic(&mut self) -> Result<HapticSubsystem, Error> {
        Self::get_or_init(&mut self.haptic, &self.drop).map(HapticSubsystem)
    }

    /// Returns a unique instance of the `JoystickSubsystem`.
    /// The subsystem will be initialized if it hasn't been yet.
    pub fn joystick(&mut self) -> Result<JoystickSubsystem, Error> {
        Self::get_or_init(&mut self.joystick, &self.drop).map(JoystickSubsystem)
    }

    /// Returns a unique instance of the `VideoSubsystem`.
    /// The subsystem will be initialized if it hasn't been yet.
    pub fn video(&mut self) -> Result<VideoSubsystem, Error> {
        Self::get_or_init(&mut self.video, &self.drop).map(VideoSubsystem)
    }

    /// Returns a unique instance of the `SensorSubsystem`.
    /// The subsystem will be initialized if it hasn't been yet.
    pub fn sensor(&mut self) -> Result<SensorSubsystem, Error> {
        Self::get_or_init(&mut self.sensor, &self.drop).map(SensorSubsystem)
    }

    fn get_or_init<const N: u32>(
        s: &mut Weak<Subsystem<N>>,
        drop: &Rc<SdlDrop>,
    ) -> Result<Rc<Subsystem<N>>, Error> {
        match s.upgrade() {
            Some(subsystem) => Ok(subsystem),
            None => {
                let subsystem = Rc::new(Subsystem::init(drop)?);
                *s = Rc::downgrade(&subsystem);
                Ok(subsystem)
            }
        }
    }
}

pub struct Subsystem<const INIT_FLAG: u32> {
    _drop: Rc<SdlDrop>,
}

impl<const INIT_FLAG: u32> Subsystem<INIT_FLAG> {
    fn init(drop: &Rc<SdlDrop>) -> Result<Self, Error> {
        // Subsystems are refcounted internally by SDL.
        // If you create two instances of the same subsystem with this method, SDL will increase
        // the refcount.
        // Once Drop gets called (calling SDL_QuitSubSystem) the refcount is decremented.
        // So it doesn't matter if a system has already been initialized by Sdl.
        let result = unsafe { sys::SDL_InitSubSystem(INIT_FLAG) };
        if !result {
            return Err(Error::new());
        }
        Ok(Self {
            _drop: Rc::clone(&drop),
        })
    }
}

impl<const INIT_FLAG: u32> Drop for Subsystem<INIT_FLAG> {
    fn drop(&mut self) {
        // This call matches the SDL_InitSubSystem from this instance.
        // SDL refcounts subsystems internally so this should be safe.
        unsafe { sys::SDL_QuitSubSystem(INIT_FLAG) };
    }
}

pub(crate) struct SdlDrop;

impl SdlDrop {
    unsafe fn init() -> Result<Self, Error> {
        let res = IS_SDL_INITIALIZED.compare_exchange(
            UNINITIALIZED,
            INITIALIZED,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
        if res.is_err() {
            return Err(Error::new());
        }

        let result = unsafe { sys::SDL_Init(0) };
        if !result {
            let _ = IS_SDL_INITIALIZED.compare_exchange(
                INITIALIZED,
                UNINITIALIZED,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
            return Err(Error::new());
        }
        Ok(Self)
    }
}

impl Drop for SdlDrop {
    fn drop(&mut self) {
        unsafe { sys::SDL_Quit() };
        IS_SDL_INITIALIZED.store(true, Ordering::Relaxed);
    }
}
