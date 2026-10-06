---
name: distill
description: Compress current state (conversation, decisions, findings, proofs) into a durable artifact for a named reader in a given format - spec, tickets, handoff, PR body, summary, or skill. Use whenever work must outlive the conversation or move to another person, agent or session; for "write the spec", "ticket this up", "hand this off", "write the PR", "summarise", "write a skill for this".
---

# Distill

Turn state into something a named reader can act on without the conversation.

## Contract

```
Layer:     verb
Input:     reader (who they are, what they already know, what they will do next); format; source ids or "this conversation"; slug
Output:    work/<slug>/<format>.md (tickets: work/<slug>/tickets/NN-<name>.md; skill: kernel/<name>/SKILL.md for a verb or combinator, workflows/<name>/SKILL.md for a workflow; eval-case: evals/cases/<name>-<habit>/)
Stop:      a cold reader, given only the artifact, states the intended next action and has no questions the sources already answer
Returns:   the path, plus any gaps marked as open questions
```

## Formats

| Format | Shape |
|---|---|
| `spec` | Problem · Outcome and acceptance · Decisions (by id) · Out of scope · Open questions |
| `tickets` | One file per vertical slice: Goal · Acceptance · Verify (a command `prove` can run) · Depends on |
| `handoff` | Where things stand · Next action · Traps already found · Files and ids to read first |
| `pr` | What changed and why · How it was proved (`P` ids) · Risk · Where the reviewer should look |
| `summary` | Answer first, then support, then open questions |
| `skill` | Frontmatter + `## Contract` per KERNEL.md, then Procedure, entry format if it writes a log, and Rules |
| `eval-case` | `prompt.md` that invites the habit's typical failure · `fixture/`, the smallest set of files that still tempts that failure · `check.sh` for everything a script can check · `rubric.md` only for what a script can't. It must be able to fail |

Any other format is allowed. Define its shape from the reader's next action before you draft.

## Procedure

1. **Name the reader and their next action.** Write both down in one line each. Every later choice serves that action.
2. **Gather the sources.** Keep only what the reader needs to act. Cite ids (`D3`, `F1`, `P2`) instead of restating arguments.
3. **Draft.** Lead with what the reader must do or know. Be concrete: real paths, real commands, real names. Strip conversational narrative ("we discussed", "after some back and forth").
4. **Mark gaps; don't fill them.** If the sources don't settle something the reader needs, write it as an open question. Never invent a decision.
5. **Cold-read test.** Required for `spec`, `tickets`, `handoff` and `skill`; optional otherwise. Give a fresh subagent only the artifact and the reader description. Ask it what it would do next and what is unclear. Fix the gaps it finds and repeat until its answer matches the intent.
6. **Write the file.**

## Rules

- One reader per artifact. If two readers need different things, write two artifacts.
- Shorter is better when the reader can still act on it. Cut anything the reader wouldn't miss.
- Distill never decides anything. It records decisions and exposes gaps.
