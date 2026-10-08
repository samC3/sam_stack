---
name: prove
description: Establish whether something works by defining a check that could fail, running it against the real surface, and recording what was observed. Use before saying done, works or fixed; to reproduce a bug; to write the failing test first; to validate a measured number; for "verify", "check it works", "reproduce", "prove it", "is this actually faster".
---

# Prove

Turn a claim into an observation. A check that has never failed proves nothing.

## Contract

```
Layer:     verb
Input:     claim; surface (the program, endpoint, UI or command where the claim must hold); slug
Output:    entry P<n> appended to $WORK/<slug>/proofs.md
Stop:      a verdict (holds | fails | inconclusive) is recorded, backed by an observed result
Returns:   the verdict in one line, plus the P id
```

## Procedure

1. **State the claim precisely,** along with the observation that would falsify it. If you can't name a falsifying observation, the claim is too vague. Sharpen it, or hand it to `ask`.
2. **Choose the strongest check you can run.** From strongest to weakest: drive the real program, endpoint or UI; integration test; unit test; type check; reading code. If you don't use the strongest, note why.
3. **Show the check can fail.** Run it before the change (red), or break the code under test temporarily and watch the check fail. Record that it did.
4. **Run it for real.** Capture the exact command and the relevant output, trimmed rather than paraphrased.
5. **For numbers, explain the number:** baseline, repeat count and spread, what limits it, and whether it measures the thing claimed.
6. **Record the entry.**

## Entry format

```markdown
## P2: <claim>
Falsified by: <observation that would show it false>
Check: `<exact command>` on <surface> (strength: real surface | integration | unit | types | reading, and why)
Could fail: <how you saw it fail: red run, mutation>
Observed:
    <trimmed output>
Verdict: holds | fails | inconclusive (what would make it conclusive)
```

## Rules

- "Should pass" and "looks right" are not observations.
- Never weaken a check to make it pass. If the check is wrong, record that and replace it in a new entry.
- Prove doesn't fix failures. It returns them. A caller such as `loop` decides what to change.
- Inconclusive is an honest verdict. False confidence is not.
