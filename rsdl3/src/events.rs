use crate::{init::EventsSubsystem, Error};
use core::cell::RefMut;
use core::marker::PhantomData;

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

/// A zero-sized type used for pumping and handling events.
/// Only a single instance of this struct can ever be obtained from the [`EventsSubsystem`].
pub struct EventPump;

impl EventPump {
    pub fn poll_iter(&mut self) -> impl Iterator<Item = Event> {
        EventPollIter
    }
}

// This can be shared between threads safely since SDL supports pushing events to the event queue
// from multiple threads. That being said, it's use is still limited to scoped threads, since its'
// lifetime is tied to the EventsSubsystem.
pub struct EventQueue<'a>(PhantomData<&'a ()>);

pub struct Event;

struct EventPollIter;

impl Iterator for EventPollIter {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
