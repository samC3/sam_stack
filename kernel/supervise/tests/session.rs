use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const FAKE_HERDR: &str = r#"#!/bin/sh
printf '%s\n' "$*" >> "$FAKE_LOG"
case "$1 $2" in
  "tab create"|"worktree create")
    [ -z "$FAKE_FAIL" ] || { echo "$FAKE_FAIL" >&2; exit 1; }
    [ -z "$FAKE_NO_PANE" ] || { echo '{"result":{}}'; exit 0; }
    echo '{"result":{"root_pane":{"pane_id":"w1:p9"}}}' ;;
  "agent get")
    case ",$FAKE_GONE," in *",$3,"*) echo agent_not_found; exit 1 ;; esac
    echo "{\"result\":{\"agent\":{\"agent_status\":\"${FAKE_STATUS:-working}\"}}}" ;;
  "agent wait")
    [ "$3" = "$FAKE_DONE" ] || exec sleep 30
    echo '{"result":{"agent":{"agent_status":"done"}}}' ;;
  *) echo '{"result":{}}' ;;
esac
"#;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Env {
    tmp: PathBuf,
}

impl Env {
    fn new() -> Env {
        let n = NEXT.fetch_add(1, Ordering::SeqCst);
        let tmp = std::env::temp_dir().join(format!("session-test-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("bin")).unwrap();
        fs::write(tmp.join("bin/herdr"), FAKE_HERDR).unwrap();
        Command::new("chmod")
            .arg("+x")
            .arg(tmp.join("bin/herdr"))
            .status()
            .unwrap();
        fs::write(tmp.join("herdr.log"), "").unwrap();
        Env {
            tmp: tmp.canonicalize().unwrap(),
        }
    }

    fn run(&self, args: &[&str], env: &[(&str, &str)]) -> (i32, String) {
        let path = format!("{}:{}", self.tmp.join("bin").display(), std::env::var("PATH").unwrap());
        let out = Command::new(env!("CARGO_BIN_EXE_session"))
            .args(args)
            .current_dir(&self.tmp)
            .env("PATH", path)
            .env("FAKE_LOG", self.tmp.join("herdr.log"))
            .env("HERDR_ENV", "1")
            .env("HERDR_WORKSPACE_ID", "w1")
            .envs(env.iter().copied())
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
        (out.status.code().unwrap_or(-1), text)
    }

    fn brief(&self, rel: &str) -> String {
        let folder = self.tmp.join(rel);
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("brief.md"), "Do the thing.\n").unwrap();
        folder.display().to_string()
    }

    fn calls(&self) -> String {
        fs::read_to_string(self.tmp.join("herdr.log")).unwrap()
    }

    fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.tmp.join(rel)).unwrap()
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.tmp);
    }
}

#[test]
fn start_opens_a_tab_and_starts_claude() {
    let e = Env::new();
    let folder = e.brief("w/sessions/parser");
    let (code, out) = e.run(&["start", &folder], &[]);
    assert_eq!(code, 0, "{out}");
    let calls = e.calls();
    let here = e.tmp.display().to_string();
    assert!(
        calls.contains(&format!(
            "tab create --workspace w1 --cwd {here} --label parser --no-focus"
        )),
        "{calls}"
    );
    assert!(
        calls.contains("agent start parser --kind claude --pane w1:p9 -- --session-id "),
        "{calls}"
    );
    assert!(calls.contains(" -n parser\n"), "{calls}");
    let meta: serde_json::Value = serde_json::from_str(&e.read("w/sessions/parser/meta.json")).unwrap();
    assert_eq!(meta["name"], "parser");
    assert_eq!(meta["pane"], "w1:p9");
    assert!(meta["branch"].is_null());
    let id = meta["session_id"].as_str().unwrap();
    assert_eq!(id.len(), 36);
    assert_eq!(id.as_bytes()[14], b'4');
    assert!(matches!(id.as_bytes()[19], b'8' | b'9' | b'a' | b'b'), "{id}");
    assert!(id.chars().all(|c| c == '-' || c.is_ascii_hexdigit()));
    assert!(calls.contains(&format!("--session-id {id} -n parser")), "{calls}");
    assert_eq!(meta["started"].as_str().unwrap().len(), 19);
    assert!(
        out.starts_with(&format!("started parser in pane w1:p9 (claude --resume {id})")),
        "{out}"
    );
    let sent = e.read("w/sessions/parser/sent.md");
    let real = e.tmp.join("w/sessions/parser");
    assert!(sent.starts_with("## 20"), "{sent}");
    assert!(
        sent.contains(&format!("Your folder is {}: use it as your task state", real.display())),
        "{sent}"
    );
    assert!(sent.ends_with("Read brief.md there and follow it.\n\n"), "{sent}");
    assert!(
        calls.contains("agent prompt parser You are a supervised session."),
        "{calls}"
    );
}

#[test]
fn start_with_edit_gets_a_worktree_and_branch() {
    let e = Env::new();
    let (code, out) = e.run(
        &["start", &e.brief("w/sessions/parser"), "--edit", "--cwd", "/repo"],
        &[],
    );
    assert_eq!(code, 0, "{out}");
    assert!(
        e.calls()
            .contains("worktree create --cwd /repo --branch supervise/parser --label parser --no-focus")
    );
    assert!(!e.calls().contains("tab create"));
    assert!(
        e.read("w/sessions/parser/meta.json")
            .contains("\"branch\": \"supervise/parser\"")
    );
}

#[test]
fn start_needs_a_brief() {
    let e = Env::new();
    fs::create_dir_all(e.tmp.join("w/sessions/parser")).unwrap();
    let (code, out) = e.run(&["start", &e.tmp.join("w/sessions/parser").display().to_string()], &[]);
    assert_eq!(code, 1);
    assert!(out.contains("brief.md first"), "{out}");
}

#[test]
fn start_twice_is_refused() {
    let e = Env::new();
    let folder = e.brief("w/sessions/parser");
    e.run(&["start", &folder], &[]);
    let (code, out) = e.run(&["start", &folder], &[]);
    assert_eq!(code, 1);
    assert!(out.contains("already started; use send"), "{out}");
}

#[test]
fn start_reports_herdr_failures() {
    let e = Env::new();
    let folder = e.brief("w/sessions/parser");
    let (code, out) = e.run(&["start", &folder], &[("FAKE_FAIL", "no such workspace")]);
    assert_eq!(code, 1);
    assert_eq!(out, "session: herdr tab create failed: no such workspace\n");
    let (code, out) = e.run(&["start", &folder], &[("FAKE_NO_PANE", "1")]);
    assert_eq!(code, 1);
    assert!(out.contains("herdr gave no root pane id"), "{out}");
    assert!(!e.tmp.join("w/sessions/parser/meta.json").exists());
}

#[test]
fn herdr_missing_is_an_error() {
    let e = Env::new();
    let folder = e.brief("w/sessions/parser");
    let (code, out) = e.run(&["start", &folder], &[("PATH", "/nonexistent")]);
    assert_eq!(code, 1);
    assert!(out.starts_with("session: herdr: "), "{out}");
}

#[test]
fn nested_sessions_join_their_path() {
    let e = Env::new();
    let (code, out) = e.run(&["start", &e.brief("w/sessions/parser/sessions/lexer")], &[]);
    assert_eq!(code, 0, "{out}");
    assert!(e.calls().contains("agent start parser-lexer "));
}

#[test]
fn names_must_suit_herdr() {
    let e = Env::new();
    for bad in [
        "w/sessions/Parser",
        "w/sessions/9lives",
        "w/sessions/a.b",
        "w/sessions/abcdefghijklmnopqrstuvwxyz0123456",
    ] {
        let (code, out) = e.run(&["start", &e.brief(bad)], &[]);
        assert_eq!(code, 1, "{bad}");
        assert!(out.contains("must match [a-z][a-z0-9_-]{0,31}"), "{bad}: {out}");
    }
    let (code, out) = e.run(&["start", &e.brief("w/sessions/abcdefghijklmnopqrstuvwxyz012345")], &[]);
    assert_eq!(code, 0, "{out}");
    let (code, out) = e.run(&["start", &e.brief("w/plain")], &[]);
    assert_eq!(code, 1);
    assert!(out.contains("is not under a sessions/ folder"), "{out}");
    let (code, out) = e.run(&["start", "w/sessions/missing"], &[]);
    assert_eq!(code, 1);
    assert!(out.contains("brief.md first"), "{out}");
}

#[test]
fn send_prompts_and_logs() {
    let e = Env::new();
    let folder = e.brief("w/sessions/parser");
    e.run(&["start", &folder], &[]);
    let (code, out) = e.run(&["send", &folder, "Test 3 still fails: see P2."], &[]);
    assert_eq!(code, 0, "{out}");
    assert!(e.calls().contains("agent prompt parser Test 3 still fails: see P2.\n"));
    assert!(
        e.read("w/sessions/parser/sent.md")
            .ends_with("\nTest 3 still fails: see P2.\n\n")
    );
}

#[test]
fn send_needs_a_started_session() {
    let e = Env::new();
    let folder = e.brief("w/sessions/parser");
    let (code, out) = e.run(&["send", &folder, "hi"], &[]);
    assert_eq!(code, 1);
    assert!(out.contains("has no meta.json; start it first"), "{out}");
    fs::write(e.tmp.join("w/sessions/parser/meta.json"), "not json").unwrap();
    let (code, out) = e.run(&["send", &folder, "hi"], &[]);
    assert_eq!(code, 1);
    assert!(out.contains("meta.json: "), "{out}");
}

#[test]
fn status_prints_the_tree() {
    let e = Env::new();
    let (parser, lexer) = (
        e.brief("w/sessions/parser"),
        e.brief("w/sessions/parser/sessions/lexer"),
    );
    e.run(&["start", &parser], &[]);
    e.run(&["start", &lexer], &[]);
    fs::write(Path::new(&parser).join("return.md"), "Status: done\nids: P1\n").unwrap();
    let w = e.tmp.join("w").display().to_string();
    let (code, out) = e.run(
        &["status", &w],
        &[("FAKE_STATUS", "idle"), ("FAKE_GONE", "parser-lexer")],
    );
    assert_eq!(code, 0, "{out}");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2, "{out}");
    assert_eq!(lines[0], format!("  {:24} {:8} return: done", "parser", "idle"));
    let lexer_id = serde_json::from_str::<serde_json::Value>(&e.read("w/sessions/parser/sessions/lexer/meta.json"))
        .unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        lines[1],
        format!(
            "    {:24} {:8} return: -  (claude --resume {lexer_id})",
            "parser-lexer", "gone"
        )
    );
    let (_, out) = e.run(&["status", &parser], &[("FAKE_STATUS", "idle")]);
    assert!(out.starts_with("parser "), "{out}");
    let sessions = e.tmp.join("w/sessions").display().to_string();
    let (_, out) = e.run(&["status", &sessions], &[("FAKE_STATUS", "idle")]);
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines[0].starts_with("parser "), "{out}");
    assert!(lines[1].starts_with("  parser-lexer "), "{out}");
}

#[test]
fn status_of_a_return_without_a_status_line() {
    let e = Env::new();
    let folder = e.brief("w/sessions/parser");
    e.run(&["start", &folder], &[]);
    fs::write(Path::new(&folder).join("return.md"), "answer: none\n").unwrap();
    let (_, out) = e.run(&["status", &folder], &[]);
    assert!(out.contains("working  return: ?"), "{out}");
}

#[test]
fn a_return_older_than_the_last_message_is_stale() {
    let e = Env::new();
    let folder = e.brief("w/sessions/parser");
    e.run(&["start", &folder], &[]);
    let ret = Path::new(&folder).join("return.md");
    fs::write(&ret, "status: done\n").unwrap();
    Command::new("touch")
        .args(["-t", "200001010000"])
        .arg(&ret)
        .status()
        .unwrap();
    let (_, out) = e.run(&["status", &folder], &[("FAKE_STATUS", "idle")]);
    assert!(out.contains("return: stale (done)"), "{out}");
    fs::write(&ret, "status: done\n").unwrap();
    let (_, out) = e.run(&["status", &folder], &[("FAKE_STATUS", "idle")]);
    assert!(out.contains("return: done") && !out.contains("stale"), "{out}");
    Command::new("touch")
        .arg("-r")
        .arg(Path::new(&folder).join("sent.md"))
        .arg(&ret)
        .status()
        .unwrap();
    let (_, out) = e.run(&["status", &folder], &[("FAKE_STATUS", "idle")]);
    assert!(out.contains("return: done") && !out.contains("stale"), "{out}");
    fs::remove_file(Path::new(&folder).join("sent.md")).unwrap();
    Command::new("touch")
        .args(["-t", "200001010000"])
        .arg(&ret)
        .status()
        .unwrap();
    let (_, out) = e.run(&["status", &folder], &[("FAKE_STATUS", "idle")]);
    assert!(out.contains("return: done") && !out.contains("stale"), "{out}");
}

#[test]
fn wait_returns_the_first_to_settle() {
    let e = Env::new();
    let (a, b) = (e.brief("w/sessions/a"), e.brief("w/sessions/b"));
    e.run(&["start", &a], &[]);
    e.run(&["start", &b], &[]);
    fs::write(Path::new(&b).join("return.md"), "status: done\n").unwrap();
    let (code, out) = e.run(&["wait", &a, &b], &[("FAKE_DONE", "b")]);
    assert_eq!((code, out.as_str()), (0, "b is done; return: done\n"));
    let (code, out) = e.run(&["wait", &b], &[("FAKE_DONE", "b")]);
    assert_eq!((code, out.as_str()), (0, "b is done; return: done\n"));
}

#[test]
fn wait_needs_started_sessions() {
    let e = Env::new();
    let (code, out) = e.run(&["wait", &e.brief("w/sessions/a")], &[]);
    assert_eq!(code, 1);
    assert!(out.contains("start it first"), "{out}");
    let (code, out) = e.run(&["wait"], &[]);
    assert_eq!(code, 2, "{out}");
}

#[test]
fn wait_with_unreadable_output_reports_unknown() {
    let e = Env::new();
    let a = e.brief("w/sessions/a");
    e.run(&["start", &a], &[]);
    fs::write(e.tmp.join("bin/herdr"), "#!/bin/sh\necho garbage\n").unwrap();
    let (code, out) = e.run(&["wait", &a], &[]);
    assert_eq!((code, out.as_str()), (0, "a is unknown; return: -\n"));
}

#[test]
fn wait_without_herdr_is_an_error() {
    let e = Env::new();
    let a = e.brief("w/sessions/a");
    e.run(&["start", &a], &[]);
    let (code, out) = e.run(&["wait", &a], &[("PATH", "/nonexistent")]);
    assert_eq!(code, 1);
    assert!(out.starts_with("session: herdr: "), "{out}");
}

#[test]
fn outside_herdr_it_refuses() {
    let e = Env::new();
    let (code, out) = e.run(&["status", &e.tmp.display().to_string()], &[("HERDR_ENV", "")]);
    assert_eq!(code, 1);
    assert!(out.contains("not inside a herdr pane"), "{out}");
}
