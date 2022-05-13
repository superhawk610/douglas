use crossterm::event::{Event, KeyCode};
use douglas::{Command, Config, Program};

struct App {
    count: isize,
}

enum Message {
    Reset,
    Increment,
    Decrement,
}

impl App {
    fn new() -> Self {
        Self { count: 0 }
    }
}

impl Program for App {
    type Message = Message;

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Reset => self.count = 0,
            Message::Increment => self.count += 1,
            Message::Decrement => self.count -= 1,
        }

        Command::none()
    }

    fn on_event(event: Event) -> Command<Self::Message> {
        if let Event::Key(event) = event {
            match event.code {
                KeyCode::Char('q') => Command::exit(),
                KeyCode::Char('r') => Command::send(Message::Reset),
                KeyCode::Up => Command::send(Message::Increment),
                KeyCode::Down => Command::send(Message::Decrement),
                _ => Command::none(),
            }
        } else {
            Command::none()
        }
    }

    fn view(&self) -> String {
        format!(
            "count: {}\n\
            [r]: reset\n\
            [↑]: increment\n\
            [↓]: decrement\n\
            [q]: quit\n",
            self.count
        )
    }
}

fn main() {
    App::new().run(&mut Config::default()).unwrap();
}
