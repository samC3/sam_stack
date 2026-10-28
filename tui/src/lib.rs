pub mod app;
pub mod sessions;
#[cfg(test)]
mod testing;
pub mod todo;
pub mod view;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use app::{App, Snapshot};

pub const REFRESH: Duration = Duration::from_secs(2);

pub fn work_dir(var: Option<OsString>, home: Option<OsString>) -> PathBuf {
    var.filter(|v| !v.is_empty()).map_or_else(
        || PathBuf::from(home.unwrap_or_default()).join(".sam_stack/work"),
        PathBuf::from,
    )
}

pub fn today() -> String {
    let out = Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .map(|o| o.stdout)
        .unwrap_or_default();
    String::from_utf8_lossy(&out).trim().to_string()
}

pub fn load(work: &Path, today: &str, state: &dyn Fn(&str) -> String) -> Snapshot {
    Snapshot {
        rows: sessions::load(work, state),
        todo: todo::load(&work.join("todo"), today),
    }
}

pub fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    mut next: impl FnMut(Duration) -> std::io::Result<Option<KeyEvent>>,
    mut load: impl FnMut() -> Snapshot,
) -> std::io::Result<()> {
    let mut app = App::new(load());
    while !app.quit {
        terminal
            .draw(|frame| view::draw(frame, &app))
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        match next(REFRESH)? {
            None => app.refresh(load()),
            Some(key) if key.code == KeyCode::Char('r') && app.popup.is_none() => app.refresh(load()),
            Some(key) => app.key(key),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
