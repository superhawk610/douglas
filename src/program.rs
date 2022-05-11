use crossterm::event::{self, Event as TermEvent};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::command::Command;
use crate::config::Config;

pub enum Event<T> {
    Exit,
    User(T),
    Term(TermEvent),
}

pub trait Program: Sized {
    type Message: Send + 'static;

    fn init(&mut self) -> Command<Self::Message> {
        Command::none()
    }

    fn on_event(&mut self, _ev: TermEvent) -> Command<Self::Message> {
        Command::none()
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> String;

    fn run(mut self, config: &mut Config) -> io::Result<()> {
        let (tx, rx) = mpsc::channel::<Event<Self::Message>>();
        let (done_tx, done_rx) = mpsc::channel();

        // spin up key/mouse event listener
        let event_handle = {
            let tx = tx.clone();
            thread::spawn(move || loop {
                if event::poll(Duration::from_millis(100)).unwrap() {
                    tx.send(Event::Term(event::read().unwrap())).unwrap();
                }

                if done_rx.try_recv().is_ok() {
                    break;
                }
            })
        };

        // initialize the renderer
        config.renderer.init();

        // if an initial command is provided, run it
        let init_cmd = self.init();
        run_command(&tx, init_cmd);

        while let Ok(message) = rx.recv() {
            // update the view
            config.renderer.render(self.view()).unwrap();

            // handle the event/message
            match message {
                Event::Exit => {
                    break;
                }
                Event::Term(ev) => run_command(&tx, self.on_event(ev)),
                Event::User(msg) => run_command(&tx, self.update(msg)),
            }
        }

        done_tx.send(()).unwrap();
        event_handle.join().unwrap();
        config.renderer.close();

        Ok(())
    }
}

fn run_command<T>(tx: &mpsc::Sender<Event<T>>, command: Command<T>) {
    if matches!(command, Command::Exit) {
        tx.send(Event::Exit).unwrap();
        return;
    }

    for action in command.actions() {
        let msg = action.run();
        tx.send(Event::User(msg)).unwrap();
    }
}
