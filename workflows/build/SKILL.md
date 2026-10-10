---
name: build
description: Implement the spec and tickets that shape wrote, one ticket at a time in dependency order, test first, to a fixed quality bar and with zero code comments, committing each ticket on a branch and ending with a PR write-up. Builds straight from the prompt only when every requirement in it is testable as stated. Use for "build this", "build the tickets", "implement the spec", "implement the tickets", "start building <slug>". Not for "shape this" or planning (shape), "review this branch" (adversarial-review), "verify" or "prove it" on its own (prove), or "make a skill" (make-skill).
---

# Build

Wiring from shape's tickets to committed, proven code. The habit: every acceptance criterion has a test that was seen failing before the code that passes it existed. It usually fails like this: the code gets written first, tests written afterwards pass on their first run, and comments, docstrings and extras slip in along the way.

## Contract

```contract
Layer:     workflow
Input:     slug (default: the feature's kebab-case name, matching shape's $WORK/<slug>/); or a prompt that states the requirements itself
Output:    code and tests in the repo, one commit per ticket on branch <slug>; $WORK/<slug>/findings.md, proofs.md, verdicts.md entries; distill's $WORK/<slug>/pr.md
Stop:      every ticket in this run is committed with a green P entry and a J entry with no confirmed findings, and pr.md is written (tickets waiting on an open O entry are listed, not built); or a ticket is still failing after two fix rounds, the run has stopped with it uncommitted, and the state is reported; or there are no tickets and the prompt isn't testable as stated, and the user was pointed to shape
Returns:   the branch, one line per ticket (committed | stopped, with P and J ids), the path to pr.md, and what blocks any stopped ticket
```

## Steps

1. **`ground`: what is to be built, and in what repo?** Sources: `$WORK/<slug>/spec.md`, `tickets/`, `decisions.md` and `findings.md` first, then the code. Record:
   - the ticket order, from Depends on. A ticket that depends on an open `O` entry is out of this run, and so is every ticket that depends on it, directly or through others.
   - the repo's conventions near the code the tickets touch (naming, structure, idioms, how tests are written), and the commands for its tests, lint, format and type checks. Note which of these the repo doesn't have.

   If there's no spec or tickets: one F entry lists each requirement the prompt states, one per line, as an observable criterion tagged `[fact: input]` with a quote. If a failing test can be written for every requirement without choosing behaviour the prompt leaves open, that list is the single ticket. Otherwise stop and recommend running `shape` first.

2. **Branch.** If on the default branch, switch to branch `<slug>`, creating it if needed. Otherwise stay on the current branch.

3. **For each ticket, in order:**
   1. **`prove` red.** Claim: each acceptance criterion holds. The check is a test through the public interface. Write the tests before any code they exercise, and record them failing.
   2. **Implement.** Write the least code that turns those tests green, in the repo's conventions.
   3. **`prove` green.** Run the ticket's tests and its Verify command, then the full test suite and whichever lint, format and type checks the repo has.
   4. **`judge` the ticket's diff.** Stance: find-defects. Criteria, fixed before reading the diff:
      - **Red first:** each acceptance criterion has a test with a red P entry from before its code existed.
      - **Conventions:** follows the repo conventions from step 1.
      - **Minimal:** nothing the ticket didn't ask for, no speculative abstraction, no dead code.
      - **Names:** names make intent clear without comments.
      - **Errors:** handled at boundaries, none swallowed.
      - **Tests:** assert behaviour through the public interface, not implementation details.
      - **Zero comments:** no added line is a comment or docstring, except a directive a tool reads.
   5. **Fix, at most twice.** A round is: fix every failing proof and every finding judge marked confirmed, then repeat substeps 3 and 4. If anything still fails after the second round, stop the run and leave the ticket uncommitted. Lint, format and type failures count as failing proofs.
   6. **Commit.** One commit per ticket. The message names the ticket and its green P id.

4. **`distill` the PR write-up,** once every ticket in this run is committed. List the tickets left out, with the `O` ids they wait on. Reader: a reviewer running adversarial-review, who has `$WORK/<slug>/` and the branch. Format: `pr`.

## Rules

- Zero comments. Code and tests that build writes contain no comments and no docstrings. The exceptions are directives a tool reads (shebang, encoding line, `type: ignore`, `noqa`, `eslint-disable`). Leave existing comments in code you don't change alone.
- Never weaken or delete a test to get green. If a test is wrong, say so in a new P entry and replace it.
- Shape's `spec.md`, `tickets/` and `decisions.md` are input. Don't edit them or close an `O` entry. If a ticket can't be built as written, stop and report it.
- Never push or open a PR. That's the user's call.
- Don't archive `$WORK/<slug>/`. Review reads it next.
