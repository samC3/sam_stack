---
name: shape
description: Turn an idea, a doc or rough notes on something to build into a spec and tickets that a build workflow can implement without the conversation. Grounds the facts, asks about every choice the input leaves open, writes the spec and tickets, then judges them against the input. Use for "shape this", "spec this out", "plan this feature", "turn this idea into tickets", "turn this doc into tickets", "get this ready to build". Not for writing up decisions that are already logged ("write the spec" goes to distill), or for an interview on its own ("grill me", "help me decide" go to ask).
---

# Shape

Wiring from raw input to build-ready artifacts. The habit: every choice in the spec and tickets traces to the input, an `F` id or a `D` id. A gap the input leaves open is asked about or recorded as open. It is never filled with an assumption.

## Contract

```
Layer:     workflow
Input:     the thing to build (an idea in the prompt, a doc path, a URL or an issue); slug (default: the feature's kebab-case name, so build and review find the files under the same work/<slug>/)
Output:    work/<slug>/findings.md, decisions.md, verdicts.md entries; work/<slug>/spec.md; work/<slug>/tickets/NN-<name>.md
Stop:      the latest J entry has no findings marked confirmed, or two fix rounds are done and the remaining defects are reported; or ask is waiting on the human's answer
Returns:   while ask is waiting: its one question. Otherwise: the spec and ticket paths, the ticket order, the open questions with their defaults, and any defects judge left unresolved, with ids
```

## Steps

1. **`ground`: what is being asked for, and what does it touch?** Sources: the input first, then the code, docs and history it touches. If `work/<slug>/findings.md` exists, reuse it and ground only what it doesn't answer. Record:
   - one F entry listing each requirement the input states, one per line, each tagged `[fact: input]` with a quote. Judge's coverage check in step 4 works from this list.
   - the facts the requirements depend on: existing code, conventions, constraints
   - what the input leaves unsaid. These become ask's topics, not assumptions.

2. **`ask`: settle every choice the input leaves open.** Topic: the gaps from step 1. Ask until no open branch would change what gets built. End the turn on each question. Don't write the spec or tickets until ask has stopped. Anything still open when ask stops is an `O` entry with its default.

3. **`distill` the spec, then the tickets.** Reader: a build workflow that has `KERNEL.md` and `work/<slug>/` and nothing else, and that will implement one ticket at a time.
   - Format `spec`. Out of scope lists what the input mentions but this work won't do. Open questions lists every `O` entry with its default.
   - Format `tickets`. Each ticket is a vertical slice and cites the `D`/`F` ids its acceptance rests on. A ticket that depends on an open question lists the `O` id under Depends on.

4. **`judge` the spec and tickets.** Stance: find-defects. Criteria, fixed before reading the drafts:
   - **Traceable:** every choice cites the input, an `F` id or a `D` id. Anything else is an invented decision.
   - **Covered:** every requirement from step 1 is in a ticket, in Out of scope, or in Open questions.
   - **Buildable:** every ticket's acceptance is observable, and its Verify is a command `prove` can run.
   - **Ordered:** Depends on has no cycles, and the first ticket depends on nothing open.
   - **Minimal:** nothing is in scope that neither the input nor a `D` entry asked for.

5. **Fix and re-judge, at most twice.** Send each finding judge marked confirmed to the step that owns it: an invented or missing decision to `ask` (step 2), a coverage or writing gap to `distill` (step 3). Then judge again. A round is one re-judge; waiting on the human's answer to ask doesn't use one up. After the second round, stop and report what's left.

## Rules

- Don't write code or start building. Shape ends at the tickets.
- Don't resolve an open question by picking its default yourself. Only the human closes an `O` entry.
- If the slug's `work/<slug>/` already has decisions, read them and don't re-ask.
