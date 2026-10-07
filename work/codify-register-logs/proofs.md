# Proofs: codify-register-logs

## P1: lint rejects a skill whose contract writes an unregistered work/ file, and only that
Falsified by: `unregistered-work-file` passing lint, or either registered near-miss failing
Check: `evals/lint-test.sh` (strength: real surface, the lint script itself)
Could fail: with the lint from HEAD (bacb797), `unregistered-work-file` FAILs (lint says ok). On this repo the new rule reports `lessons.md`, `work/lessons/candidates.md` and `tickets/NN-<name>.md`
Observed:
    unregistered-work-file  PASS · registered-work-file  PASS · registered-shared-file  PASS · lint-test: ok
Verdict: holds

## P2: with the proposed KERNEL.md registration, lint passes on this repo
Falsified by: `evals/lint.sh` reporting any error after the L1 diff is applied
Check: lint on a scratch copy of the repo with the proposed KERNEL.md (strength: real surface, on a copy)
Could fail: the same lint fails on the real repo with 3 errors (P1)
Observed:
    lint: ok   (on the copy; the real repo waits for sign-off)
Verdict: holds on the copy. Rerun on the repo after sign-off

## P3: after sign-off, lint passes on this repo (supersedes P2's pending rerun)
Check: `evals/lint.sh` and `evals/lint-test.sh` on the repo, with the L1 diff applied
Could fail: the same lint failed here with 3 errors before the edit (P1)
Observed:
    lint: ok
    lint-test: ok
Verdict: holds
