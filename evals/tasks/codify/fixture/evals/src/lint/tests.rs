use super::*;

const SKILL: &str = "kernel/sample/SKILL.md";
const TASK_MD: &str = "evals/tasks/sample-task/task.md";
const PROMPT: &str = "evals/tasks/sample-task/prompt.md";
const CHECKS: &str = "evals/tasks/sample-task/checks.sh";
const APPLY: &str = "evals/tasks/sample-task/reference/apply.sh";
const RUBRIC: &str = "evals/tasks/sample-task/rubric.md";
const KERNEL: &str = "kernel/KERNEL.md";
const STOP: &str = "Stop:      the entry is written";

const REPO: [(&str, &str); 6] = [
    (KERNEL, "# Kernel\n\n```registry\n# file      writer    ids\nnotes.md    sample    N1, …\n```\n"),
    (
        SKILL,
        "---\nname: sample\ndescription: A minimal verb used only by the lint tests.\n---\n\n# Sample\n\n\
         Appends to $WORK/<slug>/notes.md.\n\n## Contract\n\n```contract\nLayer:     verb\nInput:     a question\n\
         Output:    entry N<n> appended to $WORK/<slug>/notes.md\nStop:      the entry is written\n\
         Returns:   the answer\n```\n",
    ),
    (TASK_MD, "---\ncovers: sample\nturns: 1\n---\n\n# sample-task\n"),
    (PROMPT, "Do the sample thing.\n"),
    (CHECKS, "check \"sample written\" test -f sample.md\n"),
    (APPLY, "touch sample.md\n"),
];

type Changes = Vec<(&'static str, Option<String>)>;

fn base(path: &str) -> &'static str {
    REPO.iter().find(|(p, _)| *p == path).map(|(_, t)| *t).unwrap()
}

fn sub(path: &'static str, old: &str, new: &str) -> Changes {
    assert!(base(path).contains(old), "test setup: {old:?} not in {path}");
    vec![(path, Some(base(path).replacen(old, new, 1)))]
}

fn set(changes: &[(&'static str, Option<&str>)]) -> Changes {
    changes.iter().map(|(p, t)| (*p, t.map(String::from))).collect()
}

fn cases() -> Vec<(&'static str, Changes, &'static str)> {
    vec![
        ("valid", vec![], "ok"),
        ("no-frontmatter", sub(SKILL, "---\nname: sample", "name: sample"), "name must be 'sample'"),
        ("name-mismatch", sub(SKILL, "name: sample", "name: sampel"), "name must be 'sample'"),
        ("no-description", sub(SKILL, "description: A minimal verb used only by the lint tests.", "description:"), "no description"),
        ("no-contract-block", sub(SKILL, "```contract", "```"), "no ```contract block"),
        ("missing-field", sub(SKILL, &format!("{STOP}\n"), ""), "'Stop' is missing or empty"),
        ("blank-field", sub(SKILL, STOP, "Stop:"), "'Stop' is missing or empty"),
        ("tab-aligned-field", sub(SKILL, STOP, "Stop:\tthe entry is written"), "ok"),
        ("unknown-layer", sub(SKILL, "Layer:     verb", "Layer:     helper"), "must be verb, combinator or workflow"),
        ("layer-in-wrong-folder", sub(SKILL, "Layer:     verb", "Layer:     workflow"), "belongs in workflows/"),
        ("combinator-in-kernel", sub(SKILL, "Layer:     verb", "Layer:     combinator"), "ok"),
        ("repo-relative-work", sub(SKILL, "to $WORK/<slug>/", "to work/<slug>/"), "repo-relative work/ path"),
        ("central-work", sub(SKILL, "to $WORK/<slug>/", "to $SAM_STACK_WORK/repo/"), "ok"),
        ("no-registry-block", sub(KERNEL, "```registry", "```"), "no ```registry block"),
        ("registry-writer-typo", sub(KERNEL, "sample    N1", "smaple    N1"), "needs a writer that is a skill"),
        ("registry-comment-row", sub(KERNEL, "# file", "# any comment"), "ok"),
        ("uncovered-skill", sub(TASK_MD, "covers: sample", "covers: "), "no eval task covers"),
        ("covers-unknown-skill", sub(TASK_MD, "covers: sample", "covers: sample, smaple"), "covers 'smaple'"),
        (
            "covers-installed-skill",
            [sub(TASK_MD, "covers: sample", "covers: sample, ground"), set(&[(".claude/skills/ground/SKILL.md", Some(""))])].concat(),
            "ok",
        ),
        ("turns-not-a-number", sub(TASK_MD, "turns: 1", "turns: many"), "turns: must be"),
        ("no-prompt", set(&[(PROMPT, None)]), "no prompt.md"),
        ("no-reference", set(&[(APPLY, None)]), "no reference/apply.sh"),
        ("rubric-only", set(&[(CHECKS, None), (APPLY, None), (RUBRIC, Some("1. Anything.\n"))]), "ok"),
    ]
}

fn lint_with(changes: &Changes) -> Vec<String> {
    let dir = std::env::temp_dir().join(format!("lint-test-{}-{}", std::process::id(), changes.len()));
    let mut files: BTreeMap<&str, Option<String>> = REPO.iter().map(|(p, t)| (*p, Some((*t).to_string()))).collect();
    files.extend(changes.iter().cloned());
    let _ = fs::remove_dir_all(&dir);
    for (path, text) in files {
        if let Some(text) = text {
            let file = dir.join(path);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, text).unwrap();
        }
    }
    let errors = lint(&dir);
    let _ = fs::remove_dir_all(&dir);
    errors
}

#[test]
fn every_case() {
    let mut failed = Vec::new();
    for (name, changes, expect) in cases() {
        let errors = lint_with(&changes);
        let ok = if expect == "ok" { errors.is_empty() } else { errors.iter().any(|e| e.contains(expect)) };
        if !ok {
            failed.push(format!("{name} (expect {expect}): {errors:?}"));
        }
    }
    assert!(failed.is_empty(), "{failed:#?}");
}
