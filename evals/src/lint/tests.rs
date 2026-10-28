use super::*;
use crate::testing::TempDir;

const SKILL: &str = "kernel/sample/SKILL.md";
const TASK_MD: &str = "evals/tasks/sample-task/task.md";
const PROMPT: &str = "evals/tasks/sample-task/prompt.md";
const CHECKS: &str = "evals/tasks/sample-task/checks.sh";
const APPLY: &str = "evals/tasks/sample-task/reference/apply.sh";
const TRANSCRIPT: &str = "evals/tasks/sample-task/reference/transcript.jsonl";
const RUBRIC: &str = "evals/tasks/sample-task/rubric.md";
const KERNEL: &str = "kernel/KERNEL.md";
const STOP: &str = "Stop:      the entry is written";

const REPO: [(&str, &str); 7] = [
    (
        KERNEL,
        "# Kernel\n\n```registry\n# file      writer    ids\nnotes.md    sample    N1, …\n```\n",
    ),
    (
        SKILL,
        "---\nname: sample\ndescription: A minimal verb used only by the lint tests.\n---\n\n# Sample\n\n\
         Appends to $WORK/<slug>/notes.md.\n\n## Contract\n\n```contract\nLayer:     verb\nInput:     a question\n\
         Output:    entry N<n> appended to $WORK/<slug>/notes.md\nStop:      the entry is written\n\
         Returns:   the answer\n```\n",
    ),
    (TASK_MD, "---\ncovers: sample\nturns: 1\n---\n\n# sample-task\n"),
    (PROMPT, "Do the sample thing.\n"),
    (
        CHECKS,
        "check \"sample written\" test -f sample.md\ncheck \"invoked sample\" invoked sample\n",
    ),
    (APPLY, "touch sample.md\n"),
    (
        TRANSCRIPT,
        "{\"type\": \"assistant\", \"message\": {\"content\": [{\"type\": \"tool_use\", \"name\": \"Skill\", \"input\": {\"skill\": \"sample\"}}]}}\n",
    ),
];

type Changes = Vec<(&'static str, Option<String>)>;

fn base(path: &str) -> &'static str {
    REPO.iter().find(|(p, _)| *p == path).map(|(_, t)| *t).unwrap()
}

fn sub(path: &'static str, old: &str, new: &str) -> Changes {
    assert!(base(path).contains(old), "test setup: {old:?} not in {path}");
    vec![(path, Some(base(path).replacen(old, new, 1)))]
}

fn add(path: &'static str, text: &str) -> Changes {
    vec![(path, Some(text.to_string()))]
}

fn delete(path: &'static str) -> Changes {
    vec![(path, None)]
}

fn lint_with(changes: &Changes) -> Vec<String> {
    let dir = TempDir::new();
    let mut files: BTreeMap<&str, Option<String>> = REPO.iter().map(|(p, t)| (*p, Some((*t).to_string()))).collect();
    files.extend(changes.iter().cloned());
    for (path, text) in &files {
        if let Some(text) = text {
            dir.write(path, text);
        }
    }
    lint(dir.path())
}

fn expect(name: &str, changes: &Changes, expected: &str) {
    let errors = lint_with(changes);
    let ok = if expected == "ok" {
        errors.is_empty()
    } else {
        errors.iter().any(|e| e.contains(expected))
    };
    assert!(ok, "{name}: expected {expected:?}, lint said {errors:#?}");
}

#[test]
fn valid() {
    expect("valid", &vec![], "ok");
}

#[test]
fn frontmatter_rules() {
    expect(
        "no-frontmatter",
        &sub(SKILL, "---\nname: sample", "name: sample"),
        "name must be 'sample'",
    );
    expect(
        "unclosed-frontmatter",
        &sub(SKILL, "---\n\n# Sample", "\n# Sample"),
        "name must be 'sample'",
    );
    expect(
        "name-mismatch",
        &sub(SKILL, "name: sample", "name: sampel"),
        "name must be 'sample'",
    );
    expect(
        "no-description",
        &sub(
            SKILL,
            "description: A minimal verb used only by the lint tests.",
            "description:",
        ),
        "no description",
    );
    expect(
        "no-description-key",
        &sub(SKILL, "description: A minimal verb used only by the lint tests.\n", ""),
        "no description",
    );
}

#[test]
fn contract_rules() {
    expect(
        "no-contract-block",
        &sub(SKILL, "```contract", "```"),
        "no ```contract block",
    );
    expect(
        "unclosed-contract",
        &sub(SKILL, "Returns:   the answer\n```\n", "Returns:   the answer\n"),
        "no ```contract block",
    );
    expect(
        "missing-field",
        &sub(SKILL, &format!("{STOP}\n"), ""),
        "'Stop' is missing or empty",
    );
    expect("blank-field", &sub(SKILL, STOP, "Stop:"), "'Stop' is missing or empty");
    expect(
        "tab-only-field",
        &sub(SKILL, STOP, "Stop:\t"),
        "'Stop' is missing or empty",
    );
    expect(
        "tab-aligned-field",
        &sub(SKILL, STOP, "Stop:\tthe entry is written"),
        "ok",
    );
    expect(
        "unknown-layer",
        &sub(SKILL, "Layer:     verb", "Layer:     helper"),
        "must be verb, combinator or workflow",
    );
    expect(
        "layer-in-wrong-folder",
        &sub(SKILL, "Layer:     verb", "Layer:     workflow"),
        "belongs in workflows/",
    );
    expect(
        "combinator-in-kernel",
        &sub(SKILL, "Layer:     verb", "Layer:     combinator"),
        "ok",
    );
}

#[test]
fn workflow_layer_lives_in_workflows() {
    let skill = base(SKILL).replace("Layer:     verb", "Layer:     workflow");
    let task = base(TASK_MD).replace("covers: sample", "covers: flow");
    let checks = base(CHECKS).replace("invoked sample", "invoked flow");
    let changes: Changes = vec![
        (SKILL, None),
        (
            "workflows/flow/SKILL.md",
            Some(skill.replace("name: sample", "name: flow")),
        ),
        (TASK_MD, Some(task)),
        (CHECKS, Some(checks)),
        (KERNEL, Some(base(KERNEL).replace("sample", "flow"))),
    ];
    expect("workflow-in-workflows", &changes, "ok");
    let wrong: Changes = vec![(
        "workflows/flow/SKILL.md",
        Some(base(SKILL).replace("name: sample", "name: flow")),
    )];
    expect("verb-in-workflows", &wrong, "a verb belongs in kernel/, not workflows/");
}

#[test]
fn state_path_rules() {
    expect(
        "repo-relative-work",
        &sub(SKILL, "to $WORK/<slug>/", "to work/<slug>/"),
        "repo-relative work/ path",
    );
    expect(
        "work-at-line-start",
        &sub(SKILL, "Appends to $WORK/<slug>/", "work/<slug>/"),
        "repo-relative work/ path",
    );
    expect(
        "central-work",
        &sub(SKILL, "to $WORK/<slug>/", "to $SAM_STACK_WORK/repo/"),
        "ok",
    );
    expect(
        "work-inside-a-word",
        &sub(
            SKILL,
            "to $WORK/<slug>/",
            "to homework/ and net-work/ and a_work/ and .work/",
        ),
        "ok",
    );
}

#[test]
fn registry_rules() {
    expect(
        "no-registry-block",
        &sub(KERNEL, "```registry", "```"),
        "no ```registry block",
    );
    expect(
        "registry-writer-typo",
        &sub(KERNEL, "sample    N1", "smaple    N1"),
        "needs a writer that is a skill",
    );
    expect(
        "registry-row-without-writer",
        &sub(KERNEL, "notes.md    sample    N1, …", "notes.md"),
        "needs a writer that is a skill",
    );
    expect("registry-comment-row", &sub(KERNEL, "# file", "# any comment"), "ok");
    expect(
        "registry-blank-row",
        &sub(KERNEL, "# file      writer    ids\n", "\n"),
        "ok",
    );
    expect("no-kernel", &delete(KERNEL), "ok");
}

#[test]
fn shared_state_must_be_registered() {
    let output = "Output:    entry N<n> appended to $WORK/<slug>/notes.md";
    expect(
        "unregistered-shared-file",
        &sub(SKILL, output, "Output:    edits to $SAM_STACK_WORK/shared/list.md"),
        "writes $SAM_STACK_WORK/shared/list.md, which KERNEL.md's registry doesn't list",
    );
    let mut registered = sub(
        KERNEL,
        "notes.md    sample    N1, …\n",
        "notes.md    sample    N1, …\n$SAM_STACK_WORK/shared/list.md    sample    —\n",
    );
    registered.extend(sub(
        SKILL,
        output,
        "Output:    edits to $SAM_STACK_WORK/shared/list.md, then `$SAM_STACK_WORK/shared/list.md`.",
    ));
    expect("registered-shared-file", &registered, "ok");
    expect(
        "shared-path-outside-the-contract",
        &sub(
            SKILL,
            "Appends to $WORK/<slug>/notes.md.",
            "Reads $SAM_STACK_WORK/other/x.md.",
        ),
        "ok",
    );
}

#[test]
fn task_rules() {
    expect(
        "uncovered-skill",
        &sub(TASK_MD, "covers: sample", "covers: "),
        "no eval task covers",
    );
    expect("no-task-md", &delete(TASK_MD), "task.md needs frontmatter");
    expect(
        "covers-unknown-skill",
        &sub(TASK_MD, "covers: sample", "covers: sample, smaple"),
        "covers 'smaple'",
    );
    let mut installed = sub(TASK_MD, "covers: sample", "covers: sample, ground");
    installed.extend(add(
        CHECKS,
        &format!("{}check \"invoked ground\" invoked ground\n", base(CHECKS)),
    ));
    installed.extend(add(".claude/skills/ground/SKILL.md", ""));
    expect("covers-installed-skill", &installed, "ok");
    expect(
        "turns-not-a-number",
        &sub(TASK_MD, "turns: 1", "turns: many"),
        "turns: must be",
    );
    expect("turns-empty", &sub(TASK_MD, "turns: 1", "turns:"), "turns: must be");
    expect("turns-absent", &sub(TASK_MD, "turns: 1\n", ""), "ok");
    expect("no-prompt", &delete(PROMPT), "no prompt.md");
    expect("no-reference", &delete(APPLY), "no reference/apply.sh");
    expect(
        "no-reference-transcript",
        &delete(TRANSCRIPT),
        "no reference/transcript.jsonl",
    );
    let mut neither = delete(CHECKS);
    neither.extend(delete(APPLY));
    expect("no-checks-or-rubric", &neither, "needs checks.sh or rubric.md");
}

#[test]
fn invoked_rules() {
    expect(
        "covered-not-invoked",
        &sub(CHECKS, "check \"invoked sample\" invoked sample\n", ""),
        "no `invoked sample` check",
    );
    expect(
        "invoked-other-skill",
        &sub(CHECKS, "invoked sample\n", "invoked sampler\n"),
        "no `invoked sample` check",
    );
    expect(
        "invoked-suffix-only",
        &sub(CHECKS, "invoked sample\n", "notinvoked sample\n"),
        "no `invoked sample` check",
    );
    expect(
        "invoked-check-aligned",
        &sub(CHECKS, "\"invoked sample\" invoked", "\"invoked sample\"     invoked"),
        "ok",
    );
    expect(
        "invoked-trailing-space",
        &sub(CHECKS, "invoked sample\n", "invoked sample \t\n"),
        "ok",
    );
    expect(
        "invoked-at-line-start",
        &sub(CHECKS, "check \"invoked sample\" invoked sample\n", "invoked sample\n"),
        "ok",
    );
    let mut rubric_only = delete(CHECKS);
    rubric_only.extend(delete(APPLY));
    rubric_only.extend(add(RUBRIC, "1. Anything.\n"));
    expect("rubric-only", &rubric_only, "no `invoked sample` check");
}

#[test]
fn comment_rules() {
    expect(
        "rust-comment",
        &add("evals/src/main.rs", "fn main() {}\n// why\n"),
        "evals/src/main.rs: line 2 is a comment",
    );
    expect(
        "rust-clean",
        &add("evals/src/main.rs", "fn main() { let _ = \"//\"; }\n"),
        "ok",
    );
    expect(
        "shell-comment",
        &add("evals/lib.sh", "# helper\nfail() { :; }\n"),
        "evals/lib.sh: line 1 is a comment",
    );
    expect(
        "task-script-comment",
        &sub(
            CHECKS,
            "check \"sample written\"",
            "# section\ncheck \"sample written\"",
        ),
        "checks.sh: line 1 is a comment",
    );
    expect(
        "bin-fake-comment",
        &add("evals/tasks/sample-task/bin/herdr", "#!/bin/sh\n# fake\n"),
        "bin/herdr: line 2 is a comment",
    );
    expect(
        "shebang-only",
        &add("evals/tasks/sample-task/bin/herdr", "#!/bin/sh\necho '{}'\n"),
        "ok",
    );
    expect(
        "python-comment",
        &add("tools/x.py", "# why\n"),
        "tools/x.py: line 1 is a comment",
    );
    expect(
        "fixture-skipped",
        &add(
            "evals/tasks/sample-task/fixture/app.py",
            "# the scenario's own comment\n",
        ),
        "ok",
    );
    expect(
        "work-skipped",
        &add("evals/tasks/sample-task/work/s/x.sh", "# state\n"),
        "ok",
    );
    expect(
        "target-skipped",
        &add("target/debug/build/x.rs", "// generated\n"),
        "ok",
    );
    expect("markdown-ignored", &add("notes/x.md", "# heading\n"), "ok");
}

#[test]
fn errors_name_paths_relative_to_the_root() {
    let errors = lint_with(&sub(SKILL, "name: sample", "name: sampel"));
    assert_eq!(
        errors,
        ["kernel/sample/SKILL.md: frontmatter name must be 'sample', its directory"]
    );
}

#[test]
fn every_error_is_reported() {
    let mut changes = sub(SKILL, "name: sample", "name: sampel");
    changes.extend(delete(PROMPT));
    assert_eq!(lint_with(&changes).len(), 2);
}

#[test]
fn frontmatter_and_fields_parse_key_values() {
    let front = frontmatter("---\ncovers: a, b\nenv: K=V: x\nnot a field\n---\nbody: no\n").unwrap();
    assert_eq!(front.get("covers").map(String::as_str), Some("a, b"));
    assert_eq!(front.get("env").map(String::as_str), Some("K=V: x"));
    assert_eq!(front.len(), 2);
    assert!(frontmatter("covers: a\n").is_none());
    assert!(frontmatter("---\ncovers: a\n").is_none());
}
