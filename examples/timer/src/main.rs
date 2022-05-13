use crossterm::event::{Event, KeyCode};
use douglas::{Command, Config, Mailbox, Program, Timer};
use std::time::Duration;

struct App {
    count: usize,
    timer: Timer<Message>,
}

#[derive(Clone)]
enum Message {
    Tick,
    Reset,
}

impl App {
    fn new() -> Self {
        Self {
            count: 0,
            timer: Timer::new(Duration::from_millis(1_000), Message::Tick),
        }
    }
}

impl Program for App {
    type Message = Message;

    fn init(&mut self, mailbox: Mailbox<Self::Message>) -> Command<Self::Message> {
        self.timer.start(mailbox);

        Command::none()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Tick => self.count += 1,
            Message::Reset => self.count = 0,
        }

        Command::none()
    }

    fn on_event(event: Event) -> Command<Self::Message> {
        if let Event::Key(event) = event {
            match event.code {
                KeyCode::Char('q') => Command::exit(),
                KeyCode::Char('r') => Command::send(Message::Reset),
                _ => Command::none(),
            }
        } else {
            Command::none()
        }
    }

    fn exit(mut self) {
        self.timer.stop();
    }

    fn view(&self) -> String {
        format!(
            "count: {}\n\
            [r] reset\n\
            [q] quit\n",
            self.count
        )
    }
}

fn main() {
    App::new().run(&mut Config::default()).unwrap();
}
