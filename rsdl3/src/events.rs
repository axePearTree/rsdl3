use crate::Error;
use crate::init::EventsSubsystem;
use crate::sys;
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
            let result = sys::events::SDL_PollEvent(event.as_mut_ptr());
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
    Quit,
    Unknown,
}

impl Event {
    pub fn from_ll(ev: sys::events::SDL_Event) -> Self {
        unsafe {
            match sys::events::SDL_EventType(ev.r#type) {
                sys::events::SDL_EVENT_QUIT => Self::Quit,
                _ => Self::Unknown,
            }
        }
    }
}
