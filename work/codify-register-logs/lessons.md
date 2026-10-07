# Lessons: codify-register-logs

## L1: A skill's contract writes files under work/ that the kernel's State section doesn't register
Evidence: O1 and D7 in work/make-skill-codify/decisions.md and F8 in findings.md, all at 2567638; kernel/codify/SKILL.md Output; kernel/distill/SKILL.md Output (a second instance of the same class: `tickets/NN-<name>.md`). Bar met by: reproduced by `evals/lint.sh` with the new rule (P1).
Level: `evals/lint.sh` rule (applied). The registration itself is a KERNEL.md edit (awaiting sign-off).
Passed over: none. Lint was the strongest level and fits, because both sides of the mismatch are visible in files.
Regression check: `evals/lint-test.sh`, fixtures `unregistered-work-file` (must fail) and the near-misses `registered-work-file` and `registered-shared-file` (must pass). Red: P1. Green: waits for sign-off (P2).
Removed (proposed, sign-off): in kernel/codify/SKILL.md, "long-lived and never deleted with a task" after `work/lessons/candidates.md`, because the kernel will state it.
Status: lint rule applied; KERNEL.md registration and the codify removal awaiting sign-off (proposed diff below). Until sign-off, `evals/lint.sh` fails on this repo with exactly the three unregistered paths.

```diff
@@ -39,8 +39,8 @@
 - Work lives in `work/<slug>/` at the repo root and is **committed** with the branch it belongs to. That way a worktree or another session on the branch already has it, and reviewers see the reasoning beside the change. With no repo, use `~/.claude/work/<slug>/`.
 - The slug is kebab-case from the task. Reuse an existing directory when the task matches it.
 - One slug belongs to one branch. If two branches must share a slug, renumber the incoming entries when merging so ids stay unique.
-- The change that finishes the task deletes `work/<slug>/`; git history keeps the record.
-- Each verb appends to its own log, one entry per heading, with a stable id:
+- The change that finishes the task deletes `work/<slug>/`; git history keeps the record. The exception is `work/lessons/`, which is shared across tasks and never deleted. `lessons` is never used as a task slug.
+- Each verb appends to its own log, one entry per heading, with a stable id. This table is the registry: every file a skill's contract writes under `work/` is listed here, and `evals/lint.sh` rejects any that isn't.
 
 | File | Written by | Id |
 |---|---|---|
@@ -48,7 +48,9 @@
 | `decisions.md` | ask | `D1`, … (open items `O1`, …) |
 | `proofs.md` | prove | `P1`, … |
 | `verdicts.md` | judge | `J1`, … |
-| `<format>.md` (`spec.md`, `handoff.md`, …) | distill | — |
+| `<format>.md` (`spec.md`, `handoff.md`, …), `tickets/NN-<name>.md` | distill | — |
+| `lessons.md` | codify | `L1`, … |
+| `work/lessons/candidates.md` | codify | — (shared across tasks, never deleted) |
 
 - Logs are append-only. To revise an entry, add a new one that says `supersedes F3`.
 - Refer to entries by id instead of restating them.
--- a/kernel/codify/SKILL.md
+++ b/kernel/codify/SKILL.md
-Candidates (`work/lessons/candidates.md`, long-lived and never deleted with a task):
+Candidates (`work/lessons/candidates.md`):
```

## Sign-off for L1
Signed off by the user on 2026-10-07. The KERNEL.md diff and the codify removal are applied. Green: P3. Status: applied.
