---
name: codify
description: Turn a recurring lesson into structure, enforced at the strongest level available (lint, eval check, skill text, kernel text, then note), with a regression check and a record of why. The only verb that may edit KERNEL.md. Use when something has gone wrong or right more than once; for "codify this", "make sure this never happens again", "turn this lesson into a rule", "this keeps happening", "add a lint rule for". Not for "make a skill for X", "remember this", or a plain bug fix with no lesson.
---

# Codify

Make a lesson stop depending on anyone remembering it. A rule nobody checks is a wish.

## Contract

```
Layer:     verb
Input:     lesson (one line: what happened, and what should happen instead); evidence (F/P/J ids, transcript or eval-run paths, commit shas); slug (default codify-<class>)
Output:    the edit at the chosen level, plus entry L<n> appended to work/<slug>/lessons.md; or, below the evidence bar, an entry appended to work/lessons/candidates.md
Stop:      one of: an L entry is recorded whose regression check was seen failing on the incident and passing after the edit; the lesson is logged as a candidate; or a proposed edit is recorded as awaiting sign-off
Returns:   the L id (or "candidate"), the level chosen, files changed, and anything awaiting the user's sign-off
```

## Procedure

1. **Check the evidence bar.** Read `work/lessons/candidates.md` first: a matching candidate plus this incident makes two. Act only on two or more independent incidents (different runs or sessions), or on one incident that a script or command you can run reproduces the same way every time. Below the bar, append a candidate entry (create the file if needed) and stop. Change nothing else.
2. **Name the class.** State the failure so a reader who never saw the incident would recognise it. Leave out the incident's names, numbers and phrasing.
   - If the lesson comes from a failing check, decide whether the behaviour or the check is wrong: `judge` with stance `choose`. Loosening a check is a weakening (step 6).
   - If the fix depends on intent (who owns a trigger phrase, which trade-off wins), hand that part to `ask`.
3. **Subtract first.** Search the skills, `KERNEL.md`, `evals/lint.sh` and the eval cases for a rule that already covers the class.
   - If one exists, it wasn't enforced strongly enough. Move it up the ladder; don't restate it.
   - If your fix makes existing text redundant, propose deleting that text.
4. **Walk the ladder, strongest first.** This is principle 6 for this repo: lint, then test (check, flags, rubric), then skill text. Take the first level that can block or deterministically reject the failure:

   | Level | Use when |
   |---|---|
   | `evals/lint.sh` rule | the failure is visible in skill or case files |
   | case `check.sh` | a script can see it in a run's transcript or files |
   | runner flags (`evals/run.sh`) | a tool or permission setting can prevent it |
   | case `rubric.md` | only a grader can see it |
   | the owning skill's `SKILL.md` | only wording can change the behaviour |
   | `KERNEL.md` | the wording applies to every skill |
   | note (README) | no skill owns the behaviour and nothing can observe it |

   An eval check detects a failure but doesn't prevent it. A model habit therefore usually needs wording in the owning skill plus a check from step 5. Record every stronger level you passed over and why it didn't fit.
5. **Leave a regression check that fails on the incident.** Use the cheapest kind that can:
   - For a lint rule: add a fixture in `evals/lint-fixtures/<name>/` whose `expect` names the specific error, plus a near-miss fixture that must still pass (`expect` is `ok`). Run `evals/lint-test.sh`.
   - For behaviour: a new or existing eval case in `evals/cases/`. Its prompt must differ on the surface from the incident (a different domain and wording) so the fix isn't fitted to one run.
   - Regression checks live only in those two places. Don't invent new check directories.
   - The original incident counts as the red run; cite it. Run the green run with `prove` and log its P id in the same slug. A paid eval gets one green run. Ask before running more.
6. **Gate on sign-off.** Any edit to `KERNEL.md`, and any edit that removes or loosens a rule or check (lets through something it used to reject or forbid), is not applied. Write the exact proposed diff into the L entry, mark it `awaiting sign-off`, and return. Still write the regression check, since that is an addition; its green run waits for the sign-off. Additions and tightenings elsewhere go ahead.
7. **Read the edit cold.** Would the rule make sense to someone who never saw the incident? If it quotes the incident, rewrite it.
8. **Record the entry.** Cite the L id in the commit message.

## Entry format

```markdown
## L2: <the class of failure, in one line>
Evidence: <ids, paths, shas>. Bar met by: <two incidents | reproduced by `<command>`>
Level: <chosen level>, <path>
Passed over: lint (why not), check.sh (why not), …
Regression check: <path or command>. Red: <the incident>. Green: <P id>
Removed: <redundant text deleted, or none>
Status: applied | awaiting sign-off (proposed diff below)
```

Candidates (`work/lessons/candidates.md`):

```markdown
## <the class of failure, in one line>
Seen: <date>, <evidence>. Needs: a second independent incident, or a deterministic repro
```

## Rules

- Fix the class, not the instance. Repairing the one bad file the incident touched doesn't codify anything.
- Wording is near the bottom of the ladder, not the default.
- One lesson per entry. Never codify from memory: no evidence, no edit.
