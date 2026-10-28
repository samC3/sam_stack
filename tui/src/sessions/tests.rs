use super::*;
use crate::testing::TempDir;

fn session(dir: &TempDir, rel: &str, name: &str) {
    dir.write(&format!("{rel}/meta.json"), &format!("{{\"name\":\"{name}\"}}"));
}

fn states(name: &str) -> String {
    format!("state-of-{name}")
}

#[test]
fn every_project_slug_and_nested_session_in_order() {
    let dir = TempDir::new();
    session(&dir, "sam_stack/tree/sessions/parser", "parser");
    session(&dir, "sam_stack/tree/sessions/parser/sessions/lexer", "parser-lexer");
    session(&dir, "sam_stack/tree/sessions/docs", "docs");
    session(&dir, "_global/chores/sessions/tidy", "tidy");
    dir.write("sam_stack/tree/sessions/parser/return.md", "status: done\n");
    let rows = load(dir.path(), &states);
    let at = |rel: &str| dir.path().join(rel);
    assert_eq!(
        rows,
        [
            Row::Project("_global".into()),
            Row::Slug("chores".into()),
            Row::Session {
                depth: 0,
                name: "tidy".into(),
                state: "state-of-tidy".into(),
                returned: "-".into(),
                folder: at("_global/chores/sessions/tidy"),
            },
            Row::Project("sam_stack".into()),
            Row::Slug("tree".into()),
            Row::Session {
                depth: 0,
                name: "docs".into(),
                state: "state-of-docs".into(),
                returned: "-".into(),
                folder: at("sam_stack/tree/sessions/docs"),
            },
            Row::Session {
                depth: 0,
                name: "parser".into(),
                state: "state-of-parser".into(),
                returned: "done".into(),
                folder: at("sam_stack/tree/sessions/parser"),
            },
            Row::Session {
                depth: 1,
                name: "parser-lexer".into(),
                state: "state-of-parser-lexer".into(),
                returned: "-".into(),
                folder: at("sam_stack/tree/sessions/parser/sessions/lexer"),
            },
        ]
    );
}

#[test]
fn shared_state_archives_and_idle_slugs_are_left_out() {
    let dir = TempDir::new();
    session(&dir, "lessons/x/sessions/a", "a");
    session(&dir, "todo/x/sessions/b", "b");
    session(&dir, "proj/_archive/sessions/c", "c");
    session(&dir, "proj/live/sessions/d", "d");
    dir.write("proj/quiet/findings.md", "");
    dir.write("proj/stray.md", "");
    dir.write("loose.md", "");
    session(&dir, "empty/started/sessions/e", "e");
    std::fs::remove_file(dir.path().join("empty/started/sessions/e/meta.json")).unwrap();
    let rows = load(dir.path(), &states);
    assert_eq!(rows[..2], [Row::Project("proj".into()), Row::Slug("live".into())]);
    assert_eq!(rows.len(), 3);
}

#[test]
fn an_unreadable_meta_falls_back_to_the_folder_name() {
    let dir = TempDir::new();
    dir.write("p/s/sessions/odd/meta.json", "not json");
    dir.write("p/s/sessions/blank/meta.json", "{}");
    let names: Vec<String> = load(dir.path(), &states)
        .into_iter()
        .filter_map(|r| match r {
            Row::Session { name, .. } => Some(name),
            _ => None,
        })
        .collect();
    assert_eq!(names, ["blank", "odd"]);
}
