use std::path::PathBuf;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::*;
use crate::app::{Popup, Snapshot};
use crate::todo::Todo;

fn render(app: &App, width: u16, height: u16) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|f| draw(f, app)).unwrap();
    terminal.backend().buffer().clone()
}

fn lines(buf: &Buffer) -> Vec<String> {
    let width = usize::from(buf.area.width);
    buf.content
        .chunks(width)
        .map(|row| row.iter().map(ratatui::buffer::Cell::symbol).collect())
        .collect()
}

fn find(buf: &Buffer, text: &str) -> (u16, u16) {
    for (y, line) in lines(buf).iter().enumerate() {
        if let Some(at) = line.find(text) {
            let x = line[..at].chars().count();
            return (u16::try_from(x).unwrap(), u16::try_from(y).unwrap());
        }
    }
    panic!("{text:?} not on screen:\n{}", lines(buf).join("\n"));
}

fn style(buf: &Buffer, text: &str) -> Style {
    let (x, y) = find(buf, text);
    buf[(x, y)].style()
}

fn shown(buf: &Buffer, text: &str) -> bool {
    lines(buf).iter().any(|l| l.contains(text))
}

fn session(depth: usize, name: &str, state: &str, returned: &str) -> Row {
    Row::Session {
        depth,
        name: name.into(),
        state: state.into(),
        returned: returned.into(),
        folder: PathBuf::from("/s"),
    }
}

fn entry(status: Status, text: &str) -> Item {
    Item {
        status,
        text: text.into(),
        project: None,
        slug: None,
        details: None,
        moved: None,
    }
}

fn sample() -> App {
    let mut tagged = entry(Status::Todo, "Tree view");
    tagged.project = Some("sam_stack".into());
    tagged.slug = Some("tree-view".into());
    tagged.details = Some(PathBuf::from("/d.md"));
    let mut moved = entry(Status::Moved, "Archive");
    moved.moved = Some("backlog".into());
    App::new(Snapshot {
        rows: vec![
            Row::Project("sam_stack".into()),
            Row::Slug("tree-view".into()),
            session(0, "parser", "working", "done"),
            session(1, "parser-lexer", "blocked", "-"),
            session(0, "docs", "idle", "stale (done)"),
            session(0, "old", "gone", "-"),
            session(0, "fresh", "done", "-"),
        ],
        todo: Todo {
            date: Some("2026-10-28".into()),
            today: true,
            items: vec![
                tagged,
                entry(Status::Done, "Shipped"),
                moved,
                entry(Status::Dropped, "Dropped one"),
                entry(Status::Note, "a thought"),
            ],
            backlog: vec![entry(Status::Todo, "Parked")],
            now: vec!["1. Tree view  [sam_stack]".into()],
        },
    })
}

#[test]
fn wide_screens_put_sessions_beside_the_list() {
    let buf = render(&sample(), 120, 20);
    let screen = lines(&buf);
    assert!(screen[0].starts_with("┌ Sessions "), "{}", screen.join("\n"));
    assert!(screen[0].contains("┌ Now "), "{}", screen.join("\n"));
    assert_eq!(find(&buf, "┌ Now ").0, 60);
    assert!(screen[19].starts_with(HELP), "{}", screen[19]);
    assert_eq!(style(&buf, "←/→").fg, Some(Color::DarkGray));
}

#[test]
fn narrow_screens_stack_them() {
    let buf = render(&sample(), 99, 30);
    assert_eq!(find(&buf, "┌ Sessions ").0, 0);
    assert_eq!(find(&buf, "┌ Now "), (0, 15));
}

#[test]
fn the_tree_indents_sessions_under_project_and_slug() {
    let buf = render(&sample(), 120, 20);
    assert_eq!(find(&buf, "sam_stack").0, 1);
    assert_eq!(find(&buf, "tree-view ").0, 3);
    assert_eq!(find(&buf, "parser ").0, 5);
    assert_eq!(find(&buf, "parser-lexer").0, 7);
    let (x, y) = find(&buf, "working");
    assert_eq!(x, 5 + 14 + 2);
    assert_eq!(find(&buf, "blocked").0, x);
    assert!(lines(&buf)[usize::from(y)].contains("working  return: done"));
    assert!(shown(&buf, "idle     return: stale (done)"));
    assert!(style(&buf, "sam_stack").add_modifier.contains(Modifier::BOLD));
    assert_eq!(style(&buf, "tree-view ").fg, Some(Color::Cyan));
}

#[test]
fn herdr_states_have_colours() {
    let buf = render(&sample(), 120, 20);
    assert_eq!(style(&buf, "working").fg, Some(Color::Yellow));
    assert_eq!(style(&buf, "blocked").fg, Some(Color::Red));
    assert_eq!(style(&buf, "idle ").fg, Some(Color::Green));
    assert_eq!(style(&buf, "done     return").fg, Some(Color::Green));
    assert_eq!(style(&buf, "gone").fg, Some(Color::DarkGray));
}

#[test]
fn todo_items_show_status_tags_slugs_details_and_moves() {
    let buf = render(&sample(), 120, 20);
    assert!(shown(&buf, "1. Tree view  [sam_stack]"));
    assert!(shown(&buf, "• Tree view  [sam_stack] → tree-view +"));
    assert!(shown(&buf, "✓ Shipped"));
    assert!(shown(&buf, "> Archive (to backlog)"));
    assert!(shown(&buf, "✗ Dropped one"));
    assert!(shown(&buf, "– a thought"));
    assert_eq!(style(&buf, "• Tree").fg, Some(Color::Reset));
    assert_eq!(style(&buf, "✓").fg, Some(Color::Green));
    assert_eq!(style(&buf, "> Archive").fg, Some(Color::DarkGray));
    assert_eq!(style(&buf, " (to backlog)").fg, Some(Color::DarkGray));
    assert!(style(&buf, "✗").add_modifier.contains(Modifier::CROSSED_OUT));
    assert!(style(&buf, "– a").add_modifier.contains(Modifier::ITALIC));
    assert_eq!(style(&buf, "  [sam_stack] →").fg, Some(Color::DarkGray));
    assert_eq!(style(&buf, " → tree-view").fg, Some(Color::Cyan));
    assert_eq!(style(&buf, " +").fg, Some(Color::Magenta));
}

#[test]
fn the_title_marks_the_active_list() {
    let mut app = sample();
    let buf = render(&app, 120, 20);
    assert!(shown(&buf, "┌ Today 2026-10-28  Backlog (1) "));
    let today = style(&buf, "Today 2026");
    assert!(today.add_modifier.contains(Modifier::BOLD | Modifier::UNDERLINED));
    assert_eq!(style(&buf, "Backlog (1)").fg, Some(Color::DarkGray));
    app.key(KeyEvent::from(KeyCode::Tab));
    let buf = render(&app, 120, 20);
    assert!(style(&buf, "Backlog (1)").add_modifier.contains(Modifier::BOLD));
    assert_eq!(style(&buf, "Today 2026").fg, Some(Color::DarkGray));
    assert!(shown(&buf, "• Parked"));
    assert!(!shown(&buf, "┌ Now "));
    assert!(!shown(&buf, "Shipped"));
}

#[test]
fn a_day_before_triage_says_so() {
    let mut app = sample();
    app.snap.todo.today = false;
    assert!(shown(
        &render(&app, 120, 20),
        "┌ 2026-10-28 (today not started)  Backlog"
    ));
    app.snap.todo.date = None;
    assert!(shown(&render(&app, 120, 20), "┌ Today  Backlog"));
}

#[test]
fn only_the_focused_pane_highlights_its_selection() {
    let mut app = sample();
    let buf = render(&app, 120, 20);
    assert!(style(&buf, "sam_stack ").add_modifier.contains(Modifier::REVERSED));
    assert!(!style(&buf, "• Tree").add_modifier.contains(Modifier::REVERSED));
    app.key(KeyEvent::from(KeyCode::Right));
    let buf = render(&app, 120, 20);
    assert!(!style(&buf, "sam_stack ").add_modifier.contains(Modifier::REVERSED));
    assert!(style(&buf, "• Tree").add_modifier.contains(Modifier::REVERSED));
}

#[test]
fn empty_state_says_what_is_missing() {
    let buf = render(&App::new(Snapshot::default()), 120, 10);
    assert!(shown(&buf, "│No sessions. The supervise skill starts them."));
    assert!(shown(&buf, "│Nothing here."));
    assert!(!shown(&buf, "┌ Now "));
    assert_eq!(style(&buf, "No sessions").fg, Some(Color::DarkGray));
    assert_eq!(style(&buf, "Nothing here").fg, Some(Color::DarkGray));
}

#[test]
fn a_popup_covers_the_middle() {
    let mut app = sample();
    app.popup = Some(Popup {
        title: "parser".into(),
        text: "── brief.md ──\nGoal: a long goal that wraps around the popup".into(),
    });
    let buf = render(&app, 100, 41);
    assert_eq!(find(&buf, "┌ parser "), (10, 5));
    assert!(shown(&buf, "│── brief.md ──"));
    assert!(shown(&buf, "│Goal: a long goal"));
    assert_eq!(find(&buf, " esc close ").1, 34);
    let mut narrow = sample();
    narrow.popup = app.popup.take();
    let buf = render(&narrow, 30, 41);
    assert!(shown(&buf, "│Goal: a long goal that"), "{}", lines(&buf).join("\n"));
    assert!(shown(&buf, "│wraps around the popup"), "{}", lines(&buf).join("\n"));
}
