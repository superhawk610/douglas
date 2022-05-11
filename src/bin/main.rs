use douglas::{Command, Config, Program, Renderer};

fn main() {
    let mut config = Config::default();
    App::run(&mut config).unwrap();
}

struct App;

impl Program for App {
    type Message = ();

    fn new() -> Self {
        App
    }

    fn on_event(&mut self, ev: crossterm::event::Event) -> Command<Self::Message> {
        Command::exit()
    }

    fn view(&self) -> String {
        "hello, world!\n".into()
    }
}
