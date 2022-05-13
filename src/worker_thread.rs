use crossbeam_channel::{unbounded, Receiver, Sender};
use std::thread::{self, JoinHandle};

/// Small abstraction around a closure that runs in another thread
/// and waits for a channel to notify that it should stop.
pub struct WorkerThread {
    handle: JoinHandle<()>,
    done: Sender<()>,
}

impl WorkerThread {
    /// Spawn a worker closure in a new thread.
    pub fn new<F>(fun: F) -> Self
    where
        F: FnOnce(Receiver<()>) + Send + 'static,
    {
        let (tx, rx) = unbounded::<()>();
        let handle = thread::spawn(move || fun(rx));

        Self { handle, done: tx }
    }

    /// Notify a running worker that it should stop, then block
    /// until it returns.
    pub fn close(self) {
        self.done.send(()).unwrap();
        self.handle.join().unwrap();
    }
}
