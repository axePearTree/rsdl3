use crate::init::EventsSubsystem;
use crate::sys;
use crate::Error;
use core::cell::RefMut;
use core::ffi::c_void;
use core::marker::PhantomData;
use core::mem::MaybeUninit;

impl EventsSubsystem {
    /// Returns a mutably borrowed `EventPump`. Only a single instance of
    /// `EventPump` can ever be active.
    ///
    /// This will return an error if the `EventPump` is already borrowed.
    pub fn event_pump(&self) -> Result<RefMut<EventPump>, Error> {
        self.event_pump
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

/// A zero-sized type used for pumping and handling events.
///
/// Only a single instance of this struct can ever be obtained from the [`EventsSubsystem`].
pub struct EventPump;

impl EventPump {
    pub fn pump(&mut self) {
        unsafe { sys::SDL_PumpEvents() }
    }

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

/// Can be used to push [`Event`]s to SDL.
///
/// [`Event`]s pushed to this queue can be consumed by an [`EventPump`].
// This can be shared between threads safely since SDL supports pushing events to the event queue
// from multiple threads. That being said, its' use is still limited to scoped threads, since its'
// lifetime is tied to the EventsSubsystem.
pub struct EventQueue<'a>(PhantomData<&'a ()>);

impl EventQueue<'_> {
    /// Check for the existence of a certain event type in the event queue.
    ///
    /// If you need to check for a range of event types, use [`EventQueue::has_events`] instead.
    pub fn has_event(&self, type_: u32) -> bool {
        unsafe { sys::SDL_HasEvent(type_) }
    }

    /// Check for the existence of a certain event type in the event queue.
    ///
    /// If you need to check for a range of event types, use [`EventQueue::has_events`] instead.
    pub fn has_events(&self, min_type: u32, max_type: u32) -> bool {
        unsafe { sys::SDL_HasEvents(min_type, max_type) }
    }

    /// Query the state of processing events by type.
    pub fn event_enabled(&self, type_: u32) -> bool {
        unsafe { sys::SDL_EventEnabled(type_) }
    }

    /// Clear events of a specific type from the event queue.
    ///
    /// This will unconditionally remove any events from the queue that match `type`. If you need to remove a range
    /// of event types, use [`EventQueue::flush_events`] instead.
    ///
    /// It's also normal to just ignore events you don't care about in your event loop without calling this function.
    ///
    /// This function only affects currently queued events. If you want to make sure that all pending OS events are
    /// flushed, you can call [`EventPump::pump_events`] on the main thread immediately before the flush call.
    ///
    /// If you have user events with custom data that needs to be freed, you should use [`EventPump::peep_events`]
    /// to remove and clean up those events before calling this function.
    pub fn flush_event(&self, type_: u32) {
        unsafe { sys::SDL_FlushEvent(type_) }
    }

    /// Clear events of a range of types from the event queue.
    ///
    /// This will unconditionally remove any events from the queue that are in the range of `minType`
    /// to `maxType`, inclusive. If you need to remove a single event type, use [`EventQueue::flush_event`] instead.
    ///
    /// It's also normal to just ignore events you don't care about in your event loop without calling this function.
    ///
    /// This function only affects currently queued events. If you want to make sure that all pending OS events are
    /// flushed, you can call [`EventPump::pump_events`] on the main thread immediately before the flush call.
    pub fn flush_events(&self, min_type: u32, max_type: u32) {
        unsafe { sys::SDL_FlushEvents(min_type, max_type) }
    }

    /// Add a callback to be triggered when an event is added to the event queue.
    ///
    /// IMPORTANT: The callback is removed once the returning value gets dropped. You must store this into a
    /// long-lived struct for the callback to get called.
    ///
    /// [`EventFilterCallback::callback`] will be called when an event happens, and its return value is ignored.
    ///
    /// If the quit event is generated by a signal (e.g. SIGINT), it will bypass the internal queue and be delivered
    /// to the watch callback immediately, and arrive at the next event poll.
    ///
    /// Note: the callback is called for events posted by the user through [`EventQueue::push_event`], but not for
    /// disabled events, nor for events by a filter callback set with [`EventSubsystem::set_event_filter`], nor for
    /// events posted by the user through [`EventPump::peep_events`].
    pub fn add_event_watch<'a, T: EventFilterCallback>(
        &self,
        watch: &'a T,
    ) -> Result<EventWatch<'a, T>, Error> {
        let callback: sys::SDL_EventFilter = Some(event_filter_marshall::<T>);
        let result = unsafe { sys::SDL_AddEventWatch(callback, watch as *const T as *mut _) };
        if !result {
            return Err(Error::new());
        }
        Ok(EventWatch {
            event_data: watch,
            event_callback: callback,
        })
    }

    /// Set up a filter to process all events before they are added to the internal event queue.
    ///
    /// If you just want to see events without modifying them or preventing them from being queued, you should
    /// use [`EventSubsystem::add_event_watch`] instead.
    ///
    /// If the filter function returns true when called, then the event will be added to the internal queue.
    /// If it returns false, then the event will be dropped from the queue, but the internal state will still
    /// be updated. This allows selective filtering of dynamically arriving events.
    ///
    /// On platforms that support it, if the quit event is generated by an interrupt signal (e.g. pressing Ctrl-C),
    /// it will be delivered to the application at the next event poll.
    ///
    /// Note: Disabled events never make it to the event filter function; see [`EventSubsystem::set_event_enabled`].
    ///
    /// Note: Events pushed onto the queue with [`EventQueue::push_event`] get passed through the event filter, but
    /// events pushed onto the queue with [`EventQueue::peep_events`] do not.
    pub fn set_event_filter<T: EventFilterCallback>(&self, filter: &'static T) {
        let callback: sys::SDL_EventFilter = Some(event_filter_marshall::<T>);
        unsafe { sys::SDL_SetEventFilter(callback, filter as *const T as *mut _) };
    }

    /// Run a specific filter function on the current event queue, removing any events for which the filter returns false.
    ///
    /// See [`EventSubsystem::set_event_filter`] for more information. [`EventsSubsystem::set_event_filter`], this function
    /// does not change the filter permanently, it only uses the supplied filter until this function returns.
    pub fn filter_events<T: EventFilterCallback>(&self, filter: &T) {
        let callback: sys::SDL_EventFilter = Some(event_filter_marshall::<T>);
        unsafe { sys::SDL_FilterEvents(callback, filter as *const T as *mut _) };
    }
}

/// Defines a filter
pub trait EventFilterCallback: Send + Sync {
    fn callback(&self, event: Event) -> bool;
}

pub struct EventWatch<'a, T: EventFilterCallback> {
    event_callback: sys::SDL_EventFilter,
    event_data: &'a T,
}

impl<T: EventFilterCallback> Drop for EventWatch<'_, T> {
    fn drop(&mut self) {
        unsafe {
            sys::SDL_RemoveEventWatch(self.event_callback, self.event_data as *const T as *mut _);
        }
    }
}

unsafe extern "C" fn event_filter_marshall<T: EventFilterCallback>(
    user_data: *mut c_void,
    event: *mut sys::SDL_Event,
) -> bool {
    let f: &T = unsafe { &*(user_data as *const _) };
    let event = Event(unsafe { *event });
    f.callback(event)
}

/// A wrapper on top of [`sys::SDL_Event`].
///
/// To read the contents of the event, convert this type into an [`EventPayload`] by calling
/// [`Event::into_payload`].
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Event(pub(crate) sys::SDL_Event);

impl Event {
    /// Event type id.
    #[inline]
    pub fn event_type(&self) -> u32 {
        unsafe { self.0.type_ }
    }

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
    Camera(CameraEvent),
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

/// Payload of an event tied to a [`crate::camera::Camera`].
#[derive(Copy, Clone, Debug)]
pub enum CameraEvent {
    DeviceApproved,
    DeviceDenied,
}
