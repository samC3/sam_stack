use std::process::{Command, Output};

mod common;
use common::TempDir;

fn evals(args: &[&str], cwd: &std::path::Path, env: &[(&str, &str)]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_evals"))
        .args(args)
        .current_dir(cwd)
        .envs(env.iter().copied())
        .output()
        .unwrap()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn git(dir: &std::path::Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap()
        .success();
    assert!(ok);
}

#[test]
fn lint_reports_errors_and_exits_1() {
    let dir = TempDir::new();
    dir.write("kernel/sample/SKILL.md", "no frontmatter\n");
    let out = evals(&["lint", &dir.path().display().to_string()], dir.path(), &[]);
    assert_eq!(out.status.code(), Some(1));
    let text = stdout(&out);
    assert!(
        text.contains("kernel/sample/SKILL.md: frontmatter name must be 'sample'"),
        "{text}"
    );
    assert!(text.ends_with("lint: 9 error(s)\n"), "{text}");
}

#[test]
fn lint_ok_exits_0() {
    let dir = TempDir::new();
    let out = evals(&["lint", &dir.path().display().to_string()], dir.path(), &[]);
    assert_eq!((out.status.code(), stdout(&out).as_str()), (Some(0), "lint: ok\n"));
}

#[test]
fn lint_defaults_to_this_checkout() {
    let dir = TempDir::new();
    let out = evals(&["lint"], dir.path(), &[]);
    assert!(stdout(&out).contains("lint: "), "{}", stdout(&out));
}

#[test]
fn invoked_exit_codes() {
    let dir = TempDir::new();
    let path = dir.write(
        "t.jsonl",
        "{\"type\":\"assistant\",\"message\":{\"content\":[{\"name\":\"Skill\",\"input\":{\"skill\":\"ground\"}}]}}\n",
    );
    let t = path.display().to_string();
    assert_eq!(
        evals(&["invoked", &t, "ground"], dir.path(), &[]).status.code(),
        Some(0)
    );
    assert_eq!(evals(&["invoked", &t, "prove"], dir.path(), &[]).status.code(), Some(1));
    let missing = evals(&["invoked", "nope.jsonl", "prove"], dir.path(), &[]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&missing.stderr).starts_with("evals: nope.jsonl: "));
}

#[test]
fn comments_lists_added_comment_lines() {
    let dir = TempDir::new();
    dir.write("a.py", "x = 1\n");
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-qm", "base"]);
    assert_eq!(evals(&["comments", "HEAD"], dir.path(), &[]).status.code(), Some(0));
    dir.write("a.py", "x = 1\n# why\n");
    let out = evals(&["comments", "HEAD"], dir.path(), &[]);
    assert_eq!((out.status.code(), stdout(&out).as_str()), (Some(1), "a.py:2 # why\n"));
    assert_eq!(evals(&["comments", "nope"], dir.path(), &[]).status.code(), Some(2));
}

#[test]
fn run_rejects_an_unknown_task_and_a_missing_root() {
    let dir = TempDir::new();
    dir.write("evals/tasks/one/task.md", "---\ncovers: x\n---\n");
    let evals_dir = dir.path().join("evals").display().to_string();
    let out = evals(&["run", "--evals", &evals_dir, "nope"], dir.path(), &[]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&out.stderr), "evals: no task named 'nope'\n");
    let out = evals(&["graders", "--root", "/nonexistent/skills"], dir.path(), &[]);
    assert!(String::from_utf8_lossy(&out.stderr).starts_with("evals: /nonexistent/skills: "));
}

#[test]
fn graders_run_every_task_into_evals_out() {
    let dir = TempDir::new();
    dir.write("evals/lib.sh", include_str!("../lib.sh"));
    dir.write("evals/tasks/one/task.md", "---\ncovers: x\n---\n");
    dir.write("evals/tasks/one/checks.sh", "check \"made\" test -f made.txt\n");
    dir.write("evals/tasks/one/reference/apply.sh", "touch made.txt\n");
    dir.write("evals/tasks/two/task.md", "---\ncovers: x\n---\n");
    dir.write("kernel/KERNEL.md", "# Kernel\n");
    let evals_dir = dir.path().join("evals").display().to_string();
    let out_dir = dir.path().join("runs").display().to_string();
    let env = [
        ("EVALS_OUT", out_dir.as_str()),
        ("GIT_AUTHOR_NAME", "t"),
        ("GIT_AUTHOR_EMAIL", "t@t"),
        ("GIT_COMMITTER_NAME", "t"),
        ("GIT_COMMITTER_EMAIL", "t@t"),
    ];
    let out = evals(&["graders", "--evals", &evals_dir], dir.path(), &env);
    let text = stdout(&out);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{text}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        text.contains("one                        PASS  untouched fails 1/1  reference passes 1/1"),
        "{text}"
    );
    assert!(text.contains("two                        SKIP"), "{text}");
    assert!(text.contains(&format!("Runs: {out_dir}/")), "{text}");
    let runs: Vec<_> = std::fs::read_dir(dir.path().join("runs")).unwrap().flatten().collect();
    let name = runs[0].file_name().to_string_lossy().into_owned();
    assert!(name.len() > 16 && name.as_bytes()[8] == b'-', "{name}");
}

#[test]
fn run_out_dir_defaults_to_the_cache() {
    let dir = TempDir::new();
    dir.write("evals/tasks/one/task.md", "---\ncovers: x\n---\n");
    let evals_dir = dir.path().join("evals").display().to_string();
    let home = dir.path().join("home").display().to_string();
    let cache = dir.path().join("cache").display().to_string();
    let out = Command::new(env!("CARGO_BIN_EXE_evals"))
        .args(["run", "--evals", &evals_dir, "--repeat", "0"])
        .env_remove("EVALS_OUT")
        .env("XDG_CACHE_HOME", &cache)
        .output()
        .unwrap();
    assert!(
        stdout(&out).contains(&format!("Runs: {cache}/sam_stack-evals/")),
        "{}",
        stdout(&out)
    );
    let out = Command::new(env!("CARGO_BIN_EXE_evals"))
        .args(["run", "--evals", &evals_dir, "--repeat", "0"])
        .env_remove("EVALS_OUT")
        .env_remove("XDG_CACHE_HOME")
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(
        stdout(&out).contains(&format!("Runs: {home}/.cache/sam_stack-evals/")),
        "{}",
        stdout(&out)
    );
    assert!(stdout(&out).contains("Summary (passing trials / trials):\n  one                        0/0"));
}
