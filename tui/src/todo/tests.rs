use super::*;
use crate::testing::TempDir;

fn item(line: &str) -> Item {
    parse(line, Path::new("/t/days")).unwrap()
}

#[test]
fn a_full_line_splits_into_its_parts() {
    let got = item("TODO     Tree view TUI  [sam_stack] → tree-view  [details](../details/tree-view-tui.md)");
    assert_eq!(
        got,
        Item {
            status: Status::Todo,
            text: "Tree view TUI".into(),
            project: Some("sam_stack".into()),
            slug: Some("tree-view".into()),
            details: Some(PathBuf::from("/t/days/../details/tree-view-tui.md")),
            moved: None,
        }
    );
}

#[test]
fn moves_and_every_status_word() {
    let moved = item("MOVED    Archive summaries  [sam_stack] (to backlog)");
    assert_eq!((moved.status, moved.moved.as_deref()), (Status::Moved, Some("backlog")));
    assert_eq!(moved.text, "Archive summaries");
    assert_eq!(moved.project.as_deref(), Some("sam_stack"));
    let late = item("MOVED    Docs  [x] → docs  [details](../details/docs.md) (to 2026-10-30)");
    assert_eq!(late.moved.as_deref(), Some("2026-10-30"));
    assert_eq!(late.slug.as_deref(), Some("docs"));
    assert_eq!(item("DONE     Shape  [p] → shape").status, Status::Done);
    assert_eq!(item("DROPPED  Organise skill  [p]").status, Status::Dropped);
    let note = item("NOTE     archive summaries should cite ids");
    assert_eq!(
        (note.status, note.text.as_str(), note.project),
        (Status::Note, "archive summaries should cite ids", None)
    );
}

#[test]
fn brackets_and_parentheses_inside_the_text_stay_text() {
    let got = item("TODO     Read [the] docs (later)");
    assert_eq!(got.text, "Read [the] docs (later)");
    assert_eq!(
        (got.project, got.moved, got.slug, got.details),
        (None, None, None, None)
    );
    let open = item("TODO     Fix [details](unclosed");
    assert_eq!((open.text.as_str(), open.details), ("Fix [details](unclosed", None));
    assert_eq!(item("TODO     Tag  [p] after").project, None);
}

#[test]
fn non_item_lines_are_skipped() {
    for line in ["# 2026-10-28", "", "TODO", "todo  lower", "- a note"] {
        assert_eq!(parse(line, Path::new(".")), None, "{line}");
    }
}

fn journal() -> TempDir {
    let dir = TempDir::new();
    dir.write("days/2026-10-27.md", "# 2026-10-27\nTODO     Old  [p]\n");
    dir.write(
        "days/2026-10-28.md",
        "# 2026-10-28\nTODO     Mine  [p]\nDONE     Shipped  [p]\n",
    );
    dir.write("days/2026-10-30.md", "# 2026-10-30\nTODO     Future  [p]\n");
    dir.write("days/notes.txt", "x");
    dir.write("backlog.md", "TODO     Parked  [p]  [details](details/parked.md)\n");
    dir.write("now.md", "# Now\n\n1. Mine  [p]\n2. Parked  [p]\n");
    dir
}

#[test]
fn loads_today_the_backlog_and_now() {
    let dir = journal();
    let todo = load(dir.path(), "2026-10-28");
    assert_eq!((todo.date.as_deref(), todo.today), (Some("2026-10-28"), true));
    let texts: Vec<&str> = todo.items.iter().map(|i| i.text.as_str()).collect();
    assert_eq!(texts, ["Mine", "Shipped"]);
    assert_eq!(todo.backlog.len(), 1);
    assert_eq!(todo.backlog[0].details, Some(dir.path().join("details/parked.md")));
    assert_eq!(todo.now, ["1. Mine  [p]", "2. Parked  [p]"]);
}

#[test]
fn before_triage_it_shows_the_latest_earlier_day() {
    let dir = journal();
    let todo = load(dir.path(), "2026-10-29");
    assert_eq!((todo.date.as_deref(), todo.today), (Some("2026-10-28"), false));
    assert_eq!(todo.items[0].text, "Mine");
    let early = load(dir.path(), "2026-10-27");
    assert_eq!(
        (early.date.as_deref(), early.items[0].text.as_str()),
        (Some("2026-10-27"), "Old")
    );
}

#[test]
fn an_empty_journal_loads_empty() {
    let dir = TempDir::new();
    assert_eq!(load(dir.path(), "2026-10-28"), Todo::default());
}
