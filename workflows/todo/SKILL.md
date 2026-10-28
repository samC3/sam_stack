---
name: todo
description: Keep the user's personal todo list, bullet-journal style, across all projects in $SAM_STACK_WORK/todo/: one file per day, a backlog, and a details file per item when needed. Triages yesterday's open items on the first use of a new day. Use for "add X to my todo", "put X on my list", "what's on my list", "what should I do next", "push X to the backlog", "pull X from the backlog", "add a note to X", "I started X", "mark X done", "review my list". Not for "make a todo list for this task", "plan the steps" (Claude's own task list), or "shape this" (shape).
---

# Todo

Keep one list the user can trust: every item is in exactly one place, and nothing open is lost or silently dropped.

## Contract

```contract
Layer:     workflow
Input:     what the user asked: add, done, start, push back, pull, note, review, or show the list
Output:    edits to $SAM_STACK_WORK/todo/: days/YYYY-MM-DD.md, backlog.md, details/<name>.md
Stop:      the request is applied, every earlier open item has been triaged, and the open items are shown to the user; or a question to the user is waiting
Returns:   today's open items, then the backlog's count, then anything that needs the user
```

## Files

```
$SAM_STACK_WORK/todo/
  days/YYYY-MM-DD.md   one log per day, headed `# YYYY-MM-DD`
  backlog.md           parked items, a flat list with no heading and no dates
  details/<name>.md    notes for one item, headed `# <item>`; <name> is the item in kebab-case
```

Each item is one line: a status word, two or more spaces, the item, a `[project]` tag, then optionally `→ <slug>` and a details link.

```markdown
# 2026-10-29
TODO     Tree view TUI  [sam_stack] → tree-view  [details](../details/tree-view-tui.md)
DONE     Shape todo skill  [sam_stack] → shape-todo
MOVED    Archive summaries  [sam_stack] (to backlog)
MOVED    Language standards doc  [sam_stack] (to 2026-10-30)
DROPPED  Organise skill  [sam_stack]
NOTE     archive summaries should cite ids
```

- Status words: `TODO` open, `DONE` finished, `MOVED` now lives elsewhere (`(to YYYY-MM-DD)` or `(to backlog)`), `DROPPED` abandoned, `NOTE` not a task.
- `[project]` is the current repo's `<project>` (KERNEL.md State), or `_global` outside a repo.
- `→` marks only a slug. A move is shown by `(to …)` at the end of the line, after any slug or link.
- In `backlog.md` the details link is `details/<name>.md`; in a day file it's `../details/<name>.md`.
- When a line moves, its tag, slug and details link move with it.
- Lines are never deleted from a day file. They change status. The only deletion is pulling a line out of `backlog.md`.

## Procedure

Run these in order, every time.

1. **Triage first.** If `days/<today>.md` doesn't exist, find the most recent earlier day file. If it has `TODO` lines, list them, numbered, and ask the user for each: carry, done, backlog or drop ("carry all" is fine). End your turn on that question. Then apply each answer:
   - **carry:** the old line becomes `MOVED … (to <today>)`; a `TODO` copy goes into today's file.
   - **done:** `DONE`.
   - **backlog:** as push back (step 2).
   - **drop:** `DROPPED`.

   Create `days/<today>.md` with its heading even if nothing was carried. Never carry, drop or backlog an item without the user's answer.

2. **Apply the request,** to today's file unless it says otherwise:
   - **add:** append `TODO     <item>  [<project>]`.
   - **done:** `TODO` → `DONE`.
   - **start:** append `→ <slug>`, using the slug the user names, or else the item in kebab-case. Don't create `$WORK/<slug>/`; the work that follows does.
   - **push back:** the day-file line becomes `MOVED … (to backlog)`; append the item as `TODO` to `backlog.md`.
   - **pull:** delete the item's line from `backlog.md`; append it as `TODO` to today's file.
   - **note:** append the note as a `- ` line to `details/<name>.md`, creating it with `# <item>` if needed, and add the details link to the item's line if it's missing.

   If an item name matches more than one line, or none, ask which one. Don't guess.

3. **Review,** when asked or when showing the list: mark `DONE` every open item with `→ <slug>` whose `$SAM_STACK_WORK/<project>/_archive/<slug>/` exists.

4. **Report:** today's open items, numbered; the number of backlog items; and any question still waiting.

## Rules

- These files are the user's journal. Edit only the lines the request touches.
- Don't decide priorities, and don't reorder the user's lines.
- Don't write anywhere outside `$SAM_STACK_WORK/todo/`.
