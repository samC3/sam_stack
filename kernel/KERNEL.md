# Kernel

Rules every skill in this set obeys. It is in context only where an `AGENTS.md` includes it (see the README). Keep it short; anything that only some skills need belongs in those skills.

## Layers

| Layer | What it is | May invoke |
|---|---|---|
| **Verb** | One move that removes one kind of uncertainty or moves state forward: `ground`, `ask`, `distill`, `prove`, `judge`, `codify` | Other verbs, combinators |
| **Combinator** | Runs a verb in a particular way: `delegate` (isolated subagent), `loop` (repeat until a check passes) | Verbs |
| **Workflow** | Wiring: which verbs, in what order, with what parameters. Carries no discipline of its own | Verbs, combinators |

- Verbs never invoke workflows.
- A step in any skill that names another skill invokes that skill, so its procedure is in context. Doing the step inline from the skill's description doesn't count.
- Verbs and combinators live in `kernel/<name>/`, workflows in `workflows/<name>/`. Every skill is covered by at least one eval task: the `covers:` field in `evals/tasks/<name>/task.md` names it, and the task's `invoked <skill>` check shows it ran.
- Only `make-skill` and `codify` write skill files. Only `codify` edits this kernel.
- If a workflow starts explaining *how* to ask, verify, or review, that text belongs in a verb.
- A new verb is admitted only when about three workflows need something no existing verb plus parameters can express.

## Skill contract

Every `SKILL.md` has frontmatter (`name`, `description`) and then a `## Contract` section holding a ` ```contract ` block, one `Key: value` per line:

```contract
Layer:     verb | combinator | workflow
Input:     what the caller supplies (and defaults)
Output:    the artifact written, with path
Stop:      the observable condition that ends the skill
Returns:   what the caller gets back in the conversation
```

`evals/lint.py` checks every skill's frontmatter and contract, and every eval task.

The `description` says what the skill does and when to use it, including the phrases a user would type. Routing depends on it.

A skill with no stop condition is not finished.

## State

Skills communicate through files, not conversation memory, so work survives sessions, subagents and handoffs.

- State lives outside the project repo, in one folder per machine: `$SAM_STACK_WORK` (set in `~/.claude/settings.json` `env`; if unset, `~/.sam_stack/work`). Never commit it to the project.
- `$WORK` is `$SAM_STACK_WORK/<project>`. `<project>` is the name of the repo's main checkout, `basename "$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")"`, so every worktree and session of one repo resolves the same folder. With no repo, `$WORK` is `$SAM_STACK_WORK/_global`.
- A task's state is `$WORK/<slug>/`. The slug is kebab-case from the task. Reuse an existing directory when the task matches it.
- When the task is finished, move `$WORK/<slug>/` to `$WORK/_archive/<slug>/`.
- `$SAM_STACK_WORK/lessons/` is shared across all projects and never archived. Names starting with `_`, and `lessons`, are reserved: never a slug.
- Each verb appends to its own log, one entry per heading, with a stable id. The registry below lists every file a skill writes under `$WORK/<slug>/` or `$SAM_STACK_WORK/`, with its writer and ids. `evals/lint.py` rejects a row whose writer isn't a skill, and any repo-relative `work/` path in a skill.

```registry
# file                                    writer    ids
findings.md                               ground    F1, F2, …
decisions.md                              ask       D1, … (open items O1, …)
proofs.md                                 prove     P1, …
verdicts.md                               judge     J1, …
<format>.md                               distill   — (spec.md, handoff.md, review.md, …)
tickets/NN-<name>.md                      distill   —
lessons.md                                codify    L1, …
$SAM_STACK_WORK/lessons/candidates.md     codify    — (shared across projects, never archived)
```

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
