use std::path::PathBuf;

use super::*;
use crate::testing::TempDir;
use crate::todo::Status;

fn press(app: &mut App, codes: &[KeyCode]) {
    for code in codes {
        app.key(KeyEvent::from(*code));
    }
}

fn session(name: &str, folder: PathBuf) -> Row {
    Row::Session {
        depth: 0,
        name: name.into(),
        state: "idle".into(),
        returned: "-".into(),
        folder,
    }
}

fn entry(text: &str, details: Option<PathBuf>) -> Item {
    Item {
        status: Status::Todo,
        text: text.into(),
        project: None,
        slug: None,
        details,
        moved: None,
    }
}

fn fixture(dir: &TempDir) -> App {
    dir.write("s/a/brief.md", "Goal: a\n");
    dir.write("s/a/return.md", "status: done\n");
    dir.write("s/b/brief.md", "Goal: b\n");
    dir.write("details/one.md", "# One\n- note\n");
    App::new(Snapshot {
        rows: vec![
            Row::Project("p".into()),
            session("a", dir.path().join("s/a")),
            session("b", dir.path().join("s/b")),
            session("c", dir.path().join("s/c")),
        ],
        todo: Todo {
            items: vec![
                entry("one", Some(dir.path().join("details/one.md"))),
                entry("two", None),
                entry("three", Some(dir.path().join("details/gone.md"))),
            ],
            backlog: vec![entry("parked", None)],
            ..Todo::default()
        },
    })
}

#[test]
fn starts_on_the_sessions_pane_and_today() {
    let dir = TempDir::new();
    let app = fixture(&dir);
    assert_eq!(
        (app.focus, app.list, app.row, app.item, app.quit),
        (Pane::Sessions, List::Today, 0, 0, false)
    );
    assert_eq!(app.popup, None);
    assert_eq!(app.items().len(), 3);
}

#[test]
fn arrows_and_vim_keys_move_within_the_focused_pane() {
    let dir = TempDir::new();
    let mut app = fixture(&dir);
    press(
        &mut app,
        &[KeyCode::Down, KeyCode::Char('j'), KeyCode::Down, KeyCode::Down],
    );
    assert_eq!((app.row, app.item), (3, 0));
    press(&mut app, &[KeyCode::Up, KeyCode::Char('k')]);
    assert_eq!(app.row, 1);
    press(&mut app, &[KeyCode::Up, KeyCode::Up]);
    assert_eq!(app.row, 0);
    press(&mut app, &[KeyCode::Right, KeyCode::Down, KeyCode::Down, KeyCode::Down]);
    assert_eq!((app.focus, app.row, app.item), (Pane::Todo, 0, 2));
    press(&mut app, &[KeyCode::Char('h')]);
    assert_eq!(app.focus, Pane::Sessions);
    press(&mut app, &[KeyCode::Char('l'), KeyCode::Char('k')]);
    assert_eq!((app.focus, app.item), (Pane::Todo, 1));
    press(&mut app, &[KeyCode::Left]);
    assert_eq!(app.focus, Pane::Sessions);
}

#[test]
fn moving_in_an_empty_list_stays_at_zero() {
    let mut app = App::new(Snapshot::default());
    press(&mut app, &[KeyCode::Down, KeyCode::Right, KeyCode::Down, KeyCode::Up]);
    assert_eq!((app.row, app.item), (0, 0));
}

#[test]
fn tab_swaps_today_and_backlog_and_focuses_the_list() {
    let dir = TempDir::new();
    let mut app = fixture(&dir);
    press(&mut app, &[KeyCode::Right, KeyCode::Down, KeyCode::Left, KeyCode::Tab]);
    assert_eq!((app.focus, app.list, app.item), (Pane::Todo, List::Backlog, 0));
    assert_eq!(app.items()[0].text, "parked");
    press(&mut app, &[KeyCode::Tab]);
    assert_eq!((app.list, app.items()[0].text.as_str()), (List::Today, "one"));
}

#[test]
fn enter_on_a_session_shows_its_brief_and_return() {
    let dir = TempDir::new();
    let mut app = fixture(&dir);
    press(&mut app, &[KeyCode::Enter]);
    assert_eq!(app.popup, None);
    press(&mut app, &[KeyCode::Down, KeyCode::Enter]);
    assert_eq!(
        app.popup,
        Some(Popup {
            title: "a".into(),
            text: "── brief.md ──\nGoal: a\n\n── return.md ──\nstatus: done\n".into(),
        })
    );
    press(&mut app, &[KeyCode::Down]);
    assert_eq!(app.row, 1);
    press(&mut app, &[KeyCode::Esc, KeyCode::Down, KeyCode::Enter]);
    assert_eq!(
        app.popup.as_ref().map(|p| p.text.as_str()),
        Some("── brief.md ──\nGoal: b\n")
    );
    press(&mut app, &[KeyCode::Enter, KeyCode::Down, KeyCode::Enter]);
    assert_eq!(
        app.popup.as_ref().map(|p| p.text.as_str()),
        Some("(no brief.md or return.md yet)")
    );
    press(&mut app, &[KeyCode::Char('q')]);
    assert_eq!((app.popup.as_ref(), app.quit), (None, false));
}

#[test]
fn enter_on_an_item_shows_its_details() {
    let dir = TempDir::new();
    let mut app = fixture(&dir);
    press(&mut app, &[KeyCode::Right, KeyCode::Enter]);
    assert_eq!(
        app.popup,
        Some(Popup {
            title: "one".into(),
            text: "# One\n- note\n".into(),
        })
    );
    press(&mut app, &[KeyCode::Esc, KeyCode::Down, KeyCode::Enter]);
    assert_eq!(app.popup, None);
    press(&mut app, &[KeyCode::Down, KeyCode::Enter]);
    let missing = format!("(missing {})", dir.path().join("details/gone.md").display());
    assert_eq!(app.popup.as_ref().map(|p| p.text.as_str()), Some(missing.as_str()));
}

#[test]
fn an_open_popup_takes_the_keys() {
    let dir = TempDir::new();
    let mut app = fixture(&dir);
    press(
        &mut app,
        &[
            KeyCode::Down,
            KeyCode::Enter,
            KeyCode::Down,
            KeyCode::Tab,
            KeyCode::Right,
        ],
    );
    assert!(app.popup.is_some());
    assert_eq!((app.row, app.focus, app.list), (1, Pane::Sessions, List::Today));
    app.key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    assert!(app.quit);
}

#[test]
fn q_esc_and_ctrl_c_quit() {
    let dir = TempDir::new();
    for key in [
        KeyEvent::from(KeyCode::Char('q')),
        KeyEvent::from(KeyCode::Esc),
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
    ] {
        let mut app = fixture(&dir);
        app.key(key);
        assert!(app.quit, "{key:?}");
    }
    let mut app = fixture(&dir);
    press(&mut app, &[KeyCode::Char('c'), KeyCode::Char('x')]);
    assert!(!app.quit);
}

#[test]
fn refresh_keeps_selections_inside_the_new_lists() {
    let dir = TempDir::new();
    let mut app = fixture(&dir);
    press(
        &mut app,
        &[
            KeyCode::Down,
            KeyCode::Down,
            KeyCode::Right,
            KeyCode::Down,
            KeyCode::Down,
        ],
    );
    app.refresh(Snapshot {
        rows: vec![Row::Project("p".into()), Row::Slug("s".into())],
        todo: Todo {
            items: vec![entry("only", None)],
            ..Todo::default()
        },
    });
    assert_eq!((app.row, app.item), (1, 0));
    press(&mut app, &[KeyCode::Left, KeyCode::Up]);
    app.refresh(fixture(&dir).snap);
    assert_eq!((app.row, app.item), (0, 0));
    app.refresh(Snapshot::default());
    assert_eq!((app.row, app.item), (0, 0));
}
