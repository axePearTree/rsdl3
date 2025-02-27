use crate::init::EventsSubsystem;
use crate::sys;
use crate::Error;
use core::cell::RefMut;
use core::marker::PhantomData;
use core::mem::MaybeUninit;

impl EventsSubsystem {
    /// Returns a mutably borrowed [`EventPump`].
    /// Only a single instance of [`EventPump`] can ever be active.
    /// This will return an error if the [`EventPump`] is already borrowed.
    pub fn event_pump(&self) -> Result<RefMut<EventPump>, Error> {
        self.1
            .try_borrow_mut()
            .map_err(|_| Error::new("Event pump can't be borrowed more than once at a time."))
    }

    pub fn event_queue(&self) -> EventQueue {
        EventQueue(PhantomData)
    }
}

// This can be shared between threads safely since SDL supports pushing events to the event queue
// from multiple threads. That being said, its' use is still limited to scoped threads, since its'
// lifetime is tied to the EventsSubsystem.
pub struct EventQueue<'a>(PhantomData<&'a ()>);

/// A zero-sized type used for pumping and handling events.
/// Only a single instance of this struct can ever be obtained from the [`EventsSubsystem`].
pub struct EventPump;

impl EventPump {
    pub fn poll_iter<'a>(&'a mut self) -> EventPollIter<'a> {
        EventPollIter(PhantomData)
    }
}

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
        Some(Event::from_ll(event))
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    Window(WindowEvent),
    Quit,
    Unknown,
}

impl Event {
    fn from_ll(ev: sys::SDL_Event) -> Self {
        unsafe {
            match ev.type_ {
                sys::SDL_EventType_SDL_EVENT_WINDOW_MOVED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Moved {
                        x: ev.window.data1,
                        y: ev.window.data2,
                    },
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_SHOWN => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Shown,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_HIDDEN => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Hidden,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_EXPOSED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Exposed,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                // TODO: check if data fields have new window size.
                sys::SDL_EventType_SDL_EVENT_WINDOW_RESIZED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Resized {
                        w: ev.window.data1.max(0) as u32,
                        h: ev.window.data2.max(0) as u32,
                    },
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_HIT_TEST => Self::Window(WindowEvent {
                    payload: WindowEventPayload::HitTest,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_OCCLUDED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Occluded,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_RESTORED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Restored,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_DESTROYED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Destroyed,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_MAXIMIZED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Maximized,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_MINIMIZED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::Minimized,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_MOUSE_ENTER => Self::Window(WindowEvent {
                    payload: WindowEventPayload::MouseEnter,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_MOUSE_LEAVE => Self::Window(WindowEvent {
                    payload: WindowEventPayload::MouseLeave,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_FOCUS_GAINED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::FocusGained,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_FOCUS_LOST => Self::Window(WindowEvent {
                    payload: WindowEventPayload::FocusLost,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_CLOSE_REQUESTED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::CloseRequested,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_DISPLAY_CHANGED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::DisplayChanged {
                        display_id: ev.window.data1.max(0) as u32,
                    },
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_ICCPROF_CHANGED => Self::Window(WindowEvent {
                    payload: WindowEventPayload::IccProfileChanged,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_ENTER_FULLSCREEN => Self::Window(WindowEvent {
                    payload: WindowEventPayload::EnterFullscreen,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                sys::SDL_EventType_SDL_EVENT_WINDOW_LEAVE_FULLSCREEN => Self::Window(WindowEvent {
                    payload: WindowEventPayload::LeaveFullscreen,
                    timestamp: ev.window.timestamp,
                    window_id: ev.window.windowID,
                }),
                // TODO: check if data fields have new safe area data.
                sys::SDL_EventType_SDL_EVENT_WINDOW_SAFE_AREA_CHANGED => {
                    Self::Window(WindowEvent {
                        payload: WindowEventPayload::SafeAreaChanged,
                        timestamp: ev.window.timestamp,
                        window_id: ev.window.windowID,
                    })
                }
                sys::SDL_EventType_SDL_EVENT_WINDOW_HDR_STATE_CHANGED => {
                    Self::Window(WindowEvent {
                        payload: WindowEventPayload::HdrStateChanged,
                        timestamp: ev.window.timestamp,
                        window_id: ev.window.windowID,
                    })
                }
                sys::SDL_EventType_SDL_EVENT_WINDOW_METAL_VIEW_RESIZED => {
                    Self::Window(WindowEvent {
                        payload: WindowEventPayload::MetalViewResized,
                        timestamp: ev.window.timestamp,
                        window_id: ev.window.windowID,
                    })
                }
                sys::SDL_EventType_SDL_EVENT_WINDOW_PIXEL_SIZE_CHANGED => {
                    Self::Window(WindowEvent {
                        payload: WindowEventPayload::PixelSizeChanged {
                            w: ev.window.data1.max(0) as u32,
                            h: ev.window.data2.max(0) as u32,
                        },
                        timestamp: ev.window.timestamp,
                        window_id: ev.window.windowID,
                    })
                }
                sys::SDL_EventType_SDL_EVENT_WINDOW_DISPLAY_SCALE_CHANGED => {
                    Self::Window(WindowEvent {
                        payload: WindowEventPayload::DisplayScaleChanged,
                        timestamp: ev.window.timestamp,
                        window_id: ev.window.windowID,
                    })
                }
                sys::SDL_EventType_SDL_EVENT_QUIT => Self::Quit,
                _ => Self::Unknown,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct WindowEvent {
    pub payload: WindowEventPayload,
    pub timestamp: u64,
    pub window_id: u32,
}

#[derive(Clone, Debug)]
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
