use crossbeam_channel::{select, tick};
use std::time::Duration;

use crate::mailbox::Mailbox;
use crate::worker_thread::WorkerThread;

/// A simple timer that sends a given message on a regular interval.
pub struct Timer<Message: Clone + Send + 'static> {
    worker: Option<WorkerThread>,
    message: Message,
    interval: Duration,
}

impl<T: Clone + Send + 'static> Timer<T> {
    /// Create a timer that sends the given message at the given interval.
    ///
    /// The first message will be sent after `duration`.
    pub fn new(interval: Duration, message: T) -> Self {
        Self {
            worker: None,
            message,
            interval,
        }
    }

    /// Start the timer. Does nothing if the timer's already started.
    ///
    /// Make sure to call `stop` when your program exits.
    pub fn start(&mut self, mailbox: Mailbox<T>) {
        if self.worker.is_some() {
            return;
        }

        let interval = self.interval;
        let message = self.message.clone();
        let worker = WorkerThread::new(move |done| {
            let ticker = tick(interval);

            loop {
                let message = message.clone();
                select! {
                    recv(ticker) -> _ => {
                        if mailbox.send(message).is_err() {
                            break;
                        }
                    }
                    recv(done) -> _ => {
                        break;
                    }
                }
            }
        });

        self.worker = Some(worker);
    }

    /// Stop the timer. Does nothing if the timer hasn't started.
    ///
    /// This should usually be called from `exit` in your program.
    pub fn stop(&mut self) {
        if let Some(worker) = self.worker.take() {
            worker.close();
        }
    }
}
