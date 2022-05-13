use douglas::{Command, Config, Mailbox, Program};

fn main() {
    App.run(&mut Config::default()).unwrap();
}

struct App;

impl Program for App {
    type Message = ();

    fn init(&mut self, _: Mailbox<Self::Message>) -> Command<Self::Message> {
        Command::exit()
    }

    fn view(&self) -> String {
        "hello, world!\n".into()
    }
}
