# A correct make-skill outcome, used by `evals/run.sh --graders`.
mkdir -p workflows/investigate evals/cases/investigate-no-edit "$WORK/make-skill-investigate"
cat > workflows/investigate/SKILL.md <<'MD'
---
name: investigate
description: Answer a question about the codebase without changing it, by grounding it and summarising for the user. Use for "investigate", "dig into", "look into why". Not for requests to change code.
---

# Investigate

## Contract

```
Layer:     workflow
Input:     the question; slug
Output:    $WORK/<slug>/findings.md and $WORK/<slug>/summary.md
Stop:      the summary answers the question or says what would settle it
Returns:   the summary
```

## Steps

1. `ground` the question.
2. `distill` a `summary` for the user.
MD
echo 'Look into why the discount barely changes big orders.' > evals/cases/investigate-no-edit/prompt.md
echo 'invoked investigate' > evals/cases/investigate-no-edit/check.sh
cat > "$WORK/make-skill-investigate/proofs.md" <<'MD'
## P1: lint passes
Observed:
    lint: ok
## P2: the behaviour eval
Verdict: not run (the user asked not to spend tokens)
MD
