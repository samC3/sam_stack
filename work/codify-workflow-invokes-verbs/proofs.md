# Proofs

## P1: With the kernel rule, each workflow eval invokes every verb its check requires
Falsified by: any `invoked <verb>` line in the three updated check.sh files failing
Check: `evals/run.sh --jobs 3 shape-no-invented-decisions adversarial-review-reprove-goals make-skill-honest-report` (strength: real surface, headless sessions)
Could fail: all five earlier runs of these cases fail the same checks (L1 Red)
Observed:
    shape-no-invented-decisions      r1 PASS ($0.22)  skills: shape ground ask
    adversarial-review-reprove-goals r1 PASS ($0.55)  skills: adversarial-review ground judge prove distill
    make-skill-honest-report         r1 PASS ($0.88)  skills: make-skill ground distill prove
    runs: ~/.cache/sam_stack-evals/20261010-174414-95028
Verdict: holds (n=1 per case)
