use std::cell::Cell;
use std::collections::VecDeque;

use ratatui::backend::TestBackend;

use super::*;
use crate::testing::TempDir;

#[test]
fn work_dir_is_the_variable_or_the_home_default() {
    let some = |s: &str| Some(OsString::from(s));
    assert_eq!(work_dir(some("/w"), some("/h")), PathBuf::from("/w"));
    assert_eq!(work_dir(some(""), some("/h")), PathBuf::from("/h/.sam_stack/work"));
    assert_eq!(work_dir(None, some("/h")), PathBuf::from("/h/.sam_stack/work"));
}

#[test]
fn today_is_an_iso_date() {
    let day = today();
    let shape: String = day.chars().map(|c| if c.is_ascii_digit() { '9' } else { c }).collect();
    assert_eq!(shape, "9999-99-99");
}

#[test]
fn load_reads_sessions_and_the_todo_folder() {
    let dir = TempDir::new();
    dir.write("p/s/sessions/a/meta.json", "{\"name\":\"a\"}");
    dir.write("todo/days/2026-10-28.md", "TODO     x  [p]\n");
    let snap = load(dir.path(), "2026-10-28", &|n| format!("{n}!"));
    assert_eq!(snap.rows.len(), 3);
    assert!(matches!(&snap.rows[2], sessions::Row::Session { state, .. } if state == "a!"));
    assert_eq!(snap.todo.items[0].text, "x");
}

fn snapshot(rows: usize) -> Snapshot {
    Snapshot {
        rows: (0..rows).map(|n| sessions::Row::Slug(format!("slug{n}"))).collect(),
        todo: todo::Todo::default(),
    }
}

fn script(keys: &[Option<KeyCode>]) -> impl FnMut(Duration) -> std::io::Result<Option<KeyEvent>> {
    let mut keys: VecDeque<Option<KeyCode>> = keys.iter().copied().collect();
    move |timeout| {
        assert_eq!(timeout, REFRESH);
        Ok(keys.pop_front().expect("ran past the script").map(KeyEvent::from))
    }
}

#[test]
fn run_draws_reloads_on_timeout_and_r_and_stops_on_q() {
    let mut terminal = Terminal::new(TestBackend::new(120, 10)).unwrap();
    let loads = Cell::new(0);
    let keys = [
        None,
        Some(KeyCode::Down),
        Some(KeyCode::Char('r')),
        Some(KeyCode::Char('q')),
    ];
    run(&mut terminal, script(&keys), || {
        loads.set(loads.get() + 1);
        snapshot(loads.get())
    })
    .unwrap();
    assert_eq!(loads.get(), 3);
    let screen: String = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();
    assert!(screen.contains("slug2"), "{screen}");
    assert_eq!(REFRESH, Duration::from_secs(2));
}

#[test]
fn r_inside_a_popup_does_not_reload() {
    let dir = TempDir::new();
    dir.write("p/s/sessions/a/meta.json", "{\"name\":\"a\"}");
    let mut terminal = Terminal::new(TestBackend::new(120, 10)).unwrap();
    let loads = Cell::new(0);
    let keys = [
        Some(KeyCode::Down),
        Some(KeyCode::Down),
        Some(KeyCode::Enter),
        Some(KeyCode::Char('r')),
        Some(KeyCode::Esc),
        Some(KeyCode::Esc),
    ];
    run(&mut terminal, script(&keys), || {
        loads.set(loads.get() + 1);
        load(dir.path(), "2026-10-28", &|_| "idle".into())
    })
    .unwrap();
    assert_eq!(loads.get(), 1);
}

#[test]
fn run_passes_input_errors_up() {
    let mut terminal = Terminal::new(TestBackend::new(20, 5)).unwrap();
    let failed = run(
        &mut terminal,
        |_| Err(std::io::Error::other("tty gone")),
        || snapshot(0),
    );
    assert_eq!(failed.unwrap_err().to_string(), "tty gone");
}
