#![allow(dead_code)]
use crate::Event;

use crossterm::{
    event::{self, Event as CEvent, KeyEvent, MouseEvent, KeyCode},
};
use std::collections::HashMap;
use std::ffi::c_void;
use std::time::{Duration, Instant};
use std::sync::mpsc;
use std::any::Any;
use log::{info, debug, warn};

use tui::{
    backend::Backend,
    terminal::Frame,

    style::{Color, Modifier, Style},
    widgets::*,
    layout::*,
    Terminal,
};

pub trait InteractiveWidget2 {
    fn select_up2(&mut self) -> u32 { 0 } // up k
    fn select_down2(&mut self) { } // up k
}

pub trait InteractiveWidget {
    fn select_up(&mut self) { } // up k
    fn select_down(&mut self) { } // down j
    fn select_first(&mut self) {  } // gg
    fn select_last(&mut self) {  } // G
    fn select_page_up(&mut self) {  } // Ctrl-U
    fn select_page_down(&mut self) {  } // Ctrl-D
    fn select_on_key(&mut self) {  }
    fn click(&mut self, _x: u16, _y: u16) {  } // relative click
    fn draw<B: Backend>(&mut self, _f: &mut Frame<B>, area: Rect, name: &'static str, is_active: bool) { }
    fn selectable(&self) -> bool { true }
}

macro_rules! dispatch {
    ($ptr:expr, $target:ty , $fun:ident, ($($arg:expr ),*)) => { {
        let val: &mut $target = unsafe { &mut *($ptr as *mut $target) };
        val.$fun($($arg),*)

    } };
    ($ptr: expr, $tag:expr, [ $( ($kind:expr, $target:ty) ),* ], $fun:ident, [ $($arg:expr),* ]) => {
        dispatch!(@call_tuple $ptr, $tag, [ $(($kind, $target)),* ], $fun, ($($arg),*))
    };
    (@call_tuple $ptr: expr, $tag: expr, [ $( ($kind:expr, $target:ty) ),* ], $fun:ident, $tuple:tt) => {
        match $tag.as_ref() {
            $(
                $kind => { dispatch!($ptr, $target, $fun, $tuple) },
            )*
            _ => panic!("select_up not implemented for {}", $tag),
        }
    };
}

#[derive(Clone, Debug)]
pub enum Status {
    Normal,
    Insert(String),
    WaitG,
}

impl Default for Status {
    fn default() -> Self {
        Status::Normal
    }
}

macro_rules! impl_app {
    ($cls:ty, $map:expr, [ $( ($name:ident, $kind:ty) ),* ] ) => { paste::item! {
        impl $cls {
        $(
            fn $name(&self) -> &$kind {
                let ptr: *const c_void = *$map.get(stringify!($name)).unwrap();
                unsafe { & *(ptr as *const $kind) }
            }

            fn [<$name _mut>](&mut self) -> &mut $kind {
                let ptr: *mut c_void = *$map.get(stringify!($name)).unwrap();
                unsafe { &mut *(ptr as *mut $kind) }
            }
        )*

        unsafe fn drop_map(&mut self) {
            $(
                let ptr: *mut c_void = *$map.get(stringify!($name)).unwrap();
                let ptr: *mut $kind = unsafe { ptr as *mut $kind };
                { ptr.drop_in_place(); }
            )*
        }
        }
    } }
}

macro_rules! impl_app_dispatch {
    (ret $self:ident, $map:expr, $focus:expr, [ $( ($tag:expr, $getter:ident) ),* ], $fun:ident, [ $($arg:expr),* ]) => {
        impl_app_dispatch!(@ret_call_tuple $self, $map, $focus, [ $( ($tag, $getter) ),* ], $fun, ( $($arg),* ))
    };
    (@ret_call_tuple $self:ident, $map:expr, $focus:expr, [ $( ($tag:expr, $getter:ident) ),* ], $fun:ident, $tuple:tt) => {
        match $focus.as_ref() {
            $( $tag => impl_app_dispatch!(@call $self.$getter(), $fun, $tuple), )*
            _ => unreachable!()
        }
    };
    (@call $at:expr, $fun:ident, ( $($arg:expr),* )) => {
       $at.$fun( $($arg),* )
    };
    ($self:ident, $map:expr, $focus:expr, [ $( ($tag:expr, $getter:ident) ),* ], $fun:ident, [ $($arg:expr),* ]) => {
        impl_app_dispatch!(@call_tuple $self, $map, $focus, [ $( ($tag, $getter) ),* ], $fun, ( $($arg),* ))
    };
    (@call_tuple $self:ident, $map:expr, $focus:expr, [ $( ($tag:expr, $getter:ident) ),* ], $fun:ident, $tuple:tt) => {
        match $focus.as_ref() {
            $( $tag => { impl_app_dispatch!(@call $self.$getter(), $fun, $tuple); }, )*
            _ => {  }
        }
    };
}

pub struct App {
    widgets: HashMap<&'static str, *mut c_void>,
    widgets_location: HashMap<&'static str, Rect>,

    pub curr_widget: Option<&'static str>,
    pub status: Status,
    stack_kevent_time: Instant,

    receiver: mpsc::Receiver<String>,
}

impl_app!(
    App, self.widgets,
    [
        (assets, IEmpty), (positions, IEmpty), (orders, IEmpty),
        (trades, ITable), (logs, IParagraph)
    ]
);

/// this is where you set layout and event handler.
/// just use 'static element. It's clear and easy to access.
impl InteractiveWidget for App {
    fn select_page_up(&mut self) {
        if let Some(curr_tag) = self.curr_widget {
            impl_app_dispatch!(self, self.widgets, curr_tag, [
                ("assets", assets_mut), ("positions", positions_mut), ("orders", orders_mut),
                ("trades", trades_mut), ("logs", logs_mut)
            ], select_page_up, [  ])
        }
    }

    fn select_page_down(&mut self) {
        if let Some(curr_tag) = self.curr_widget {
            impl_app_dispatch!(self, self.widgets, curr_tag, [
                ("assets", assets_mut), ("positions", positions_mut), ("orders", orders_mut),
                ("trades", trades_mut), ("logs", logs_mut)
            ], select_page_down, [  ])
        }
    }

    fn select_up(&mut self) {
        if let Some(curr_tag) = self.curr_widget {
            impl_app_dispatch!(self, self.widgets, curr_tag, [
                ("assets", assets_mut), ("positions", positions_mut), ("orders", orders_mut),
                ("trades", trades_mut), ("logs", logs_mut)
            ], select_up, [  ])
        }
    }

    fn select_down(&mut self) {
        if let Some(curr_tag) = self.curr_widget {
            impl_app_dispatch!(self, self.widgets, curr_tag, [
                ("assets", assets_mut), ("positions", positions_mut), ("orders", orders_mut),
                ("trades", trades_mut), ("logs", logs_mut)
            ], select_down, [  ])
        }
    }

    fn select_last(&mut self) {
        if let Some(curr_tag) = self.curr_widget {
            impl_app_dispatch!(self, self.widgets, curr_tag, [
                ("assets", assets_mut), ("positions", positions_mut), ("orders", orders_mut),
                ("trades", trades_mut), ("logs", logs_mut)
            ], select_last, [  ])
        }
    }

    fn select_first(&mut self) {
        if let Some(curr_tag) = self.curr_widget {
            impl_app_dispatch!(self, self.widgets, curr_tag, [
                ("assets", assets_mut), ("positions", positions_mut), ("orders", orders_mut),
                ("trades", trades_mut), ("logs", logs_mut)
            ], select_first, [  ])
        }
    }

    fn click(&mut self, xpos: u16, ypos: u16, ) {
        for (name, rect) in self.widgets_location.iter() {
            if xpos > rect.left() && xpos < rect.right() && ypos < rect.bottom() && ypos > rect.top() {
                if self.curr_widget == Some(name) {
                    info!("Ori posi {} {} {:?}", xpos, ypos, rect);
                    let xpos = xpos - rect.left(); let ypos = ypos - rect.top();
                    info!("Ori posi {} {}", xpos, ypos);

                    impl_app_dispatch!(self, self.widgets, name, [
                        ("assets", assets_mut), ("positions", positions_mut), ("orders", orders_mut),
                        //("trades", trades_mut),
                        ("logs", logs_mut)
                    ], click, [ xpos, ypos  ])
                } else {
                    self.curr_widget = Some(name);
                }
                break;
            }
        }

        info!("change foucs pos: {} {} => select: {:?}", xpos, ypos, self.curr_widget);
    }
}

impl App {
    pub fn new(rx: mpsc::Receiver<String>) -> Self {
        let mut slf = Self {
            widgets           : Default::default(),
            widgets_location  : Default::default(),
            curr_widget       : Default::default(),
            status            : Default::default(),
            stack_kevent_time : Instant::now(),
            receiver          : rx,
        };

        slf.add_widget("assets", IEmpty::default());
        slf.add_widget("positions", IEmpty::default());
        slf.add_widget("orders", IEmpty::default());

        let mut trades = ITable::new();
        trades.add_column(
            "code", Style::default(), 6,
            vec!["000001", "000001", "000002"],
            |_, _| { Style::default().fg(Color::White) }
        );
        trades.add_column(
            "exchange", Style::default(), 8,
            vec!["SZSE", "SSE", "SZSE"],
            |_, _| { Style::default().fg(Color::White) }
        );
        trades.add_column(
            "price", Style::default(), 8,
            vec!["12.23", "12.45", "13.45"],
            |_, _| { Style::default().fg(Color::White) }
        );
        trades.add_column(
            "volume", Style::default(), 10,
            vec!["100", "700", "10000"],
            |_, _| { Style::default().fg(Color::White) }
        );
        trades.add_column(
            "direction", Style::default(), 9,
            vec!["Buy", "Sell", "Buy"],
            |r: &str, s: bool| {
                match (r, s) {
                    ("Buy", false) => Style::default().fg(Color::Green),
                    ("Sell", false)   => Style::default().fg(Color::Cyan),
                    _                  => Style::default().fg(Color::White)
                }
            }
        );
        trades.add_column(
            "status", Style::default(), 8,
            vec!["Cancel", "Pending", "Error"],
            |r: &str, s: bool| {
                match (r, s) {
                    ("Pending", false) => Style::default().fg(Color::Magenta),
                    ("Error", false)   => Style::default().fg(Color::Red),
                    _                  => Style::default().fg(Color::White)
                }
            }
        );

        slf.add_widget("trades", trades);
        slf.add_widget("logs", IParagraph::new());
        slf
    }

    fn add_widget<T: InteractiveWidget>(&mut self, name: &'static str, widget: T) {
        assert!(!self.widgets.contains_key(name), "widget with same name");
        self.widgets_location.insert(name, Rect::default());
        self.widgets.insert(name, Box::into_raw(Box::new(widget)) as *mut c_void);
    }

    fn move_left(&mut self) {
        if let Some(ref mut curr) = self.curr_widget {
            match *curr {
                "Trades"    => { *curr = "Logs" }
                "Logs"      => { *curr = "Trades" }
                "Assets"    => { *curr = "Orders" }
                "Positions" => { *curr = "Assets" }
                "Orders"    => { *curr = "Positions" }
                _           => {  }
            }
        }
    }

    fn move_right(&mut self) {
        if let Some(ref mut curr) = self.curr_widget {
            match *curr {
                "Trades"    => { *curr = "Logs" }
                "Logs"      => { *curr = "Trades" }
                "Assets"    => { *curr = "Positions" }
                "Positions" => { *curr = "Orders" }
                "Orders"    => { *curr = "Assets" }
                _           => {  }
            }
        }

    }

    fn move_up(&mut self) {
        if let Some(ref mut curr) = self.curr_widget {
            match *curr {
                "Assets"    => { *curr = "Trades" }
                "Positions" => { *curr = "Trades" }
                "Orders"    => { *curr = "Logs" }

                "Trades"    => { *curr = "Assets" }
                "Logs"      => { *curr = "Positions" }
                _           => {  }
            }
        }
    }

    fn move_down(&mut self) {
        if let Some(ref mut curr) = self.curr_widget {
            match *curr {
                "Assets"    => { *curr = "Trades" }
                "Positions" => { *curr = "Trades" }
                "Orders"    => { *curr = "Logs" }

                "Trades"    => { *curr = "Assets" }
                "Logs"      => { *curr = "Positions" }
                _           => {  }
            }
        }
    }

    pub fn draw<B: Backend>(&mut self, mut f: Frame<B>) {
        let height = f.size().height;
        if height < 2 {
            // give us a lager screen
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Length(height - 1), Constraint::Length(1)].as_ref())
            .split(f.size());

        {
            let msg = match self.status {
                Status::Insert(ref input) => { vec![
                    Text::styled(":", Style::default().fg(Color::White)),
                    Text::styled(input, Style::default().fg(Color::White))
                ] },
                _ => {vec![
                    Text::styled("help: ", Style::default().fg(Color::Blue)),
                    Text::raw("hjkl: left/down/up/right, gg: first, G: last"),
                ] }
            };
            f.render_widget(Paragraph::new(msg.iter()), chunks[1])
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(chunks[0]);

        {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [Constraint::Percentage(20), Constraint::Percentage(40), Constraint::Percentage(40)].as_ref())
                .split(chunks[0]);

            let is_select = self.curr_widget.map(|t| t == "assets").unwrap_or(false);
            self.assets_mut().draw(&mut f, chunks[0], "Assets", is_select);
            *self.widgets_location.get_mut("assets").unwrap() = chunks[0];

            let is_select = self.curr_widget.map(|t| t == "positions").unwrap_or(false);
            self.positions_mut().draw(&mut f, chunks[1], "Positions", is_select);
            *self.widgets_location.get_mut("positions").unwrap() = chunks[1];

            let is_select = self.curr_widget.map(|t| t == "orders").unwrap_or(false);
            self.orders_mut().draw(&mut f, chunks[2], "Orders", is_select);
            *self.widgets_location.get_mut("orders").unwrap() = chunks[2];
        }

        {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(chunks[1]);

            let is_select = self.curr_widget.map(|t| t == "trades").unwrap_or(false);
            self.trades_mut().draw(&mut f, chunks[0], "Trades", is_select);
            *self.widgets_location.get_mut("trades").unwrap() = chunks[0];

            let is_select = self.curr_widget.map(|t| t == "logs").unwrap_or(false);
            self.logs_mut().draw(&mut f, chunks[1], "Logs", is_select);
            *self.widgets_location.get_mut("logs").unwrap() = chunks[1];
        }
    }

    fn refresh_log(&mut self) {
        while let Ok(msg) = self.receiver.try_recv() {
            let style = if msg.find("ERROR").is_some() {
                Style::default().fg(Color::White)
            } else if msg.find("WARN").is_some() {
                Style::default().fg(Color::Yellow)
            } else if msg.find("INFO").is_some() {
                Style::default().fg(Color::Green)
            } else if msg.find("DEBUG").is_some() {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            };

            self.logs_mut().add_styled(msg, style);
            self.logs_mut().select_last();
        }
    }

    fn timeout_waitg(&mut self) {
        if self.stack_kevent_time.elapsed() > Duration::from_secs(2) {
            self.status = Status::Normal
        }
    }

    pub fn on_event(&mut self, event: Event) {
        match (self.status.clone(), event.clone()) {
            (Status::Normal, Event::CtrlKey('l'))      => { self.move_right(); },
            (Status::Normal, Event::CtrlKey('h'))      => { self.move_left(); },
            (Status::Normal, Event::CtrlKey('j'))      => { self.move_down(); },
            (Status::Normal, Event::CtrlKey('k'))      => { self.move_up(); },
            (Status::Normal, Event::CtrlKey('u'))      => { self.select_page_up(); },
            (Status::Normal, Event::CtrlKey('d'))      => { self.select_page_down(); },
            (Status::Normal, Event::CharKey(':'))      => { self.status = Status::Insert("".into()); },
            (Status::Normal, Event::CharKey('G'))      => { self.select_last(); },
            (Status::Normal, Event::CharKey('g'))      => { self.status = Status::WaitG; },
            (Status::WaitG,  Event::CharKey('g'))      => { self.select_first(); self.status = Status::Normal; },
            (Status::Normal, Event::CharKey('j'))      => { self.select_down(); },
            (Status::Normal, Event::CharKey('k'))      => { self.select_up(); },
            (Status::WaitG,  Event::Tick)              => { self.timeout_waitg(); self.refresh_log(); },
            (Status::Normal, Event::Tick)              => { self.refresh_log(); }
            (Status::Insert(_), Event::Tick)           => { self.refresh_log() }
            (Status::Normal, Event::Up)                => { self.select_up() },
            (Status::Normal, Event::Down)              => { self.select_down() },
            (Status::Normal, Event::ScrollUp(_, _))    => { self.select_up(); },
            (Status::Normal, Event::ScrollDown(_, _))  => { self.select_down(); },
            (Status::Normal, Event::Press(xpos, ypos)) => { self.click(xpos, ypos); },
            (Status::Insert(mut m), Event::CharKey(c)) => { m.push(c); self.status = Status::Insert(m); },
            (Status::Insert(m), Event::Enter)          => { info!("got input {}", m); self.status = Status::Normal; },
            (Status::Insert(mut m), Event::Backspace)  => { m.pop(); self.status = Status::Insert(m); },
            (Status::Insert(_), Event::CtrlKey('w'))   => { self.status = Status::Insert("".into()); },
            (Status::Insert(_), Event::Esc)            => { self.status = Status::Normal; },
            _                                          => {
                //self.debug(format!("Got key: {:?} status: {:?}", event, self.status));
                debug!("Got key: {:?} status: {:?}", event, self.status);
            }
        }
    }
}

/// content contain lines, and each line must be trimmed, they can not render space at startup or
/// end
#[derive(Default)]
pub struct IParagraph {
    content      : Vec<Vec<(String, Style)>>,
    idx_page     : usize,  // max item reached.
    window       : Rect,
}
impl IParagraph {
    pub fn new() -> Self {
        let slf = Self::default();
        slf
    }

    pub fn add<S: AsRef<str>>(&mut self, item: S) {
        self.add_styled(item, Style::default());
    }

    pub fn add_styled<S: AsRef<str>>(&mut self, item: S, style: Style) {
        let lines: Vec<_> = item.as_ref().lines().collect();
        if self.content.len() == 0 { self.content.push(vec![]); }
        if lines.len() == 0 { return; }
        for (idx, line) in lines[..lines.len()-1].iter().enumerate() {
            let res = (line.to_string(), style);
            if idx == 0 {
                self.content.last_mut().unwrap().push(res);
            } else {
                self.content.push(vec![res]);
            }
        }

        let res = (lines[lines.len()-1].to_string(), style);
        if lines.len() == 1 {
            self.content.last_mut().unwrap().push(res);
        } else {
            self.content.push(vec![res]);
        }
        if item.as_ref().ends_with("\n") {
            self.content.push(vec![]);
        }
    }

    pub fn get_content_height(&self) -> usize {
        if self.window.height > 2 {
            (self.window.height - 2) as _
        } else {
            0
        }
    }

    pub fn get_content_width(&self) -> usize {
        if self.window.width > 2 {
            (self.window.width - 2) as _
        } else {
            0
        }
    }
}

impl InteractiveWidget for IParagraph {
    fn select_up(&mut self) {
        if self.idx_page > 0 {
            self.idx_page -= 1;
        }
    }

    fn select_down(&mut self) {
        if self.content.len() > 0 && self.idx_page < self.content.len() - 1 {
            self.idx_page += 1;
        }
    }

    fn select_first(&mut self) {
        if self.idx_page != 0 {
            self.idx_page = 0;
        }
    }

    fn select_last(&mut self) {
        if self.content.len() >= self.get_content_height() &&
            self.idx_page != self.content.len() - self.get_content_height() {
            self.idx_page = self.content.len() - self.get_content_height();
        }
    }

    fn select_page_up(&mut self) {
        if self.idx_page != 0 {
            let width_limit = self.get_content_width();
            let height_limit = self.get_content_height();
            let mut shift = 0; let mut line_count = 0;
            for line in self.content[..self.idx_page].iter().rev() {
                if shift > height_limit { break; }
                let mut this_height = 0; let mut curr_width = 0;
                for (item, _) in line {
                    curr_width += item.len();
                    this_height += curr_width / width_limit;
                    curr_width %= width_limit;
                }
                this_height += 1;
                shift += this_height;
                line_count += 1;
            }
            self.idx_page -= line_count;
        }
    }

    fn select_page_down(&mut self) {
        if self.idx_page < self.content.len() {
            let width_limit = self.get_content_width();
            let height_limit = self.get_content_height();
            let mut shift = 0; let mut line_count = 0;
            for line in self.content[self.idx_page..].iter() {
                if shift > height_limit { break; }
                let mut this_height = 0; let mut curr_width = 0;
                for (item, _) in line {
                    curr_width += item.len();
                    this_height += curr_width / width_limit;
                    curr_width %= width_limit;
                }
                this_height += 1;
                shift += this_height;
                line_count += 1;
            }
            self.idx_page += line_count;

            self.idx_page = (self.idx_page+self.get_content_height()).min(self.content.len());
        }
    }

    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, name: &'static str, is_active: bool) {
        self.window = area;

        if self.get_content_width() == 0 { return; }
        let mut curr_height = 0; let mut buf = vec![];
        let width_limit = self.get_content_width();
        let height_limit = self.get_content_height();
        for line in self.content[self.idx_page..].iter() {
            if curr_height > height_limit { break; }
            let mut this_height = 0; let mut curr_width = 0;
            for (item, style) in line {
                curr_width += item.len();
                this_height += curr_width / width_limit;
                curr_width %= width_limit;
                buf.push(Text::styled(item, *style));
            }
            this_height += 1;
            curr_height += this_height;
            buf.push(Text::raw("\n"));
        }

        f.render_widget(
            Paragraph::new(buf.iter())
            .block(Block::default()
                .title(name)
                .borders(Borders::ALL)
                .border_style(
                    if is_active { Style::default().fg(Color::Red) } else { Style::default().fg(Color::White) })
                .title_style(Style::default().fg(Color::Yellow)))
            .wrap(true)
            .alignment(Alignment::Left),
            area
        );
    }
}

#[derive(Default)]
pub struct ITable {
    content    : Vec<(String, Style, usize, Vec<String>, Box<dyn Fn(&str, bool) -> Style>)>,

    width      : usize,
    height     : usize,

    idx_page   : usize,
    idx_select : usize,

    window     : Rect,
}

fn default_highlight(_: &str, is_select: bool) -> Style {
    if is_select {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::White)
    }
}

impl ITable {
    pub fn new() -> Self {
        let mut slf: Self = Default::default();
        //slf.add_column("col1", Style::default().fg(Color::Gray), 8, vec![], default_highlight);
        //slf.add_column("col2", Style::default().fg(Color::Gray), 8, vec![], default_highlight);
        //for i in 0..100 {
            //slf.add_row(vec![format!("row{}1", i), format!("row{}2", i)]);
        //}
        slf
    }

    pub fn add_column<F, S1, S2>(&mut self, header: S1, hstyle: Style, width: usize, mut column: Vec<S2>, f: F)
        where F : Fn(&str, bool) -> Style + 'static,
              S1 : ToString,
              S2 : ToString,
    {
        let mut align_column: Vec<String>;
        if self.content.len() == 0 { self.height = column.len(); }
        if self.height < column.len() {
            align_column = column.into_iter().take(self.height).map(|s| s.to_string()).collect();
        } else {
            let left = self.height - column.len();
            align_column = column.into_iter().map(|s| s.to_string()).collect();
            for _ in 0..left {
                align_column.push("".into());
            }
        }

        self.width += 1;
        let f : Box<dyn Fn(&str, bool) -> Style> = Box::new(f);
        self.content.push((header.to_string(), hstyle, width, align_column, f));
    }

    pub fn add_row<S: ToString>(&mut self, rows: Vec<S>) {
        assert!(rows.len() == self.width);
        for ((_, _, _, row, _), ref nval) in self.content.iter_mut().zip(rows.into_iter()) {
            row.push(nval.to_string());
        }
        self.height += 1;
    }

    pub fn remove_col(&mut self, idx: usize) {
        assert!(self.width > idx);
        self.content.remove(idx);
        self.width -= 1;
    }

    pub fn remove_row(&mut self, idx: usize) {
        assert!(self.height > idx);
        for (_, _, _, row, _) in self.content.iter_mut() {
            row.remove(idx);
        }
        self.height -= 1;
    }

    fn wrap_page(&mut self) {
        if self.idx_select < self.idx_page {
            self.idx_page = self.idx_select;
        }

        if self.idx_select >= self.idx_page + self.get_content_height() {
            self.idx_page = self.idx_select - self.get_content_height();
            if self.idx_page + 1 < self.height { self.idx_page += 1; }
        }
    }

    fn wrap_select(&mut self) {
        if self.idx_select < self.idx_page {
            self.idx_select = self.idx_page;
        }

        let maxreach = self.idx_page + self.get_content_height();
        if maxreach > 0 && self.idx_select > maxreach {
            self.idx_select = maxreach - 1;
        }
    }

    fn get_content_width(&self) -> usize {
        if self.window.width > 2 {
            (self.window.width - 2) as _
        } else {
            0
        }
    }

    fn get_content_height(&self) -> usize {
        if self.window.height > 3 {
            (self.window.height - 3) as _
        } else {
            0
        }
    }
}

impl InteractiveWidget for ITable {
    fn select_up(&mut self) {
        if self.idx_select > 0 {
            self.idx_select -= 1;
            self.wrap_page();
        }
    }

    fn select_down(&mut self) {
        if self.idx_select < self.height - 1 {
            self.idx_select += 1;
            self.wrap_page();
        }
    }

    fn select_first(&mut self) {
        if self.idx_select != 0 {
            self.idx_select = 0;
            self.wrap_page();
        }
    }

    fn select_last(&mut self) {
        if self.idx_select != self.height - 1 {
            self.idx_select = self.height - 1;
            self.wrap_page();
        }
    }

    fn select_page_up(&mut self) {
        if self.idx_page != 0 {
            let shift = self.get_content_height().min(self.idx_page);
            if shift > 0 {
                self.idx_page -= shift;
            } else {
                self.idx_page -= 0;
            }
            self.wrap_select();
        }
    }

    fn select_page_down(&mut self) {
        if self.get_content_height() > 0 && self.height > 0 && self.idx_page < self.height {
            self.idx_page = (self.idx_page + self.get_content_height() - 1).min(self.height-1);
            self.wrap_select();
        }
    }

    fn select_on_key(&mut self) {  }

    fn click(&mut self, _xpos: u16, ypos: u16, ) {
        if ypos > 1 && ypos < self.window.height {
            self.idx_select = self.idx_page + ypos as usize - 2;
        }
    }

    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, name: &'static str, is_active: bool) {
        self.window = area;

        let block = Block::default()
            .title(name)
            .borders(Borders::ALL)
            .border_style(if is_active { Style::default().fg(Color::Red) } else { Style::default().fg(Color::White) })
            .title_style(Style::default().fg(Color::Yellow));
        f.render_widget(block, area);

        let constraints: Vec<_> = self.content.iter()
            .map(|(_, _, len, _, _)| Constraint::Length(*len as u16 + 1))
            .collect();
        let tablerows = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(block.inner(area));

        let idx_end = self.height.min(self.idx_page+self.get_content_height());

        for (idx, (header, headerstyle, _, rows, stylefun)) in self.content.iter().enumerate() {
            let rowchunk = tablerows[idx];
            let mut rowlist = Vec::with_capacity(rows.len());
            rowlist.push(Text::styled(header, *headerstyle));
            for (idx, row) in rows[self.idx_page..idx_end].iter().enumerate() {
                let abs_idx = idx + self.idx_page;
                let is_select = self.idx_select == abs_idx;
                let style = stylefun(row, is_select);
                rowlist.push(Text::styled(row, style));
            }

            f.render_widget(List::new(rowlist.into_iter()), rowchunk);
        }
    }
}

#[derive(Default)]
pub struct IEmpty;
impl InteractiveWidget for IEmpty {  }

impl Drop for App {
    fn drop(&mut self) {
        unsafe { self.drop_map(); }
        // no obj need manually drop ??
    }
}

