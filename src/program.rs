use crossbeam_channel::{unbounded, Sender};
use crossterm::event::{self, Event as TermEvent};
use std::io;
use std::sync::Arc;
use std::time::Duration;

use crate::command::Command;
use crate::config::Config;
use crate::mailbox::Mailbox;
use crate::worker_thread::WorkerThread;

pub(crate) enum Event<T> {
    Exit,
    User(T),
    Term(TermEvent),
}

/// A program describes a terminal application capable of generating
/// and responding to messages, as well as rendering its UI to a string.
///
/// A douglas program consists of 3 main ingredients:
///
/// 1. `init` initialize your state model
/// 2. `update` update your state model in response to messages
/// 3. `view` render your program's UI
///
/// At minimum, you must implement `view`.
pub trait Program: Sized {
    type Message: Send + 'static;

    /// Initialize your program's state.
    fn init(&mut self, _mailbox: Mailbox<Self::Message>) -> Command<Self::Message> {
        Command::none()
    }

    /// Perform any cleanup before your application exits.
    fn exit(self) {}

    /// Respond to any terminal events (mouse, keyboard) by generating messages.
    fn on_event(_ev: TermEvent) -> Command<Self::Message> {
        Command::none()
    }

    /// Update your program's state in response to messages.
    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    /// Render your program's UI.
    fn view(&self) -> String;

    /// Run your application.
    fn run(mut self, config: &mut Config) -> io::Result<()> {
        let (tx, rx) = unbounded::<Event<Self::Message>>();

        // spin up key/mouse event listener
        let event_listener = {
            let tx = tx.clone();
            WorkerThread::spawn(move |done| loop {
                if event::poll(Duration::from_millis(100)).unwrap() {
                    tx.send(Event::Term(event::read().unwrap())).unwrap();
                }

                if done.try_recv().is_ok() {
                    break;
                }
            })
        };

        // initialize the renderer
        config.renderer.init();

        // if an initial command is provided, run it
        let tx = Arc::new(tx);
        let mailbox = Mailbox::new(&tx);
        let init_cmd = self.init(mailbox);
        run_command(&tx, init_cmd);

        loop {
            // update the view
            config.renderer.render(self.view());

            // handle the event/message
            match rx.recv().unwrap() {
                Event::Term(ev) => run_command(&tx, <Self as Program>::on_event(ev)),
                Event::User(msg) => run_command(&tx, self.update(msg)),
                Event::Exit => {
                    break;
                }
            }
        }

        // clean up
        event_listener.close();
        config.renderer.close();
        self.exit();

        Ok(())
    }
}

fn run_command<T>(tx: &Arc<Sender<Event<T>>>, command: Command<T>) {
    if matches!(command, Command::Exit) {
        tx.send(Event::Exit).unwrap();
        return;
    }

    for action in command.actions() {
        let msg = action.run();
        tx.send(Event::User(msg)).unwrap();
    }
}
