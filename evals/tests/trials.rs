use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use evals::eval::{Ctx, run_all, run_trial, test_graders};

mod common;
use common::TempDir;

const FAKE_CLAUDE: &str = r#"#!/bin/sh
case "$2" in
  "You are grading"*)
    echo "1: PASS — fine"
    echo "2: UNKNOWN — unsure"
    echo "10: PASSED is not a verdict"
    echo "chatter" ;;
  "You are role-playing"*)
    n=$(cat count 2>/dev/null || echo 0)
    echo $((n + 1)) > count
    if [ "$n" -ge "${FAKE_USER_TURNS:-1}" ]; then echo "<<END>>"; else echo "Use CSV."; fi ;;
  *)
    case " $* " in *" --resume s1 "*) echo resumed >> "$FAKE_CALLS" ;; esac
    touch made.txt
    printf 'a\000b' > blob.bin
    printf 'a\000b' > "$SAM_STACK_WORK/blob.bin"
    printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"text","text":"Hi"},{"type":"tool_use","name":"Skill","input":{"skill":"ground"}}]}}'
    [ -n "$FAKE_NO_SESSION" ] || printf '%s\n' '{"type":"result","session_id":"s1"}'
    echo "agent stderr" >&2 ;;
esac
"#;

const CHECKS: &str = r#"check "fixture copied"   test -f app.txt
check "made"             test -f made.txt
check "env from task"    test "$FOO" = bar
check "no herdr"         test -z "$HERDR_ENV"
check "bin on path"      tool
check "state copied"     has "$WORK/s/brief.md" "^Goal"
check "invoked ground"   invoked ground
"#;

struct Setup {
    dir: TempDir,
    ctx: Ctx,
}

fn setup() -> Setup {
    let dir = TempDir::new();
    let root = dir.path();
    dir.write("evals/lib.sh", include_str!("../lib.sh"));
    dir.write("evals/prompts/grader.md", include_str!("../prompts/grader.md"));
    dir.write("evals/prompts/user.md", include_str!("../prompts/user.md"));
    let task = "evals/tasks/one";
    dir.write(
        &format!("{task}/task.md"),
        "---\ncovers: ground\nturns: 3\nenv: FOO=bar BARE\n---\n",
    );
    dir.write(&format!("{task}/prompt.md"), "Do it.\n");
    dir.write(&format!("{task}/user.md"), "You want CSV.\n");
    dir.write(&format!("{task}/rubric.md"), "1. Good.\n2. Clear.\n");
    dir.write(&format!("{task}/fixture/app.txt"), "app\n");
    dir.write(&format!("{task}/fixture/sub/deep.txt"), "deep\n");
    dir.write(&format!("{task}/work/s/brief.md"), "Goal: x\n");
    dir.write(&format!("{task}/setup.sh"), "git tag base\n");
    dir.write(&format!("{task}/checks.sh"), CHECKS);
    dir.write(&format!("{task}/reference/apply.sh"), "touch made.txt\n");
    dir.write(
        &format!("{task}/reference/transcript.jsonl"),
        "{\"type\":\"assistant\",\"message\":{\"content\":[{\"name\":\"Skill\",\"input\":{\"skill\":\"ground\"}}]}}\n",
    );
    let tool = dir.write(&format!("{task}/bin/tool"), "#!/bin/sh\nexit 0\n");
    let claude = dir.write("fakebin/claude", FAKE_CLAUDE);
    for script in [tool, claude] {
        Command::new("chmod").arg("+x").arg(script).status().unwrap();
    }
    dir.write("skills/kernel/KERNEL.md", "# Kernel\n");
    dir.write("skills/kernel/ground/SKILL.md", "---\nname: ground\n---\n");
    dir.write("skills/kernel/notes.txt", "not a skill dir\n");
    dir.write("skills/workflows/flow/SKILL.md", "---\nname: flow\n---\n");
    dir.write("home/.keep", "");
    let env: BTreeMap<String, String> = [
        ("PATH", format!("{}:/usr/bin:/bin", root.join("fakebin").display())),
        ("HOME", root.join("home").display().to_string()),
        ("HERDR_ENV", "1".into()),
        ("FAKE_CALLS", root.join("calls.log").display().to_string()),
        ("GIT_AUTHOR_NAME", "t".into()),
        ("GIT_AUTHOR_EMAIL", "t@t".into()),
        ("GIT_COMMITTER_NAME", "t".into()),
        ("GIT_COMMITTER_EMAIL", "t@t".into()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .chain(std::env::var("LLVM_PROFILE_FILE").map(|v| ("LLVM_PROFILE_FILE".to_string(), v)))
    .collect();
    let ctx = Ctx {
        evals: root.join("evals"),
        skills: root.join("skills"),
        out: root.join("out"),
        exe: PathBuf::from(env!("CARGO_BIN_EXE_evals")),
        env,
    };
    Setup { dir, ctx }
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn a_full_trial_runs_the_agent_user_checks_and_grader() {
    let s = setup();
    fs::create_dir_all(&s.ctx.out).unwrap();
    let repo = s.ctx.out.canonicalize().unwrap().join("one/r1/repo");
    let encoded: String = repo
        .display()
        .to_string()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let remembered = s.dir.path().join("home/.claude/projects").join(&encoded);
    fs::create_dir_all(&remembered).unwrap();

    let (passed, report) = run_trial(&s.ctx, "one", 1).unwrap();
    assert!(!passed, "{report}");
    assert_eq!(
        report,
        "one                        r1   FAIL  checks 7/7  rubric 1/2 (1 unknown)  turns 2\n    rubric 2: UNKNOWN — unsure"
    );
    let trial = s.ctx.out.join("one/r1");
    assert_eq!(
        read(&trial.join("conversation.md")),
        "AGENT:\nHi\n\nUSER:\nUse CSV.\n\n\nAGENT:\nHi\n\nUSER:\n<<END>>\n\n\n"
    );
    assert_eq!(read(&s.dir.path().join("calls.log")), "resumed\n");
    assert!(read(&trial.join("turn-2.jsonl")).contains("session_id"));
    assert_eq!(read(&trial.join("transcript.jsonl")).lines().count(), 4);
    assert!(read(&trial.join("digest.txt")).starts_with("ASSISTANT: Hi\nTOOL Skill: {\"skill\":\"ground\"}"));
    assert!(read(&trial.join("stderr.log")).contains("agent stderr"));
    assert_eq!(read(&trial.join("checks.txt")).lines().count(), 7);
    let outcome = read(&trial.join("outcome.txt"));
    assert!(outcome.contains("=== state: repo/s/brief.md\nGoal: x"), "{outcome}");
    assert!(outcome.contains("=== repo: git diff since setup\n"), "{outcome}");
    assert!(outcome.contains("=== repo: made.txt (new)"), "{outcome}");
    assert!(!outcome.contains(".claude/skills"), "{outcome}");
    assert!(
        outcome.contains("=== repo: blob.bin (new)\n(binary, 3 bytes)"),
        "{outcome}"
    );
    assert!(outcome.contains("=== state: blob.bin\n(binary, 3 bytes)"), "{outcome}");
    assert!(read(&trial.join("grade.md")).contains("chatter"));
    assert!(repo.join(".claude/skills/ground").is_symlink());
    assert!(repo.join(".claude/skills/flow").is_symlink());
    assert!(!repo.join(".claude/skills/notes.txt").exists());
    assert_eq!(read(&repo.join("AGENTS.md")), "@KERNEL.md\n");
    assert_eq!(read(&repo.join("sub/deep.txt")), "deep\n");
    assert!(!remembered.exists());
}

#[test]
fn the_simulated_user_stops_at_max_turns() {
    let mut s = setup();
    s.ctx.env.insert("FAKE_USER_TURNS".into(), "99".into());
    let (_, report) = run_trial(&s.ctx, "one", 1).unwrap();
    assert!(report.contains("turns 3"), "{report}");
}

#[test]
fn no_session_id_ends_the_conversation() {
    let mut s = setup();
    s.ctx.env.insert("FAKE_NO_SESSION".into(), "1".into());
    let (_, report) = run_trial(&s.ctx, "one", 1).unwrap();
    assert!(report.contains("turns 1"), "{report}");
}

#[test]
fn single_turn_task_without_rubric_passes_on_checks() {
    let s = setup();
    fs::remove_file(s.dir.path().join("evals/tasks/one/user.md")).unwrap();
    fs::remove_file(s.dir.path().join("evals/tasks/one/rubric.md")).unwrap();
    let (passed, report) = run_trial(&s.ctx, "one", 2).unwrap();
    assert!(passed, "{report}");
    assert_eq!(
        report,
        "one                        r2   PASS  checks 7/7  rubric 0/0  turns 1"
    );
    assert!(!s.ctx.out.join("one/r2/grade.md").exists());
}

#[test]
fn failed_checks_and_a_work_folder_are_listed() {
    let s = setup();
    fs::remove_file(s.dir.path().join("evals/tasks/one/rubric.md")).unwrap();
    s.dir.write("evals/tasks/one/fixture/work/x.md", "state in the repo\n");
    s.dir
        .write("evals/tasks/one/checks.sh", "check \"missing\" test -f nope.txt\n");
    let (passed, report) = run_trial(&s.ctx, "one", 1).unwrap();
    assert!(!passed);
    assert!(report.contains("checks 0/2"), "{report}");
    assert!(report.contains("\n    check  FAIL missing"), "{report}");
    assert!(
        report.contains("\n    check  FAIL task state stays out of the repo"),
        "{report}"
    );
}

#[test]
fn an_unreadable_grader_fails_the_rubric() {
    let s = setup();
    s.dir.write("evals/prompts/grader.md", "Say nothing useful.\n");
    let (_, report) = run_trial(&s.ctx, "one", 1).unwrap();
    assert!(report.contains("rubric 0/1"), "{report}");
    assert!(report.contains("0: FAIL — grader output unreadable"), "{report}");
}

#[test]
fn a_broken_setup_is_an_error() {
    let s = setup();
    s.dir.write("evals/tasks/one/setup.sh", "echo broken >&2; exit 3\n");
    let err = run_trial(&s.ctx, "one", 1).unwrap_err();
    assert!(err.contains("setup.sh failed: broken"), "{err}");
}

#[test]
fn graders_fail_untouched_and_pass_on_the_reference() {
    let s = setup();
    let (passed, report) = test_graders(&s.ctx, "one").unwrap();
    assert!(passed, "{report}");
    assert_eq!(
        report,
        "one                        PASS  untouched fails 2/7  reference passes 7/7"
    );
}

#[test]
fn graders_report_reference_failures() {
    let s = setup();
    s.dir.write("evals/tasks/one/reference/apply.sh", "true\n");
    let (passed, report) = test_graders(&s.ctx, "one").unwrap();
    assert!(!passed);
    assert!(report.ends_with("\n    reference FAIL made: "), "{report}");
}

#[test]
fn graders_fail_when_nothing_fails_untouched() {
    let s = setup();
    s.dir.write("evals/tasks/one/checks.sh", "check \"always\" true\n");
    let (passed, _) = test_graders(&s.ctx, "one").unwrap();
    assert!(!passed);
}

#[test]
fn graders_skip_a_task_without_reference() {
    let s = setup();
    fs::remove_file(s.dir.path().join("evals/tasks/one/reference/apply.sh")).unwrap();
    assert_eq!(
        test_graders(&s.ctx, "one").unwrap(),
        (true, "one                        SKIP  no reference/apply.sh".into())
    );
}

#[test]
fn run_all_reports_every_trial_and_errors() {
    let mut s = setup();
    fs::remove_file(s.dir.path().join("evals/tasks/one/user.md")).unwrap();
    fs::remove_file(s.dir.path().join("evals/tasks/one/rubric.md")).unwrap();
    s.dir.write("evals/tasks/two/task.md", "---\ncovers: ground\n---\n");
    s.dir.write("evals/tasks/two/setup.sh", "exit 1\n");
    let tasks = s.ctx.tasks();
    assert_eq!(tasks, ["one", "two"]);
    assert!(!run_all(&s.ctx, &tasks, 2, 3, false).unwrap());
    let results = read(&s.ctx.out.join("results.txt"));
    assert_eq!(results.lines().filter(|l| l.contains(" PASS ")).count(), 2, "{results}");
    assert_eq!(
        results
            .lines()
            .filter(|l| l.starts_with("two") && l.contains("ERROR"))
            .count(),
        2,
        "{results}"
    );
    assert!(
        results.ends_with(
            "\nSummary (passing trials / trials):\n  one                        2/2\n  two                        0/2\n"
        ),
        "{results}"
    );
    s.ctx.out = s.dir.path().join("out2");
    assert!(run_all(&s.ctx, &tasks[..1], 1, 0, false).unwrap());
}

#[test]
fn run_all_graders_mode() {
    let s = setup();
    assert!(run_all(&s.ctx, &["one".to_string()], 5, 4, true).unwrap());
    assert_eq!(read(&s.ctx.out.join("results.txt")).lines().count(), 1);
}
