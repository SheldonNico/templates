mod app;

use app::App;

use log::{debug, warn};
use std::path::{Path, PathBuf};

use std::io::{self, Read, Write};
use std::thread;
use std::sync::mpsc;
use tui::{ backend::CrosstermBackend, Terminal };

use crossterm::{
    event::{self, Event as CEvent, KeyEvent, MouseEvent, KeyCode, KeyModifiers, MouseButton},
    execute,
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

    Unsupported(String),
}

impl From<CEvent> for Event {
    fn from(event: CEvent) -> Self {
        match event {
            CEvent::Key(KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::NONE })    => Self::CharKey(c),
            CEvent::Key(KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::SHIFT })   => Self::CharKey(c),
            CEvent::Key(KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::CONTROL }) => Self::CtrlKey(c),

            CEvent::Key(KeyEvent { code: KeyCode::Up, modifiers: KeyModifiers::NONE })         => Self::Up,
            CEvent::Key(KeyEvent { code: KeyCode::Down, modifiers: KeyModifiers::NONE })       => Self::Down,
            CEvent::Key(KeyEvent { code: KeyCode::Left, modifiers: KeyModifiers::NONE })       => Self::Left,
            CEvent::Key(KeyEvent { code: KeyCode::Right, modifiers: KeyModifiers::NONE })      => Self::Right,
            CEvent::Key(KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE })      => Self::Enter,
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

fn main() {
    main_().unwrap();
}

fn main_() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    execute!(stdout, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    let events = Events::new(std::time::Duration::from_millis(250));
    let mut app = App::new();
    loop {
        //terminal.set_cursor(0, 0)?;
        terminal.draw(|f| app.draw(f))?;

        match events.next()? {
            Event::CharKey('q') => {
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                execute!(terminal.backend_mut(), event::DisableMouseCapture)?;
                terminal.show_cursor()?;
                break;
            }
            e => app.on_event(e),
        }
    }
    //terminal.show_cursor()?;
    Ok(())
}
