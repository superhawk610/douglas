pub enum Command<Message> {
    Exit,
    None,
    Single(Action<Message>),
    Batch(Vec<Action<Message>>),
}

pub enum Action<Message> {
    Send(Message),
    Sync(Box<dyn Fn() -> Message>),
    // TODO: async w/ future?
}

impl<T> Action<T> {
    pub fn run(self) -> T {
        match self {
            Action::Send(msg) => msg,
            Action::Sync(fun) => fun(),
        }
    }
}

impl<T> Command<T> {
    pub fn exit() -> Self {
        Command::Exit
    }

    pub fn none() -> Self {
        Command::None
    }

    pub fn send(message: T) -> Self {
        Command::Single(Action::Send(message))
    }

    pub fn sync(fun: impl Fn() -> T + 'static) -> Self {
        Command::Single(Action::Sync(Box::new(fun)))
    }

    pub fn batch(commands: impl IntoIterator<Item = Command<T>>) -> Self {
        Command::Batch(commands.into_iter().flat_map(|c| c.actions()).collect())
    }

    pub(crate) fn actions(self) -> Vec<Action<T>> {
        match self {
            Command::Single(action) => vec![action],
            Command::Batch(actions) => actions,
            _ => Vec::new(),
        }
    }
}
