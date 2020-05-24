#![feature(concat_idents)]
mod app;

use app::{App, Status};

use log::{debug, warn};
use std::path::{Path, PathBuf};

use std::io::{self, Read, Write};
use std::sync::{Arc, RwLock, Mutex};
use std::thread;
use std::sync::mpsc;
use tui::{ backend::{CrosstermBackend, Backend}, Terminal };
use fern::colors::{Color, ColoredLevelConfig};

use crossterm::{
    event::{self, Event as CEvent, KeyEvent, MouseEvent, KeyCode, KeyModifiers, MouseButton},
    execute,
    ExecutableCommand,
    cursor,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Event {
    Tick,
    CharKey(char),
    CtrlKey(char),
    Up,
    Down,
    Left,
    Right,
    ScrollUp(u16, u16),
    ScrollDown(u16, u16),
    Press(u16, u16),
    Enter,
    Backspace,
    Esc,

    Unsupported(String),
}

impl From<CEvent> for Event {
    fn from(event: CEvent) -> Self {
        match event {
            CEvent::Key(KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::NONE })    => Self::CharKey(c),
            CEvent::Key(KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::SHIFT })   => Self::CharKey(c),
            CEvent::Key(KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::CONTROL }) => Self::CtrlKey(c),

            CEvent::Key(KeyEvent { code: KeyCode::Up, modifiers: _ })                          => Self::Up,
            CEvent::Key(KeyEvent { code: KeyCode::Down, modifiers: _ })                        => Self::Down,
            CEvent::Key(KeyEvent { code: KeyCode::Left, modifiers: _ })                        => Self::Left,
            CEvent::Key(KeyEvent { code: KeyCode::Right, modifiers: _ })                       => Self::Right,
            CEvent::Key(KeyEvent { code: KeyCode::Enter, modifiers: _ })                       => Self::Enter,
            CEvent::Key(KeyEvent { code: KeyCode::Backspace, modifiers: _ })                   => Self::Backspace,
            CEvent::Key(KeyEvent { code: KeyCode::Esc, modifiers: _ })                         => Self::Esc,
            CEvent::Mouse(MouseEvent::Down(MouseButton::Left, xpos, ypos, KeyModifiers::NONE)) =>
                Self::Press(xpos, ypos),
            CEvent::Mouse(MouseEvent::ScrollDown(xpos, ypos, KeyModifiers::NONE))              =>
                Self::ScrollDown(xpos, ypos),
            CEvent::Mouse(MouseEvent::ScrollUp(xpos, ypos, KeyModifiers::NONE))                =>
                Self::ScrollUp(xpos, ypos),
            e                                                                                  =>
                Self::Unsupported(format!("{:?}", e)),
        }
    }
}

pub struct Events {
    rx: mpsc::Receiver<Event>,
}

impl Events {
    pub fn new(tick_rate: std::time::Duration) -> Self {
        assert!(tick_rate >= std::time::Duration::from_millis(100), "tick_rate to small");
        let (tx, rx) = mpsc::channel();
        {
            let tx = tx.clone();
            thread::spawn(move || {
                let mut last_tick = std::time::Instant::now();
                loop {
                    if event::poll(tick_rate-last_tick.elapsed()).unwrap() {
                        if let Ok(event) = event::read() {
                            tx.send(event.into()).unwrap();
                        }
                    }
                    if last_tick.elapsed() >= tick_rate {
                        let _ = tx.send(Event::Tick);
                        last_tick = std::time::Instant::now();
                    }
                }
            })
        };
        Self { rx }
    }

    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.rx.recv()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    let level: log::LevelFilter = std::env::var("RUST_LOG").unwrap_or("debug".to_string()).parse()?;

    //let colors = ColoredLevelConfig::new().info(Color::Green);

    let (tx, rx) = mpsc::channel();
    fern::Dispatch::new()
        .level(level)
        // .chain(
        //     fern::Dispatch::new()
        //     .format(move |out, message, record| {
        //         out.finish(format_args!(
        //                 "[{} {:<5} {}] {}",
        //                 chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S.%3f"),
        //                 colors.color(record.level()),
        //                 record.target(),
        //                 message
        //         ))
        //     })
        //     .chain(std::io::stdout())
        // )
        .chain(
            fern::Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                        "[{} {:<5} {}] {}",
                        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S"),
                        record.level(),
                        record.target(),
                        message
                ))
            })
            .chain(fern::log_file("output.log")?)
            .chain(tx)
        )
        .apply()?;

    enable_raw_mode()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.backend_mut().execute(EnterAlternateScreen)?;
    terminal.backend_mut().execute(event::EnableMouseCapture)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    let events = Events::new(std::time::Duration::from_millis(250));
    let mut app = App::new(rx); let mut cursor_show = false; let mut app_state_insert = false;
    loop {
        terminal.draw(|f| app.draw(f))?;

        match events.next()? {
            Event::CharKey('q') if !app_state_insert => {
                disable_raw_mode()?;
                terminal.backend_mut().execute(LeaveAlternateScreen)?;
                terminal.backend_mut().execute(event::DisableMouseCapture)?;
                terminal.show_cursor()?;
                break;
            }
            e => app.on_event(e),
        }
        match app.status {
            Status::Insert(ref m) => {
                if !app_state_insert {
                    app_state_insert = true;
                }
                let rect = terminal.backend_mut().size()?;
                if !cursor_show {
                    terminal.backend_mut().execute(cursor::Show)?;
                    cursor_show = true;
                }
                terminal.backend_mut().execute(cursor::MoveTo(m.len() as u16 + 1, rect.height))?;
            },
            _ => {
                if app_state_insert {
                    app_state_insert = false;
                }
                if cursor_show {
                    terminal.backend_mut().execute(cursor::Hide)?;
                }
            }
        }
    }
    Ok(())
}
