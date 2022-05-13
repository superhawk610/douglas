use crossbeam_channel::Sender;
use std::sync::{Arc, Weak};

use crate::program::Event;

/// A mailbox can deliver messages to a running douglas program.
/// This is typically used to send messages outside of the normal
/// application lifecycle, e.g. on a recurring basis or in response
/// to external stimuli.
#[derive(Clone)]
pub struct Mailbox<Message> {
    sender: Weak<Sender<Event<Message>>>,
}

impl<T> Mailbox<T> {
    pub(crate) fn new(sender: &Arc<Sender<Event<T>>>) -> Self {
        Self {
            sender: Arc::downgrade(sender),
        }
    }

    /// Send a message to the mailbox. If the program associated with
    /// the mailbox has exited, this will return an error.
    pub fn send(&self, message: T) -> Result<(), &'static str> {
        self.sender
            .upgrade()
            .ok_or("mailbox has been closed")?
            .send(Event::User(message))
            .unwrap();

        Ok(())
    }
}
