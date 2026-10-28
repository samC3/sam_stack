use super::*;
use crate::testing::TempDir;

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
    dir.write("evals/lib.sh", include_str!("../../lib.sh"));
    dir.write("evals/prompts/grader.md", include_str!("../../prompts/grader.md"));
    dir.write("evals/prompts/user.md", include_str!("../../prompts/user.md"));
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
    .collect();
    let ctx = Ctx {
        evals: root.join("evals"),
        skills: root.join("skills"),
        out: root.join("out"),
        exe: PathBuf::from("/nonexistent/evals"),
        env,
    };
    Setup { dir, ctx }
}

#[test]
fn forget_refuses_paths_outside_the_run_folder() {
    let s = setup();
    fs::create_dir_all(&s.ctx.out).unwrap();
    let t = Trial::new(&s.ctx, "one", s.ctx.out.join("x"));
    assert!(t.forget_sessions_in(s.dir.path()).unwrap_err().contains("refusing"));
    assert!(t.forget_sessions_in(&s.ctx.out).unwrap_err().contains("refusing"));
}

#[test]
fn fill_replaces_only_named_placeholders() {
    assert_eq!(
        fill("a {x} {y} {z} {", &[("x", "1 {y}"), ("y", "2")]),
        "a 1 {y} 2 {z} {"
    );
}

#[test]
fn verdict_lines() {
    assert!(is_verdict("1: PASS — ok"));
    assert!(is_verdict("12: UNKNOWN"));
    assert!(is_verdict("3: FAIL: no"));
    assert!(!is_verdict("1: PASSED"));
    assert!(!is_verdict(": PASS"));
    assert!(!is_verdict("a1: PASS"));
    assert!(!is_verdict("1: MAYBE"));
    assert!(!is_verdict("1 PASS"));
}

#[test]
fn trial_env_and_turns_come_from_the_task() {
    let s = setup();
    let t = Trial::new(&s.ctx, "one", s.ctx.out.join("x"));
    assert_eq!(t.max_turns, 3);
    assert_eq!(t.env.get("FOO").map(String::as_str), Some("bar"));
    assert!(!t.env.contains_key("BARE"));
    assert!(!t.env.contains_key("HERDR_ENV"));
    assert!(t.env["PATH"].starts_with(&s.dir.path().join("evals/tasks/one/bin").display().to_string()));
    assert_eq!(t.env["ROOT"], s.dir.path().display().to_string());
    assert!(t.env["WORK"].ends_with("x/work/repo"));
    let bare = Trial::new(&s.ctx, "missing", s.ctx.out.join("y"));
    assert_eq!(bare.max_turns, 8);
}
