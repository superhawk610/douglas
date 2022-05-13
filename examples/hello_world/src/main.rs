use crossterm::event::Event;
use douglas::{Command, Config, Program};

fn main() {
    App.run(&mut Config::default()).unwrap();
}

struct App;

impl Program for App {
    type Message = ();

    fn on_event(&mut self, _: Event) -> Command<Self::Message> {
        Command::exit()
    }

    fn view(&self) -> String {
        "hello, world!\n".into()
    }
}
