---
name: ask
description: Resolve what the human wants by interviewing them one question at a time, each with a recommended answer, until no open decision would change the outcome. Use when intent, preference, priority or a trade-off is unclear and can't be looked up; for "grill me", "poke holes in this", "help me decide", "flesh this out", or before writing anything non-trivial.
---

# Ask

Remove uncertainty about intent. Facts are `ground`'s job; ask covers what only the human can decide.

## Contract

```
Layer:     verb
Input:     topic; slug (existing decisions.md and findings.md are read first)
Output:    entries D<n> (decided) and O<n> (left open) appended to work/<slug>/decisions.md
Stop:      no open branch would change what gets built, or the human says stop
Returns:   decisions made, items left open with your default for each
```

## Procedure

1. **Load what's known.** Read `decisions.md` and `findings.md`. Don't re-ask anything decided.
2. **Ground the facts first.** For each unknown, ask yourself whether it's a fact or a preference. Facts go to `ground` before you ask anything.
3. **Build the decision tree.** List the open decisions. Order them so that decisions others depend on come first, then by cost of getting them wrong.
4. **Ask one question at a time.** Each question gets:
   - one line of context: why this matters now
   - the question
   - 2–4 options, with your recommendation first and the reason for it
   Use a structured question tool when one is available.
5. **After each answer:** log `D<n>`, then update the tree. Add branches the answer opened and drop branches it closed.
6. **Push back once.** If an answer contradicts a finding or creates a risk, say so with the evidence (`F2`), then accept the human's call and log it.
7. **Surface conflicts.** If an answer contradicts an earlier decision, ask which one wins and log the winner as superseding the other.

## Entry format

```markdown
## D4: <decision in one line>
Chose: <option>. Because: <the human's reason, or "accepted recommendation">
Rejected: <options and one-line why>
Depends on: D2 · Informed by: F1

## O2: <open question>
Default if unanswered: <your recommendation>
```

## Rules

- Never batch questions unless they are independent and trivial.
- No leading or rhetorical questions. A recommendation is stated openly, not smuggled in through the wording.
- Stop asking when the remaining branches wouldn't change what gets built. Diminishing returns is a valid stop condition.
