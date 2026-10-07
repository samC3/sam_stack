# Decisions

## D1: Where the goals come from
Chose: ground them from the explicit input, then work/<slug>/ (spec.md, decisions.md, tickets/), then the PR body and linked issues. If none are found, stop and ask. Never derive goals from the diff. Because: accepted recommendation.
Rejected: infer goals from the diff (that only checks the code against itself); always ask (makes the user repeat what's already written).
Informed by: F2

## D2: Output
Chose: work/<slug>/verdicts.md (judge) + work/<slug>/review.md (distill, reader = the PR author). Post to GitHub only if the prompt asks. Because: accepted recommendation.
Rejected: always post to the PR (can't be cleanly taken back); conversation only (doesn't last beyond the session).

## D3: Evidence bar for "met"
Chose: re-prove. Every "met" needs a check the reviewer ran itself. The author's P entries, PR body and "tested" claims are leads, not evidence. A goal no one could check is "unproven". Because: accepted recommendation.
Rejected: citing the author's proofs (trusts self-report, against kernel principle 1); reading only.

## D4: Scope
Chose: goals first, with a verdict per goal (met / not met / unproven), then a find-defects pass on the change. Because: accepted recommendation.
Rejected: goals only.

## D5: Triggers
Chose: "adversarial review", "review PR #N", "review this branch", "does this meet its goals/spec", "check this PR does what it says". judge keeps "which is better", "rank these", and reviews of a file or snippet with no goals behind it. The negative case is "compare these two approaches". Because: accepted recommendation.
Rejected: explicit name only.
Informed by: F4

## D6: Name and layer
Chose: adversarial-review, workflow. Because: user named it; F3.
