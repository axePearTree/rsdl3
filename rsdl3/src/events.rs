use crate::init::EventsSubsystem;
use crate::sys;
use crate::Error;
use core::cell::RefMut;
use core::marker::PhantomData;
use core::mem::MaybeUninit;

impl EventsSubsystem {
    /// Returns a mutably borrowed `EventPump`. Only a single instance of
    /// `EventPump` can ever be active.
    ///
    /// This will return an error if the `EventPump` is already borrowed.
    pub fn event_pump(&self) -> Result<RefMut<EventPump>, Error> {
        self.1
            .try_borrow_mut()
            .map_err(|_| Error::register(c"Event pump already borrowed."))
    }

    /// Returns an [`EventQueue`] that can be used to push events to SDL.
    ///
    /// These events can be consumed by an [`EventPump`].
    pub fn event_queue(&self) -> EventQueue {
        EventQueue(PhantomData)
    }
}

/// Can be used to push [`Event`]s to SDL.
///
/// [`Event`]s pushed to this queue can be consumed by an [`EventPump`].
// This can be shared between threads safely since SDL supports pushing events to the event queue
// from multiple threads. That being said, its' use is still limited to scoped threads, since its'
// lifetime is tied to the EventsSubsystem.
pub struct EventQueue<'a>(PhantomData<&'a ()>);

/// A zero-sized type used for pumping and handling events.
///
/// Only a single instance of this struct can ever be obtained from the [`EventsSubsystem`].
pub struct EventPump;

impl EventPump {
    /// Returns an [`Iterator`] that yields [`Event`]s.
    pub fn poll_iter<'a>(&'a mut self) -> EventPollIter<'a> {
        EventPollIter(PhantomData)
    }
}

/// An [`Iterator`] that yields [`Event`]s.
pub struct EventPollIter<'a>(PhantomData<&'a ()>);

impl Iterator for EventPollIter<'_> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let mut event = MaybeUninit::uninit();
        // SAFETY:
        // To call SDL_PollEvent the event subsystem must be alive.
        // The lifetime of this struct is tied to the EventSubsystem, therefore the subsystem is
        // alive.
        let event = unsafe {
            let result = sys::SDL_PollEvent(event.as_mut_ptr());
            if !result {
                return None;
            }
            event.assume_init()
        };
        Some(Event(event))
    }
}

/// A wrapper on top of [`sys::SDL_Event`].
///
/// To read the contents of the event, convert this type into an [`EventPayload`] by calling
/// [`Event::into_payload`].
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Event(pub(crate) sys::SDL_Event);

impl Event {
    pub fn into_payload(self) -> EventPayload {
        EventPayload::from_ll(self.0)
    }
}

/// Payload of an SDL event.
///
/// The contents of a raw [`sys::SDL_Event`] get parsed and transformed into this value.
#[derive(Copy, Clone, Debug)]
pub enum EventPayload {
    Window(WindowEvent),
    Quit,
    Unknown,
}

impl EventPayload {
    /// Converts a [`sys::SDL_Event`] to an [`Event`].
    /// Returns [`Event::Unknown`] if `event` is not a valid [`sys::SDL_Event`].
    fn from_ll(event: sys::SDL_Event) -> Self {
        unsafe {
            match event.type_ {
                sys::SDL_EventType_SDL_EVENT_WINDOW_MOVED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Moved {
                        x: event.window.data1,
                        y: event.window.data2,
                    },
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_SHOWN => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Shown,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_HIDDEN => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Hidden,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_EXPOSED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Exposed,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                // TODO: check if data fields have new window size.
                sys::SDL_EventType_SDL_EVENT_WINDOW_RESIZED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Resized {
                        w: event.window.data1.max(0) as u32,
                        h: event.window.data2.max(0) as u32,
                    },
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_HIT_TEST => Self::Window(WindowEvent {
                    payload: WindowEventPayload::HitTest,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_OCCLUDED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Occluded,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_RESTORED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Restored,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_DESTROYED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Destroyed,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_MAXIMIZED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Maximized,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_MINIMIZED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Minimized,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_MOUSE_ENTER => Self::Window(WindowEvent {
                    payload: WindowEventPayload::MouseEnter,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_MOUSE_LEAVE => Self::Window(WindowEvent {
                    payload: WindowEventPayload::MouseLeave,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_FOCUS_GAINED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::FocusGained,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_FOCUS_LOST => Self::Window(WindowEvent {
                    payload: WindowEventPayload::FocusLost,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_CLOSE_REQUESTED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::CloseRequested,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_DISPLAY_CHANGED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::DisplayChanged {
                        display_id: event.window.data1.max(0) as u32,
                    },
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_ICCPROF_CHANGED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::IccProfileChanged,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_ENTER_FULLSCREEN => Self::Window(WindowEvent {
                    payload: WindowEventPayload::EnterFullscreen,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_LEAVE_FULLSCREEN => Self::Window(WindowEvent {
                    payload: WindowEventPayload::LeaveFullscreen,
                    timestamp: event.window.timestamp,
                    window_id: event.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_SAFE_AREA_CHANGED => {
                    Self::Window(WindowEvent {
                        payload: WindowEventPayload::SafeAreaChanged,
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    })
                }
                sys::SDL_EventType_SDL_EVENT_WINDOW_HDR_STATE_CHANGED => {
                    Self::Window(WindowEvent {
                        payload: WindowEventPayload::HdrStateChanged,
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    })
                }
                sys::SDL_EventType_SDL_EVENT_WINDOW_METAL_VIEW_RESIZED => {
                    Self::Window(WindowEvent {
                        payload: WindowEventPayload::MetalViewResized,
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    })
                }
                sys::SDL_EventType_SDL_EVENT_WINDOW_PIXEL_SIZE_CHANGED => {
                    Self::Window(WindowEvent {
                        payload: WindowEventPayload::PixelSizeChanged {
                            w: event.window.data1.max(0) as u32,
                            h: event.window.data2.max(0) as u32,
                        },
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    })
                }
                sys::SDL_EventType_SDL_EVENT_WINDOW_DISPLAY_SCALE_CHANGED => {
                    Self::Window(WindowEvent {
                        payload: WindowEventPayload::DisplayScaleChanged,
                        timestamp: event.window.timestamp,
                        window_id: event.window.windowID,
                    })
                }
                sys::SDL_EventType_SDL_EVENT_QUIT => Self::Quit,
                _ => Self::Unknown,
            }
        }
    }
}

/// An event tied to a [`crate::video::Window`].
#[derive(Copy, Clone, Debug)]
pub struct WindowEvent {
    pub payload: WindowEventPayload,
    pub timestamp: u64,
    pub window_id: u32,
}

/// Payload of an event tied to a [`crate::video::Window`].
#[derive(Copy, Clone, Debug)]
pub enum WindowEventPayload {
    Moved { x: i32, y: i32 },
    Shown,
    Hidden,
    MouseEnter,
    MouseLeave,
    Unknown,
    Exposed,
    Resized { w: u32, h: u32 },
    HitTest,
    Occluded,
    Restored,
    Destroyed,
    Maximized,
    Minimized,
    FocusLost,
    FocusGained,
    CloseRequested,
    DisplayChanged { display_id: u32 },
    IccProfileChanged,
    EnterFullscreen,
    LeaveFullscreen,
    SafeAreaChanged,
    HdrStateChanged,
    MetalViewResized,
    PixelSizeChanged { w: u32, h: u32 },
    DisplayScaleChanged,
}
