#!/usr/bin/env python3
"""Tests for evals/lint.py. Free and deterministic.

Each case changes a minimal valid repo and says what lint must report: "ok", or
a phrase from the error. Each rule has a failing case and a near-miss that must
still pass, so a rule can't be "fixed" by rejecting too much.
"""
import sys
import tempfile
from pathlib import Path

from lint import lint

SKILL, TASK, KERNEL = "kernel/sample/SKILL.md", "evals/tasks/sample-task", "kernel/KERNEL.md"
STOP = "Stop:      the entry is written"

REPO = {
    KERNEL: """# Kernel

```registry
# file      writer    ids
notes.md    sample    N1, …
```
""",
    SKILL: """---
name: sample
description: A minimal verb used only by the lint tests.
---

# Sample

Appends to $WORK/<slug>/notes.md.

## Contract

```contract
Layer:     verb
Input:     a question
Output:    entry N<n> appended to $WORK/<slug>/notes.md
Stop:      the entry is written
Returns:   the answer
```
""",
    f"{TASK}/task.md": "---\ncovers: sample\nturns: 1\n---\n\n# sample-task\n",
    f"{TASK}/prompt.md": "Do the sample thing.\n",
    f"{TASK}/checks.sh": 'check "sample written" test -f sample.md\ncheck "invoked sample" invoked sample\n',
    f"{TASK}/reference/apply.sh": "touch sample.md\n",
    f"{TASK}/reference/transcript.jsonl": '{"type": "assistant", "message": {"content": [{"type": "tool_use", "name": "Skill", "input": {"skill": "sample"}}]}}\n',
}


def sub(path, old, new):
    """REPO[path] with old replaced by new."""
    assert old in REPO[path], f"test setup: {old!r} not in {path}"
    return {path: REPO[path].replace(old, new)}


CASES = [
    # name                     changes (path: new text, or None to delete)                      expect
    ("valid",                  {},                                                              "ok"),
    # frontmatter
    ("no-frontmatter",         sub(SKILL, "---\nname: sample", "name: sample"),                 "name must be 'sample'"),
    ("name-mismatch",          sub(SKILL, "name: sample", "name: sampel"),                      "name must be 'sample'"),
    ("no-description",         sub(SKILL, "description: A minimal verb used only by the lint tests.", "description:"),"no description"),
    # contract
    ("no-contract-block",      sub(SKILL, "```contract", "```"),                                "no ```contract block"),
    ("missing-field",          sub(SKILL, STOP + "\n", ""),                                     "'Stop' is missing or empty"),
    ("blank-field",            sub(SKILL, STOP, "Stop:"),                                       "'Stop' is missing or empty"),
    ("tab-only-field",         sub(SKILL, STOP, "Stop:\t"),                                     "'Stop' is missing or empty"),
    ("tab-aligned-field",      sub(SKILL, STOP, "Stop:\tthe entry is written"),                 "ok"),
    ("unknown-layer",          sub(SKILL, "Layer:     verb", "Layer:     helper"),              "must be verb, combinator or workflow"),
    ("layer-in-wrong-folder",  sub(SKILL, "Layer:     verb", "Layer:     workflow"),            "belongs in workflows/"),
    ("combinator-in-kernel",   sub(SKILL, "Layer:     verb", "Layer:     combinator"),          "ok"),
    # state paths
    ("repo-relative-work",     sub(SKILL, "to $WORK/<slug>/", "to work/<slug>/"),               "repo-relative work/ path"),
    ("central-work",           sub(SKILL, "to $WORK/<slug>/", "to $SAM_STACK_WORK/repo/"),      "ok"),
    # the registry
    ("no-registry-block",      sub(KERNEL, "```registry", "```"),                               "no ```registry block"),
    ("registry-writer-typo",   sub(KERNEL, "sample    N1", "smaple    N1"),                     "needs a writer that is a skill"),
    ("registry-comment-row",   sub(KERNEL, "# file", "# any comment"),                          "ok"),
    # tasks and coverage
    ("uncovered-skill",        sub(f"{TASK}/task.md", "covers: sample", "covers: "),            "no eval task covers"),
    ("covers-unknown-skill",   sub(f"{TASK}/task.md", "covers: sample", "covers: sample, smaple"), "covers 'smaple'"),
    ("covers-installed-skill", {**sub(f"{TASK}/task.md", "covers: sample", "covers: sample, ground"),
                                f"{TASK}/checks.sh": REPO[f"{TASK}/checks.sh"] + "check \"invoked ground\" invoked ground\n",
                                ".claude/skills/ground/SKILL.md": ""},                           "ok"),
    ("turns-not-a-number",     sub(f"{TASK}/task.md", "turns: 1", "turns: many"),               "turns: must be"),
    ("no-prompt",              {f"{TASK}/prompt.md": None},                                     "no prompt.md"),
    ("no-reference",           {f"{TASK}/reference/apply.sh": None},                            "no reference/apply.sh"),
    ("no-reference-transcript", {f"{TASK}/reference/transcript.jsonl": None},                      "no reference/transcript.jsonl"),
    # every covered skill is checked as invoked
    ("covered-not-invoked",    sub(f"{TASK}/checks.sh", 'check "invoked sample" invoked sample\n', ""), "no `invoked sample` check"),
    ("invoked-other-skill",    sub(f"{TASK}/checks.sh", "invoked sample\n", "invoked sampler\n"),  "no `invoked sample` check"),
    ("invoked-check-aligned",  sub(f"{TASK}/checks.sh", '"invoked sample" invoked', '"invoked sample"     invoked'), "ok"),
    ("rubric-only",            {f"{TASK}/checks.sh": None, f"{TASK}/reference/apply.sh": None,
                                f"{TASK}/rubric.md": "1. Anything.\n"},                          "no `invoked sample` check"),
]


def lint_with(changes):
    with tempfile.TemporaryDirectory() as tmp:
        for path, text in {**REPO, **changes}.items():
            if text is not None:
                (Path(tmp) / path).parent.mkdir(parents=True, exist_ok=True)
                (Path(tmp) / path).write_text(text)
        return lint(tmp)


failed = 0
for name, changes, expect in CASES:
    errors = lint_with(changes)
    ok = not errors if expect == "ok" else any(expect in e for e in errors)
    print(f"{'PASS' if ok else 'FAIL'}  {name:24} (expect {expect})")
    if not ok:
        failed += 1
        print("".join(f"        {e}\n" for e in errors or ["(lint reported nothing)"]), end="")
print(f"lint_test: {failed} failed" if failed else "lint_test: ok")
sys.exit(1 if failed else 0)
