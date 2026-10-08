---
name: make-skill
description: Create a new skill, or regenerate an existing one, by wiring the kernel verbs, and ship it with an eval case that can fail. Use for "make a skill for X", "turn this into a skill", "write a skill that…", "regenerate the ground skill", "rebuild <skill>".
---

# Make skill

The workflow that builds skills, including itself. It is wiring only: how to ask, write, check or compare lives in the verbs it calls.

## Contract

```
Layer:     workflow
Input:     the need (what keeps happening, or what should be automated), or the path of an existing skill to regenerate; slug (default make-skill-<name>)
Output:    kernel/<name>/SKILL.md or workflows/<name>/SKILL.md; at least one case in evals/cases/<name>-<habit>/; logs in $WORK/<slug>/
Stop:      evals/lint.sh passes and the behaviour eval has a recorded verdict (or is recorded as not run, with the reason). When regenerating, judge has chosen between the old and new versions.
Returns:   the paths written, the eval verdicts with their cost, and any open questions
```

## Steps

1. **`ground`: should this be a skill at all?**
   - What recurring behaviour is it for? Look for evidence in transcripts, the request and existing skills.
   - Does an existing skill, or a verb with different parameters, already cover it? If so, stop and report that. The answer is a parameter or a composition, not a new skill.
   - Which layer is it? A new verb must meet the kernel's admission rule, with evidence.
   - When regenerating: read the current `SKILL.md` and its eval cases, and record its contract as the baseline.

2. **`ask`: settle the design.** Cover anything `ground` didn't answer:
   - the name
   - phrases that should trigger it, and nearby phrases that must not
   - inputs and their defaults
   - the output artifact
   - the stop condition
   - the one habit the skill exists for, and how that habit typically fails

   When regenerating, ask only about the changes the user intends.

3. **`distill` the skill.** Reader: an agent that has `KERNEL.md` and nothing else. Format: `skill`.
   - For a workflow, the body is a list of steps, each naming a verb or combinator and its parameters.
   - If text explaining *how* to do something appears, it belongs in a verb. Record it as an open question for `codify` instead of keeping it in the workflow.

4. **`distill` the eval case.** Reader: `evals/run.sh`. Format: `eval-case`, targeting the habit and its typical failure from step 2. If the trigger phrases overlap another skill's, also write a case where the skill must not fire.

5. **`prove` the structure:** run `evals/lint.sh`. Fix the skill or the case until it passes.

6. **`prove` the behaviour:** run `evals/run.sh <case>` once. Record the cost alongside the verdict.
   - Each run costs real tokens. Before running anything beyond one run per new case, say what you want to run and its estimated cost, and wait for the user.
   - If the user has said not to run evals, record the behaviour check as not run. Don't claim the skill works.

7. **When regenerating, `judge` old against new.** Stance: choose.
   - Put the old version on a baseline checkout: `git worktree add <tmp> HEAD`.
   - Run the cases against both versions: `evals/run.sh --root <tmp> <case>` for the old one, and the plain command for the new one. Confirm this extra cost with the user first, as in step 6.
   - Judge the two versions blind.
   - Criteria: the eval results; whether each contract field is concrete; whether the habit is stated; and length (shorter wins a tie).
   - If the old version wins, restore it with `git checkout -- <path>` and report why.

8. **`distill` the report.** Reader: the user. Format: `summary`. Include the paths written, verdicts with ids, cost, and any open questions.

## Rules

- Create one skill per run.
- Never edit `KERNEL.md`. That's `codify`'s job. Until `codify` exists, propose kernel changes to the user as open questions.
- Don't install or link skills into `~/.claude`. Installing is the user's call.
