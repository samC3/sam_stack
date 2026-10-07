---
name: adversarial-review
description: Review a finished piece of work, usually a PR or branch, from a fresh session. Find the goals it was written to meet, re-prove each one rather than trusting the author's claims, then hunt for defects. Use for "adversarial review", "review PR #12", "review this branch", "does this meet its goals", "does this meet the spec", "check this PR does what it says". Not for comparing or ranking options, or for reviewing a snippet that has no goals behind it; those go to judge.
---

# Adversarial review

Wiring for a cold review: what the work was meant to do, whether it does it, and what it breaks. The author's account is a lead, never evidence.

## Contract

```
Layer:     workflow
Input:     subject (PR number, branch, or commit range; default: current branch against the default branch); goals (optional; otherwise grounded); slug (default: review-<branch>)
Output:    work/<slug>/findings.md, proofs.md, verdicts.md entries; distill's work/<slug>/<format>.md with format review
Stop:      the J entry gives every goal a verdict (met | not met | unproven), each met or not met citing a P id and each unproven saying what was tried, the defects pass is recorded, and work/<slug>/review.md is written; or no goals could be found and the user has been asked for them
Returns:   merge / changes needed in one line, the goals not met or unproven, the top defects, and the path to review.md
```

## Steps

1. **`ground`: what is the subject?** Question: which commits and files does the change contain, against what base? Get names and stats only (`git log`, `git diff --stat`, `gh pr view`). The diff itself waits until step 3. Unless the subject is already checked out, check it out in a worktree (`git worktree add`) so the user's checkout stays put. The review's own `work/<slug>/` goes in the user's checkout, not the worktree.

2. **`ground`: what was it meant to do?** Question: which goals and acceptance criteria were set for this work? Sources, in order:
   - goals the caller passed in
   - the author's `work/<their-slug>/spec.md`, `decisions.md`, `tickets/`
   - the PR body and the issues it links

   Record each goal as an observable criterion. If no source states any goal, stop and `ask` the user for them. Never derive goals from the diff.

3. **`judge` the change.** Stance: find-defects. Criteria: the goals from step 2, fixed before reading the diff, plus "introduces no regression". Give each goal a verdict of met, not met or unproven.

4. **`prove` each goal's verdict.** Surface: the subject's worktree, using the real program, endpoint or test suite. The author's `P` entries, PR body and "tested" claims say where to look; re-run them, don't cite them. If a goal can't be checked, its verdict is unproven, not met.

5. **`distill` the review.** Reader: the author, who will fix the work without this conversation. Format: `review`, with this shape: verdict (merge / changes needed) · one row per goal, giving its verdict and `J`/`P` ids · defects ranked by consequence, each with its failure scenario · for unproven goals, what would prove them.

## Rules

- The author's `work/` files are input. Never append to them. The review's logs go in its own `work/<slug>/`.
- Don't edit, fix or commit the work under review, and don't commit the review. The user decides where it lives.
- Post to GitHub only when the user asks.
- If this conversation wrote the change, say so and recommend running the review in a new session.
