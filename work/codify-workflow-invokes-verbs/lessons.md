# Lessons

## L1: A workflow step that names a verb is done inline, so only the verb's one-line description guides it
Evidence: eval runs under ~/.cache/sam_stack-evals/: 20261010-170309-66196 and 20261010-171808-77122 (shape-no-invented-decisions: Skill calls `shape` only); 20261008-085510-6096 (adversarial-review-reprove-goals: `adversarial-review` only); 20261006-213551-84908 (make-skill-honest-report: `make-skill` only); 20261006-213914-87515 (make-skill-honest-report: `make-skill ground prove`, no `distill`). Bar met by: five runs across three workflows.
Level: case check.sh (detects it) + KERNEL.md Layers (prevents it; the rule applies to every workflow, so no single skill owns it)
Passed over: lint (the failure only shows in a run's transcript, not in skill files); runner flags (no flag forces a Skill call); rubric (a script can see it); per-workflow SKILL.md text (it would have to be repeated in every workflow).
Regression check: `invoked <verb>` lines added to evals/cases/shape-no-invented-decisions/check.sh (ground, ask), adversarial-review-reprove-goals/check.sh (ground, judge, prove, distill), make-skill-honest-report/check.sh (ground, distill, prove; ask is left out because the prompt settles every decision). Red: all five incident runs above are missing at least one required verb (`jq` over each transcript, the same selector `invoked` uses). Green: P1.
Removed: none
Status: applied (user signed off on the diff below)

```diff
 - Verbs never invoke workflows.
+- A workflow step that names a verb or combinator invokes that skill, so its procedure is in context. Doing the step inline from the verb's description doesn't count.
 - Verbs and combinators live in `kernel/<name>/`, workflows in `workflows/<name>/`. Every skill has at least one eval case in `evals/cases/<name>-<habit>/`.
```
