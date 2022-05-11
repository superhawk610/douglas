use douglas::{Command, Config, Program};

fn main() {
    let mut config = Config::default();
    App.run(&mut config).unwrap();
}

struct App;

impl Program for App {
    type Message = ();

    fn on_event(&mut self, _ev: crossterm::event::Event) -> Command<Self::Message> {
        Command::exit()
    }

    fn view(&self) -> String {
        "hello, world!\n".into()
    }
}
