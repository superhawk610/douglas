use crossterm::cursor::MoveUp;
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal;
use crossterm::terminal::{Clear, ClearType};
use std::io::{self, Write as _};

enum Output {
    Stdout(Screen),
    Buffered(String),
}

struct Screen {
    handle: io::Stdout,
    lines_rendered: u32,
    last_render: Option<String>,
}

pub struct Renderer {
    output: Output,
}

impl Screen {
    fn new() -> Self {
        Self {
            handle: io::stdout(),
            lines_rendered: 0,
            last_render: None,
        }
    }
}

impl Renderer {
    pub fn stdout() -> Self {
        Self {
            output: Output::Stdout(Screen::new()),
        }
    }

    pub fn buffered() -> Self {
        Self {
            output: Output::Buffered(String::new()),
        }
    }

    pub fn output(&self) -> &str {
        if let Output::Buffered(ref buf) = self.output {
            buf
        } else {
            panic!("can only retrieve output when buffered")
        }
    }

    pub(crate) fn init(&self) {
        match self.output {
            Output::Stdout(_) => terminal::enable_raw_mode().unwrap(),
            Output::Buffered(_) => {}
        }
    }

    pub(crate) fn close(&self) {
        match self.output {
            Output::Stdout(_) => terminal::disable_raw_mode().unwrap(),
            Output::Buffered(_) => {}
        }
    }

    pub(crate) fn render(&mut self, view: String) -> io::Result<()> {
        match self.output {
            Output::Stdout(ref mut screen) => render_to_stdout(screen, view),
            Output::Buffered(ref mut buf) => render_buffered(buf, view),
        }
    }
}

// TODO: intelligent diffing (don't queue unnecessary commands)
fn render_to_stdout(screen: &mut Screen, view: String) -> io::Result<()> {
    let stdout = &mut screen.handle;
    let old_lines: Vec<&str> = screen
        .last_render
        .as_ref()
        .map(|s| s.split("\n").collect())
        .unwrap_or_else(Vec::new);
    let new_lines: Vec<&str> = view.split("\n").collect();

    // clear lines from previous render (if any)
    for (i, _line) in old_lines.iter().rev().enumerate() {
        queue!(stdout, Clear(ClearType::CurrentLine))?;
        if i > 0 {
            queue!(stdout, MoveUp(1))?;
        }
    }

    // render new lines
    for (i, line) in new_lines.iter().enumerate() {
        queue!(stdout, Print(line))?;
        if i < new_lines.len() - 1 {
            queue!(stdout, Print("\r\n"))?;
        }
    }

    screen.lines_rendered = new_lines.len() as _;
    screen.last_render = Some(view);
    stdout.flush()
}

fn render_buffered(buf: &mut String, view: String) -> io::Result<()> {
    buf.replace_range(.., view.as_ref());
    Ok(())
}
