---
name: judge
description: Assess an artifact, or choose among options, against explicit criteria when no executable check can decide, producing a ranked verdict with evidence. Use for code review, comparing approaches or prototypes, ranking findings, evaluating a skill or prompt change, or merging results from parallel attempts; for "review this", "which is better", "tear this apart", "rank these".
---

# Judge

Make a call that can't be settled by running something, and make it defensible.

## Contract

```contract
Layer:     verb
Input:     subject(s); stance (find-defects | choose | rank); criteria (given, or derived in step 1); slug
Output:    entry J<n> appended to $WORK/<slug>/verdicts.md
Stop:      every criterion has a verdict for every subject, and a ranking or choice is stated
Returns:   the choice or top findings in 1–3 lines, plus the J id
```

## Procedure

1. **Fix the criteria before you look at the subjects.** Take them from the spec and decisions, the kernel principles, or the caller. Each criterion must be observable in the subject. If one isn't, rewrite it or drop it.
2. **Blind the subjects** when there are several. Label them A, B, C… and ignore who or what produced them.
3. **Run what can be run.** If a criterion can be settled by running something, hand it to `prove` and cite the `P` id rather than judging it.
4. **Give a verdict per criterion per subject,** each with evidence (`path:line`, a quote, a `P`/`F` id). No evidence means no verdict.
5. **Stance-specific step:**
   - *find-defects*: try to break it. Each finding needs a concrete failure scenario (this input or state produces this wrong result). Rank by consequence. Drop style nits unless the caller asked for them.
   - *choose*: pick one. Name what you give up by not choosing the runner-up.
   - *rank*: give a total order and say where the gaps between items are large.
6. **Attack your own findings.** Try to refute each one. Mark it `confirmed` (the refutation failed) or `plausible` (couldn't settle). Drop anything you refuted.
7. **Record the entry.**

## Entry format

```markdown
## J1: <subject> (<stance>)
Criteria: <list, fixed before looking>
| Criterion | A | B |
|---|---|---|
| <criterion> | ✓ src/x.ts:10 | ✗ fails on empty input (P4) |
Result: <choice, ranked findings, or order>
Findings (find-defects):
1. [confirmed] <defect>. Scenario: <input/state → wrong result>. Evidence: <ref>
Trade-off accepted: <what the choice gives up>
```

## Rules

- Judge doesn't fix anything. It hands findings back.
- Don't soften a verdict to be agreeable, and don't invent findings to look thorough. Zero findings is a valid result.
- If the criteria really are tied, say so and name the one fact or decision that would break the tie.
