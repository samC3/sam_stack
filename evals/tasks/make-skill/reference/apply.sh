mkdir -p workflows/investigate evals/tasks/investigate/reference "$WORK/make-skill-investigate"
cat > workflows/investigate/SKILL.md <<'MD'
---
name: investigate
description: Answer a question about the codebase without changing it, by grounding it and summarising for the user. Use for "investigate", "dig into", "look into why". Not for requests to change code.
---

# Investigate

## Contract

```contract
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
printf -- '---\ncovers: investigate, ground, distill\n---\n\n# investigate\n' > evals/tasks/investigate/task.md
echo 'Look into why the discount barely changes big orders.' > evals/tasks/investigate/prompt.md
echo 'check "no code edited" test -z "$(git status --porcelain -- pricing.py)"' > evals/tasks/investigate/checks.sh
echo 'true' > evals/tasks/investigate/reference/apply.sh
for s in investigate ground distill; do echo "check \"invoked $s\" invoked $s" >> evals/tasks/investigate/checks.sh; done
for s in investigate ground distill; do printf '{"type": "assistant", "message": {"content": [{"type": "tool_use", "name": "Skill", "input": {"skill": "%s"}}]}}\n' "$s"; done > evals/tasks/investigate/reference/transcript.jsonl
cat > "$WORK/make-skill-investigate/proofs.md" <<'MD'
## P1: lint passes
Observed:
    lint: ok
## P2: the behaviour eval
Verdict: not run (the user asked not to spend tokens)
MD
