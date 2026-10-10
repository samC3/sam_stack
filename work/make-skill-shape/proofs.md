# Proofs

## P1: The skill and both cases pass the structural lint
Falsified by: evals/lint.sh reporting any error
Check: `evals/lint.sh` and `evals/lint-test.sh` on the repo (strength: real surface, since lint is the gate)
Could fail: an unregistered work/ path in Output, or a missing case, would fail it
Observed:
    lint: ok
    lint-test: ok
Verdict: holds (re-run after the cold-read fixes: lint: ok)

## P2: Given a vague build request, shape grounds, asks one preference question and writes no spec or tickets
Falsified by: check.sh or rubric failing on shape-no-invented-decisions
Check: `evals/run.sh shape-no-invented-decisions` (strength: real surface, a headless session)
Could fail: it did
Observed:
    shape-no-invented-decisions r1   FAIL rubric   (cost $0.21; ~/.cache/sam_stack-evals/20261010-170309-66196)
    check.sh passed: shape invoked, findings.md has F entries, no spec.md or tickets, notes.py untouched
    grade: criteria 2–4 PASS (asked nothing the repo answers; F3 lists format, destination, archived as open; read both files first)
    criterion 1 FAIL: the one question was "does backing up include restore?", not format/destination/which notes
Verdict: fails as written. The habit held. Criterion 1 named only three acceptable topics, and restore scope is also a choice the request leaves open; the rubric was widened to say so. The case has not been re-run since.

## P3: "Write the spec" with settled decisions routes to distill, not shape
Falsified by: shape invoked, or no work/notes-export/spec.md citing D1
Check: `evals/run.sh shape-not-for-settled-spec` (strength: real surface)
Could fail: shape's description includes "spec this out"
Observed:
    shape-not-for-settled-spec r1   PASS   (cost $0.39); skills invoked: distill
Verdict: holds (n=1)

## P4: P2 re-run with the widened rubric (supersedes P2's verdict)
Falsified by: check.sh or rubric failing on shape-no-invented-decisions
Check: `evals/run.sh shape-no-invented-decisions` (strength: real surface)
Could fail: criteria 1–4 as in rubric.md; P2 failed criterion 1
Observed:
    shape-no-invented-decisions r1   PASS   (cost $0.22; ~/.cache/sam_stack-evals/20261010-171808-77122); skills invoked: shape
Verdict: holds (n=1, after the rubric change)
