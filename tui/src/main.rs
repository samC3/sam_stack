use std::io;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyEvent, KeyEventKind};

fn next_key(timeout: Duration) -> io::Result<Option<KeyEvent>> {
    if !event::poll(timeout)? {
        return Ok(None);
    }
    Ok(Some(match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => key,
        _ => KeyEvent::from(event::KeyCode::Null),
    }))
}

fn main() -> io::Result<()> {
    let work = tui::work_dir(std::env::var_os("SAM_STACK_WORK"), std::env::var_os("HOME"));
    let mut terminal = ratatui::try_init()?;
    let result = tui::run(&mut terminal, next_key, || {
        tui::load(&work, &tui::today(), &supervise::state)
    });
    ratatui::restore();
    result
}
