use std::fs;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::sessions::Row;
use crate::todo::{Item, Todo};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub rows: Vec<Row>,
    pub todo: Todo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Sessions,
    Todo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum List {
    Today,
    Backlog,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Popup {
    pub title: String,
    pub text: String,
}

#[derive(Debug)]
pub struct App {
    pub snap: Snapshot,
    pub focus: Pane,
    pub list: List,
    pub row: usize,
    pub item: usize,
    pub popup: Option<Popup>,
    pub quit: bool,
}

fn step(at: &mut usize, len: usize, down: bool) {
    if down {
        *at = (*at + 1).min(len.saturating_sub(1));
    } else {
        *at = at.saturating_sub(1);
    }
}

fn session_text(folder: &std::path::Path) -> String {
    let parts: Vec<String> = ["brief.md", "return.md"]
        .iter()
        .filter_map(|f| Some(format!("── {f} ──\n{}", fs::read_to_string(folder.join(f)).ok()?)))
        .collect();
    if parts.is_empty() {
        "(no brief.md or return.md yet)".into()
    } else {
        parts.join("\n")
    }
}

impl App {
    pub fn new(snap: Snapshot) -> App {
        App {
            snap,
            focus: Pane::Sessions,
            list: List::Today,
            row: 0,
            item: 0,
            popup: None,
            quit: false,
        }
    }

    pub fn items(&self) -> &[Item] {
        match self.list {
            List::Today => &self.snap.todo.items,
            List::Backlog => &self.snap.todo.backlog,
        }
    }

    pub fn refresh(&mut self, snap: Snapshot) {
        self.snap = snap;
        self.row = self.row.min(self.snap.rows.len().saturating_sub(1));
        self.item = self.item.min(self.items().len().saturating_sub(1));
    }

    fn open(&self) -> Option<Popup> {
        match self.focus {
            Pane::Sessions => match self.snap.rows.get(self.row)? {
                Row::Session { name, folder, .. } => Some(Popup {
                    title: name.clone(),
                    text: session_text(folder),
                }),
                _ => None,
            },
            Pane::Todo => {
                let item = self.items().get(self.item)?;
                let path = item.details.as_ref()?;
                Some(Popup {
                    title: item.text.clone(),
                    text: fs::read_to_string(path).unwrap_or_else(|_| format!("(missing {})", path.display())),
                })
            }
        }
    }

    pub fn key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        if self.popup.is_some() {
            if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
                self.popup = None;
            }
            return;
        }
        let down = matches!(key.code, KeyCode::Down | KeyCode::Char('j'));
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            KeyCode::Left | KeyCode::Char('h') => self.focus = Pane::Sessions,
            KeyCode::Right | KeyCode::Char('l') => self.focus = Pane::Todo,
            KeyCode::Tab => {
                self.focus = Pane::Todo;
                self.list = match self.list {
                    List::Today => List::Backlog,
                    List::Backlog => List::Today,
                };
                self.item = 0;
            }
            KeyCode::Up | KeyCode::Down | KeyCode::Char('k' | 'j') => match self.focus {
                Pane::Sessions => step(&mut self.row, self.snap.rows.len(), down),
                Pane::Todo => {
                    let len = self.items().len();
                    step(&mut self.item, len, down);
                }
            },
            KeyCode::Enter => self.popup = self.open(),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
