# Kernel

Rules every skill in this stack obeys.

## Layers

- Verbs live in `kernel/<name>/`. Every skill is covered by at least one eval task: a `Covers:` line in `evals/tasks/<name>/task.md` names it.
- Only `codify` edits this kernel, and only with the user's sign-off.

## Log entries

Each verb appends entries to its own log under `$WORK/<slug>/`, one per heading, with a stable id.

## Principles

1. Prove, don't claim.
2. Encode lessons in structure: lint, then test, then skill text, then note.
