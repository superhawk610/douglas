use crossbeam_channel::{select, tick};
use crossterm::cursor::{self, MoveUp};
use crossterm::style::Print;
use crossterm::terminal;
use crossterm::terminal::{Clear, ClearType};
use crossterm::{execute, queue};
use parking_lot::Mutex;
use std::io::{self, Write as _};
use std::sync::Arc;
use std::time::Duration;

use crate::worker_thread::WorkerThread;

enum Output {
    Stdout(Screen),
    Buffered(WriteBuffer),
}

const DEFAULT_FRAMERATE: usize = 60;
const DEFAULT_BUF_CAPACITY: usize = 1_024;

/// A screen represents a terminal for displaying output, a write buffer
/// for storing UI updates until they're ready to be flushed, and a worker
/// thread responsible for watching the write buffer and flushing it on
/// a regular interval.
struct Screen {
    buffer: Arc<Mutex<WriteBuffer>>,
    handle: io::Stdout,
    worker: Option<WorkerThread>,
}

/// A write buffer is a light wrapper around a `String` that tracks
/// whether it's been written to, allowing readers to skip work if
/// no changes have been written since the previous read.
struct WriteBuffer {
    dirty: bool,
    inner: String,
}

/// A renderer is responsible for displaying a program's view.
pub struct Renderer {
    output: Output,
}

impl Screen {
    fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(WriteBuffer::new())),
            handle: io::stdout(),
            worker: None,
        }
    }
}

impl WriteBuffer {
    fn new() -> Self {
        Self {
            dirty: false,
            inner: String::with_capacity(DEFAULT_BUF_CAPACITY),
        }
    }

    fn output(&self) -> String {
        self.inner.clone()
    }

    fn write(&mut self, string: &str) {
        self.inner.replace_range(.., string);
        self.dirty = true;
    }
}

impl Renderer {
    /// Render output to stdout (the default).
    pub fn stdout() -> Self {
        Self {
            output: Output::Stdout(Screen::new()),
        }
    }

    /// Render output to a string buffer (don't display anything on screen).
    ///
    /// This should generally only be used for testing.
    pub fn buffered() -> Self {
        Self {
            output: Output::Buffered(WriteBuffer::new()),
        }
    }

    /// Retrieve a copy of the renderer's most recent output.
    ///
    /// This should generally only be used for testing.
    pub fn output(&self) -> String {
        match self.output {
            Output::Buffered(ref buf) => buf.output(),
            Output::Stdout(ref screen) => screen.buffer.lock().output(),
        }
    }

    pub(crate) fn init(&mut self) {
        match self.output {
            Output::Stdout(ref mut screen) => {
                // enter raw mode and hide cursor
                terminal::enable_raw_mode().unwrap();
                execute!(screen.handle, cursor::Hide).unwrap();

                // start up worker thread to handle drawing to screen
                screen.worker = Some(spawn_render_thread(&screen.buffer));
            }
            Output::Buffered(_) => {}
        }
    }

    pub(crate) fn close(&mut self) {
        match self.output {
            Output::Stdout(ref mut screen) => {
                // close worker thread
                screen.worker.take().unwrap().close();

                // disable raw mode and re-enable cursor
                terminal::disable_raw_mode().unwrap();
                execute!(screen.handle, cursor::Show).unwrap();
            }
            Output::Buffered(_) => {}
        }
    }

    pub(crate) fn render(&mut self, view: String) {
        match self.output {
            Output::Stdout(ref mut screen) => screen.buffer.lock().write(view.as_ref()),
            Output::Buffered(ref mut buf) => buf.write(view.as_ref()),
        }
    }
}

fn spawn_render_thread(buffer: &Arc<Mutex<WriteBuffer>>) -> WorkerThread {
    let buffer = Arc::clone(buffer);
    WorkerThread::spawn(move |done| {
        let mut stdout = io::stdout();
        let mut last_render: Option<String> = None;
        let frame_delay = (1_000. / DEFAULT_FRAMERATE as f64).floor() as u64;
        let next_frame = tick(Duration::from_millis(frame_delay));

        loop {
            select! {
                recv(next_frame) -> _ => {
                    let mut buffer = buffer.lock();
                    if buffer.dirty {
                        render_to_stdout(&mut stdout, &mut last_render, buffer.inner.as_ref()).unwrap();
                        buffer.dirty = false;
                    }
                }
                recv(done) -> _ => {
                    break;
                }
            }
        }
    })
}

fn render_to_stdout(
    stdout: &mut io::Stdout,
    last_render: &mut Option<String>,
    view: &str,
) -> io::Result<()> {
    // if nothing's changed, do nothing
    if last_render.as_deref() == Some(view) {
        return Ok(());
    }

    let old_lines: Vec<&str> = last_render
        .as_ref()
        .map(|s| s.split("\n").collect())
        .unwrap_or_else(Vec::new);
    let new_lines: Vec<&str> = view.split("\n").collect();

    // clear lines from previous render (if any)
    for (i, line) in old_lines.iter().enumerate().rev() {
        if new_lines.len() <= old_lines.len()
            && (new_lines.len() > i && old_lines.len() > i)
            && new_lines[i] == *line
        {
            // The number of lines we're rendering hasn't increased and
            // this new line is the same as the old line, so do nothing.
        } else {
            queue!(stdout, Clear(ClearType::CurrentLine))?;
        }

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

    // flush to the screen
    *last_render = Some(view.to_owned());
    stdout.flush()
}
