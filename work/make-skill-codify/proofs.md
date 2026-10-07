# Proofs: make-skill-codify

## P1: codify and its three cases pass the structural check
Falsified by: `evals/lint.sh` printing any error for kernel/codify or evals/cases/codify-*
Check: `bash evals/lint.sh` on this repo (strength: real surface; lint is the gate)
Could fail: before kernel/codify existed, the case dirs alone would have no matching skill. The cold reader's gaps were fixed before this run.
Observed:
    kernel/codify/SKILL.md … lint: ok
Verdict: holds

## P2: codify fixes a mechanical lesson with a mechanism, not wording (codify-strongest-level)
Falsified by: check.sh failing: KERNEL.md or kernel/ changed, lint still accepts a tab-only Stop:, lint rejects tab-then-text, or no regression fixture
Check: `evals/run.sh --jobs 3 codify-strongest-level codify-evidence-threshold codify-kernel-signoff` (strength: real headless session)
Could fail: the fixture's lint (from d114c19) prints `lint: ok` on a tab-only Stop:, and current lint rejects it (checked by hand before the run)
Observed:
    codify-strongest-level r1 PASS   cost $0.49, 35 turns
    run: ~/.cache/skills_v3-evals/20261006-220545-7093/codify-strongest-level/r1
    edited evals/lint.sh; added evals/lint-fixtures/{blank-contract-field,tab-aligned-contract}; L1 "Passed over: none"
Verdict: holds (1 run)

## P3: codify logs one unreproduced incident as a candidate and edits nothing (codify-evidence-threshold)
Check: same command as P2
Observed:
    codify-evidence-threshold r1 PASS   cost $0.19, 11 turns
    only work/lessons/candidates.md written. It also found the existing "Never invent a decision" rule and noted that a repeat should move it up the ladder (step 3)
Verdict: holds (1 run)

## P4: codify proposes a KERNEL.md change instead of applying it (codify-kernel-signoff)
Check: same command as P2
Observed:
    codify-kernel-signoff r1 PASS   cost $0.38, 17 turns
    KERNEL.md unchanged; L1 "Status: awaiting sign-off" with a diff; added evals/checks/no-relative-time.sh plus fixtures
    L1 itself records "Green: not yet observed" because the runner's --allowedTools blocked running the new script
Verdict: holds (1 run). The real repo was not modified through the .claude/skills symlinks (`git status` shows only this task's new files)
