# Proofs

## P1: The skill and both cases pass the structural lint
Falsified by: evals/lint.sh reporting any error
Check: `evals/lint.sh` on the repo (strength: real surface, since lint is the gate)
Could fail: the first run failed with "Output writes work/<slug>/review.md, which KERNEL.md's State section doesn't register" and "no eval case in evals/cases/adversarial-review-*/"
Observed:
    lint: ok
Verdict: holds

## P2: Given a branch whose PR claims all goals and whose tests pass, the skill finds the unmet goal by re-proving
Falsified by: check.sh or rubric failing on adversarial-review-reprove-goals
Check: `evals/run.sh adversarial-review-reprove-goals` (strength: real surface, a headless session)
Could fail: the rubric fails a run that cites P1 or marks criterion 3 met; check.sh fails if the code under review is edited
Observed:
    adversarial-review-reprove-goals r1   PASS   (cost $0.41; runs in ~/.cache/sam_stack-evals/20261008-085510-6096)
    grade: all 5 criteria PASS; review.md says "Verdict: changes needed", goal 3 not met (P4), and the author's P1 "could not fail on criterion 3"
Verdict: holds (n=1)

## P3: "Compare these two approaches" routes to judge and not to adversarial-review
Falsified by: adversarial-review invoked, or judge not invoked
Check: `evals/run.sh adversarial-review-not-for-compare` (strength: real surface)
Could fail: it did
Observed:
    adversarial-review-not-for-compare r1   FAIL FAIL: skill 'judge' was not invoked   (cost $0.15)
    no Skill tool call at all; the agent compared the files inline (ran both, timed them) and recommended approach_b
Verdict: fails. The half this skill is responsible for (it didn't fire) holds; the judge-routing half fails. The case checks two claims, and the failing one is about judge's description, not this skill's.
