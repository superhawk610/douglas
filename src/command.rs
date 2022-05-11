pub enum Command<T> {
    Exit,
    None,
    Single(Action<T>),
    Batch(Vec<Action<T>>),
}

pub enum Action<T> {
    Sync(Box<dyn Fn() -> T>),
    // TODO: async w/ future?
}

impl<T> Action<T> {
    pub fn run(self) -> T {
        match self {
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

    pub fn single<F>(fun: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
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
