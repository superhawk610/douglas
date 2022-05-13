mod command;
mod config;
mod mailbox;
mod program;
mod renderer;
mod timer;
mod worker_thread;

pub use command::Command;
pub use config::Config;
pub use mailbox::Mailbox;
pub use program::Program;
pub use renderer::Renderer;
pub use timer::Timer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        struct App;

        impl Program for App {
            type Message = ();

            fn init(&mut self, _: Mailbox<Self::Message>) -> Command<Self::Message> {
                Command::exit()
            }

            fn update(&mut self, _: Self::Message) -> Command<Self::Message> {
                // make sure all commands compile
                Command::batch(vec![
                    Command::none(),
                    Command::send(()),
                    Command::sync(|| {}),
                    Command::exit(),
                ])
            }

            fn view(&self) -> String {
                "hello, world!\n".into()
            }
        }

        let mut config = Config::buffered();
        App.run(&mut config).unwrap();
        assert_eq!(config.renderer.output(), "hello, world!\n");
    }
}
