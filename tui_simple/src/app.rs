#![allow(dead_code)]
use crate::Event;

use crossterm::{
    event::{self, Event as CEvent, KeyEvent, MouseEvent, KeyCode},
};
use std::collections::HashMap;
use std::ffi::c_void;
use std::time::{Duration, Instant};
use std::any::Any;

use tui::{
    backend::Backend,
    terminal::Frame,

    style::{Color, Modifier, Style},
    widgets::*,
    layout::*,
    Terminal,
};

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

pub struct App {
    widgets: HashMap<&'static str, *mut c_void>,
    widgets_location: HashMap<&'static str, Rect>,

    curr_widget: Option<&'static str>,
    status: Status,
    stack_kevent_time: Instant,
}
impl Default for App {
    fn default() -> Self {
        Self {
            widgets           : Default::default(),
            widgets_location  : Default::default(),
            curr_widget       : Default::default(),
            status            : Default::default(),
            stack_kevent_time : Instant::now(),
        }
    }
}

/// this is where you set layout and event handler.
/// just use 'static element. It's clear and easy to access.
impl InteractiveWidget for App {
    fn select_page_up(&mut self) {
        if let Some(curr_tag) = self.curr_widget {
            let ptr: *mut c_void = *self.widgets.get(curr_tag).unwrap();
            dispatch!(ptr, curr_tag, [
                ("Assets", IEmpty), ("Positions", IEmpty), ("Orders", IEmpty),
                ("Trades", ITable), ("Logs", IParagraph)
            ], select_page_up, [  ] );
        }
    }

    fn select_page_down(&mut self) {
        if let Some(curr_tag) = self.curr_widget {
            let ptr: *mut c_void = *self.widgets.get(curr_tag).unwrap();
            dispatch!(ptr, curr_tag, [
                ("Assets", IEmpty), ("Positions", IEmpty), ("Orders", IEmpty),
                ("Trades", ITable), ("Logs", IParagraph)
            ], select_page_down, [  ] );
        }
    }

    fn select_up(&mut self) {
        if let Some(curr_tag) = self.curr_widget {
            let ptr: *mut c_void = *self.widgets.get(curr_tag).unwrap();
            dispatch!(ptr, curr_tag, [
                ("Assets", IEmpty), ("Positions", IEmpty), ("Orders", IEmpty),
                ("Trades", ITable), ("Logs", IParagraph)
            ], select_up, [  ] );
        }
    }

    fn select_down(&mut self) {
        if let Some(curr_tag) = self.curr_widget {
            let ptr: *mut c_void = *self.widgets.get(curr_tag).unwrap();
            dispatch!(ptr, curr_tag, [
                ("Assets", IEmpty), ("Positions", IEmpty), ("Orders", IEmpty),
                ("Trades", ITable), ("Logs", IParagraph)
            ], select_down, [  ] );
        }
    }

    fn select_last(&mut self) {
        if let Some(curr_tag) = self.curr_widget {
            let ptr: *mut c_void = *self.widgets.get(curr_tag).unwrap();
            dispatch!(ptr, curr_tag, [
                ("Assets", IEmpty), ("Positions", IEmpty), ("Orders", IEmpty),
                ("Trades", ITable), ("Logs", IParagraph)
            ], select_last, [  ] );
        }
    }

    fn select_first(&mut self) {
        if let Some(curr_tag) = self.curr_widget {
            let ptr: *mut c_void = *self.widgets.get(curr_tag).unwrap();
            dispatch!(ptr, curr_tag, [
                ("Assets", IEmpty), ("Positions", IEmpty), ("Orders", IEmpty),
                ("Trades", ITable), ("Logs", IParagraph)
            ], select_first, [  ] );
        }
    }

    fn click(&mut self, xpos: u16, ypos: u16, ) {
        for (name, rect) in self.widgets_location.iter() {
            if xpos > rect.left() && xpos < rect.right() && ypos < rect.bottom() && ypos > rect.top() {
                self.curr_widget = Some(name);
                break;
            }
        }

        self.info(format!("change foucs pos: {} {} => select: {:?}", xpos, ypos, self.curr_widget));
    }
}

impl App {
    pub fn new() -> Self {
        let mut slf = Self::default();
        slf.add_widget("Assets", IEmpty::default());
        slf.add_widget("Positions", IEmpty::default());
        slf.add_widget("Orders", IEmpty::default());
        slf.add_widget("Trades", ITable::new());
        slf.add_widget("Logs", IParagraph::new());
        slf
    }

    pub fn debug<S: AsRef<str>>(&mut self, msg: S) { self.log(msg, "DEBUG"); }
    pub fn info<S: AsRef<str>>(&mut self, msg: S) { self.log(msg, "INFO"); }
    pub fn warning<S: AsRef<str>>(&mut self, msg: S) { self.log(msg, "WARNING"); }
    pub fn error<S: AsRef<str>>(&mut self, msg: S) { self.log(msg, "ERROR"); }
    pub fn critical<S: AsRef<str>>(&mut self, msg: S) { self.log(msg, "CRITICAL"); }
    fn log<S: AsRef<str>>(&mut self, msg: S, level: &str) {
        let style = match level.as_ref() {
            "CRITICAL" => Style::default().fg(Color::Red),
            "ERROR"    => Style::default().fg(Color::Magenta),
            "WARNING"  => Style::default().fg(Color::Yellow),
            "INFO"     => Style::default().fg(Color::White),
            _          => Style::default().fg(Color::Gray),
        };

        let msg = format!("{}: {}\n", level, msg.as_ref());
        let tag = "Logs";
        let ptr: *mut c_void = *self.widgets.get(tag).unwrap();
        dispatch!(ptr, IParagraph, add_styled, (msg, style));
    }

    /// I will handle msg in dispatch. This function is unsafe, since 1. widget will become raw
    /// pointer, 2. kind must be the same as widget type. (there are ways bypass this, but they all
    /// are ugly)
    fn add_widget<T: InteractiveWidget>(&mut self, name: &'static str, widget: T) {
        assert!(!self.widgets.contains_key(name), "widget with same name");
        self.widgets_location.insert(name, Rect::default());
        self.widgets.insert(name, Box::into_raw(Box::new(widget)) as *mut c_void);
    }

    fn move_left(&mut self) {
        if let Some(ref mut curr) = self.curr_widget {
            match *curr {
                "Trades" => { *curr = "Logs" }
                "Logs"   => { *curr = "Trades" }
                _        => {  }
            }
        }
    }

    fn move_right(&mut self) {
        if let Some(ref mut curr) = self.curr_widget {
            match *curr {
                "Trades" => { *curr = "Logs" }
                "Logs"   => { *curr = "Trades" }
                _        => {  }
            }
        }

    }

    fn move_up(&mut self) {

    }

    fn move_down(&mut self) {

    }

    pub fn draw<B: Backend>(&mut self, mut f: Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(f.size());

        {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [Constraint::Percentage(20), Constraint::Percentage(40), Constraint::Percentage(40)].as_ref())
                .split(chunks[0]);

            let tag = "Assets"; let rect = chunks[0];
            *self.widgets_location.get_mut(tag).unwrap() = rect;
            let ptr: *mut c_void = *self.widgets.get(tag).unwrap();
            dispatch!(ptr, IEmpty, draw, (&mut f, rect, tag, self.curr_widget.map(|t| t == tag).unwrap_or(false)));

            let tag = "Positions"; let rect = chunks[1];
            *self.widgets_location.get_mut(tag).unwrap() = rect;
            let ptr: *mut c_void = *self.widgets.get(tag).unwrap();
            dispatch!(ptr, IEmpty, draw, (&mut f, rect, tag, self.curr_widget.map(|t| t == tag).unwrap_or(false)));

            let tag = "Orders"; let rect = chunks[2];
            *self.widgets_location.get_mut(tag).unwrap() = rect;
            let ptr: *mut c_void = *self.widgets.get(tag).unwrap();
            dispatch!(ptr, IEmpty, draw, (&mut f, rect, tag, self.curr_widget.map(|t| t == tag).unwrap_or(false)));
        }

        {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(chunks[1]);

            let tag = "Trades"; let rect = chunks[0];
            *self.widgets_location.get_mut(tag).unwrap() = rect;
            let ptr: *mut c_void = *self.widgets.get(tag).unwrap();
            dispatch!(ptr, ITable, draw, (&mut f, rect, tag, self.curr_widget.map(|t| t == tag).unwrap_or(false)));

            let tag = "Logs"; let rect = chunks[1];
            *self.widgets_location.get_mut(tag).unwrap() = rect;
            let ptr: *mut c_void = *self.widgets.get(tag).unwrap();
            dispatch!(ptr, IParagraph, draw, (&mut f, rect, tag, self.curr_widget.map(|t| t == tag).unwrap_or(false)));
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
            (Status::Normal, Event::CharKey('g'))      => { self.debug("Got g signal"); self.status = Status::WaitG; },
            (Status::WaitG,  Event::CharKey('g'))      => { self.select_first(); self.status = Status::Normal; },
            (Status::Normal, Event::CharKey('j'))      => { self.select_down(); },
            (Status::Normal, Event::CharKey('k'))      => { self.select_up(); },
            (Status::Normal, Event::Tick)              => { if self.stack_kevent_time.elapsed() > Duration::from_secs(2) { } }
            (Status::Insert(_), Event::Tick)           => {}
            (Status::Normal, Event::Up)                => { self.select_up() },
            (Status::Normal, Event::Down)              => { self.select_down() },
            (Status::Normal, Event::ScrollUp(_, _))    => { self.select_up(); },
            (Status::Normal, Event::ScrollDown(_, _))  => { self.select_down(); },
            (Status::Normal, Event::Press(xpos, ypos)) => { self.click(xpos, ypos); },
            (Status::Insert(mut m), Event::CharKey(c)) => { m.push(c); self.status = Status::Insert(m); },
            (Status::Insert(m), Event::Enter)          => { self.info(format!("input {}", m)); self.status = Status::Normal; },
            _                                          => {
                self.debug(format!("Got key: {:?} status: {:?}", event, self.status));
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
        let mut slf = Self::default();
        for _ in 0..1 {
            slf.add("     line1\n");
            slf.add_styled("line2\nline3\nline4\n\n\nline6", Style::default().fg(Color::Red));
            slf.add_styled("continue this    line", Style::default().fg(Color::Green));
            slf.add_styled(", and more\n", Style::default().fg(Color::Cyan));
        }
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
        if self.idx_page < self.content.len() - 1 {
            self.idx_page += 1;
        }
    }

    fn select_first(&mut self) {
        if self.idx_page != 0 {
            self.idx_page = 0;
        }
    }

    fn select_last(&mut self) {
        if self.idx_page != self.content.len() - 1 {
            self.idx_page = self.content.len() - 1;
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
        let mut slf = Self::default();
        slf.add_column("col1", Style::default().fg(Color::Gray), 8, vec![], default_highlight);
        slf.add_column("col2", Style::default().fg(Color::Gray), 8, vec![], default_highlight);
        for i in 0..100 {
            slf.add_row(vec![format!("row{}1", i), format!("row{}2", i)]);
        }
        slf
    }

    pub fn add_column<F, S>(&mut self, header: S, hstyle: Style, width: usize, mut column: Vec<String>, f: F)
        where F : Fn(&str, bool) -> Style + 'static,
              S : ToString,
    {
        if self.height < column.len() {
            column = column.into_iter().take(self.height).collect();
        } else {
            for _ in 0..self.height-column.len() {
                column.push("".into());
            }
        }

        self.width += 1;
        let f : Box<dyn Fn(&str, bool) -> Style> = Box::new(f);
        self.content.push((header.to_string(), hstyle, width, column, f));
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
                self.idx_page -= shift - 1;
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
    fn click(&mut self, _x: u16, _y: u16, ) {  } // relative click

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
        // no obj need manually drop ??
    }
}

