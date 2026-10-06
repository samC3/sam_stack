# Kernel

Rules every skill in this set obeys. It is in context only where an `AGENTS.md` includes it (see the README). Keep it short; anything that only some skills need belongs in those skills.

## Layers

| Layer | What it is | May invoke |
|---|---|---|
| **Verb** | One move that removes one kind of uncertainty or moves state forward: `ground`, `ask`, `distill`, `prove`, `judge`, `codify` | Other verbs, combinators |
| **Combinator** | Runs a verb in a particular way: `delegate` (isolated subagent), `loop` (repeat until a check passes) | Verbs |
| **Workflow** | Wiring: which verbs, in what order, with what parameters. Carries no discipline of its own | Verbs, combinators |

- Verbs never invoke workflows.
- Only `codify` edits skills or this kernel.
- If a workflow starts explaining *how* to ask, verify, or review, that text belongs in a verb.
- A new verb is admitted only when about three workflows need something no existing verb plus parameters can express.

## Skill contract

Every `SKILL.md` has frontmatter (`name`, `description`) and then a `## Contract` block:

```
Layer:     verb | combinator | workflow
Input:     what the caller supplies (and defaults)
Output:    the artifact written, with path
Stop:      the observable condition that ends the skill
Returns:   what the caller gets back in the conversation
```

The `description` says what the skill does and when to use it, including the phrases a user would type. Routing depends on it.

A skill with no stop condition is not finished.

## State

Skills communicate through files, not conversation memory, so work survives sessions, subagents and handoffs.

- Work lives in `work/<slug>/` at the repo root and is **committed** with the branch it belongs to. That way a worktree or another session on the branch already has it, and reviewers see the reasoning beside the change. With no repo, use `~/.claude/work/<slug>/`.
- The slug is kebab-case from the task. Reuse an existing directory when the task matches it.
- One slug belongs to one branch. If two branches must share a slug, renumber the incoming entries when merging so ids stay unique.
- The change that finishes the task deletes `work/<slug>/`; git history keeps the record.
- Each verb appends to its own log, one entry per heading, with a stable id:

| File | Written by | Id |
|---|---|---|
| `findings.md` | ground | `F1`, `F2`, … |
| `decisions.md` | ask | `D1`, … (open items `O1`, …) |
| `proofs.md` | prove | `P1`, … |
| `verdicts.md` | judge | `J1`, … |
| `<format>.md` (`spec.md`, `handoff.md`, …) | distill | — |

- Logs are append-only. To revise an entry, add a new one that says `supersedes F3`.
- Refer to entries by id instead of restating them.

## Evidence

Every load-bearing claim is tagged:

- `[fact: <source>]`: `path:line`, a command and its output, a commit sha, a URL, a transcript reference, or a log id (`P2`).
- `[inference]`: reasoning from facts. Say which facts.

An untagged claim is treated as an inference.

## Principles

1. **Prove, don't claim.** "Done", "works" and "fixed" need a `prove` entry observed on the real surface. A self-report is not evidence.
2. **Ground before you ask.** Never ask the human something you could find out. Ask only about intent, preference and trade-offs.
3. **Subtract before adding.** The smallest change that meets the need wins. Delete dead weight before building around it.
4. **Name the reader.** Every artifact is for a specific reader who must be able to act on it without the conversation.
5. **Guard the context window.** Send bulk reading, searching and attempts to a subagent; keep summaries and ids in the main thread.
6. **Encode lessons in structure.** Enforce at the strongest level available: type, lint, test, skill text, then note.
7. **Stopping is part of the work.** When the stop condition can't be reached within budget, stop and report state, what blocked you, and what would unblock it.
