use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const FAKE_HERDR: &str =
    "#!/bin/sh\n[ \"$3\" = parser ] || exit 1\necho '{\"result\":{\"agent\":{\"agent_status\":\"working\"}}}'\n";

fn temp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tui-terminal-{}-{tag}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("bin")).unwrap();
    fs::write(dir.join("bin/herdr"), FAKE_HERDR).unwrap();
    Command::new("chmod")
        .arg("+x")
        .arg(dir.join("bin/herdr"))
        .status()
        .unwrap();
    dir
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn journal(work: &Path) {
    let today = Command::new("date").arg("+%Y-%m-%d").output().unwrap().stdout;
    let today = String::from_utf8(today).unwrap();
    let session = work.join("proj/slug/sessions/parser");
    write(&session.join("meta.json"), "{\"name\":\"parser\"}");
    write(&session.join("return.md"), "status: done\n");
    write(&work.join("proj/slug/sessions/docs/meta.json"), "{\"name\":\"docs\"}");
    write(
        &work.join(format!("todo/days/{}.md", today.trim())),
        "TODO     Write the tests  [proj]\n",
    );
}

fn drive(dir: &Path, env: &[(&str, &Path)], keys: &str) -> (bool, String) {
    let path = format!("{}:{}", dir.join("bin").display(), std::env::var("PATH").unwrap());
    let command = format!("stty cols 120 rows 20; exec {}", env!("CARGO_BIN_EXE_tui"));
    let mut child = Command::new("script")
        .args(["-q", "/dev/null", "sh", "-c", &command])
        .env("PATH", path)
        .env_remove("SAM_STACK_WORK")
        .envs(env.iter().copied())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(keys.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    (out.status.success(), String::from_utf8_lossy(&out.stdout).into_owned())
}

#[test]
fn shows_the_tree_and_today_then_restores_the_terminal() {
    let dir = temp("work");
    journal(&dir.join("work"));
    let (ok, screen) = drive(&dir, &[("SAM_STACK_WORK", &dir.join("work"))], "jq");
    assert!(ok, "{screen}");
    assert!(screen.contains("\x1b[?1049h"), "{screen:?}");
    for text in ["parser", "working", "done", "gone", "Today", "tests"] {
        assert!(screen.contains(text), "{text} missing from {screen:?}");
    }
    assert!(screen.ends_with("\x1b[?1049l\x1b[?25h"), "{screen:?}");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn without_sam_stack_work_it_reads_the_home_default() {
    let dir = temp("home");
    journal(&dir.join(".sam_stack/work"));
    let (ok, screen) = drive(&dir, &[("HOME", &dir)], "q");
    assert!(ok, "{screen}");
    assert!(screen.contains("tests"), "{screen:?}");
    fs::remove_dir_all(dir).unwrap();
}
