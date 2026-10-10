---
name: supervise
description: Run work in child Claude Code sessions inside herdr, one tab each (a worktree when it edits code), message them, report their status as a tree, and verify what they return. Children are ordinary interactive sessions the user can jump into, and can supervise their own children. Use for "start a session for", "spin up sessions for these", "run these in parallel sessions", "supervise", "check on my sessions", "how are my sessions doing", "tell the parser session to". Not for a one-shot subagent, work that fits inline, or running evals.
---

# Supervise

Start child sessions, steer them by message, and accept nothing they return until you've checked it.

Every node in the tree is the same thing: an interactive Claude Code session in a herdr pane, with this kernel loaded. The user is the root. They can jump into any session and drive it themselves; that's expected, not a failure.

## Contract

```contract
Layer:     combinator
Input:     the work for each child (a goal, its inputs as paths and ids, whether it edits code); slug (the current task's, per KERNEL.md State). Or a status request for $WORK/<slug>/sessions/
Output:    a session folder per child, $WORK/<slug>/sessions/<name>/, which is also the child's task state (brief.md, meta.json and sent.md from you; the child's logs and return.md); your prove entries in $WORK/<slug>/proofs.md
Stop:      every child's return is verified and accepted, or handed to the user (stopped, failed, or a decision only they can make). For a status request: every session reported, with any new done return verified first
Returns:   one line per session (name, herdr state, verified status, ids), then what needs the user
```

The mechanics are in `session.py`, next to this file (`python3 <this skill's directory>/session.py --help`). It only works inside a herdr pane. If `HERDR_ENV` isn't `1`, say so and offer to do the work inline instead.

## Session folders

A child's folder is its task state. The first message `session.py` sends tells the child the folder's absolute path. Its logs (`findings.md`, `proofs.md`, …) and `return.md` go there, and its own children go in the folder's `sessions/`. A child's ids are therefore scoped to its folder: `parser`'s P1 is `sessions/parser/proofs.md` P1, and your own ids stay in `$WORK/<slug>/`.

Don't archive the slug while any session in it is still open, or before the user has dealt with the children's branches.

## Procedure

1. **Decide whether a session is worth it.** A session suits work that runs long, needs steering, runs in parallel with other work, or that the user may want to drive. Each one is a full Claude Code session with its own context and cost. Otherwise do the work inline.

2. **Write the brief** to `$WORK/<slug>/sessions/<name>/brief.md`. Keep the name short and lowercase, since herdr agent names are the folder path joined with `-` and cap at 32 characters. Point at paths and ids instead of pasting prose:

   ```markdown
   Goal:    <what done looks like, observably>
   Inputs:  <paths and log ids>
   Reach:   <what it may read and change>
   Edits:   no | yes: you are on branch supervise/<name> in your own worktree; commit there
   Stop:    <the observable condition>
   Budget:  <rough size; stop and return `stopped` past it>
   Return:  when you finish, stop, or need a decision, write return.md in your folder:
              status: done | stopped | failed | asks
              ids: <ids in your folder's logs that prove it, e.g. P3>
              answer: <1–3 lines>
              blocker: <for stopped or failed: what blocked you and what would unblock it>
              question: <for asks>
            then say so in one line.
   ```

3. **Start each child** with `session.py start <folder>`, adding `--edit` when it changes code. An editing child gets its own herdr worktree on branch `supervise/<name>` (`git worktree list` shows where), so children never edit the same checkout. A child that doesn't edit gets a tab in your workspace, starting in the current directory (or `--cwd`).

4. **Wait on events, don't poll.** Run `session.py wait <folder> …` for the children still working, as a background command. You're woken when one of them goes idle, finishes, or is blocked. Handle it (steps 5 and 6), then wait again on the ones still working. If the user asks for an update in the meantime, run `session.py status $WORK/<slug>/sessions`, verify any new done return, and report.

5. **Handle the child that woke you.** Check its herdr state first, then its return:
   - **herdr state `blocked`:** an approval or question dialog is open in its pane. Don't answer approvals for the user. Run `herdr notification show "<name> needs you" --sound request` and tell the user which session.
   - **return is `-` or `stale`:** it went idle without writing a return for its latest message. `send` it a one-line request to write one.
   - **`asks`:** answer it if `ground` can settle the question from the repo or the logs. Otherwise use `ask` with the user, then `send` the answer.
   - **`stopped` or `failed`:** if a better brief or one answer would unblock it, `send` that. Otherwise hand it to the user with its blocker.
   - **`done`:** verify it (step 6).

6. **Verify before you accept.** A return is a self-report (principle 1).
   - Check that each id it names exists in the child's folder logs.
   - `prove` the central claim yourself, on the child's surface: its branch or worktree for code, the repo for a finding. Re-run the check; don't reread its proof. For a claim like "every place", re-run the search and compare.
   - If the check passes, accept the return. If it fails, `send` the session your `P` id and the failing output, and go back to step 4. Don't fix its work yourself, since it owns its branch.

7. **Report** one line per session, verified state first, then what needs the user. Merging a child's branch, closing its tab and removing its worktree are the user's calls. Don't do them unless asked, and only for verified work.

## Rules

- The user may drive any session by hand at any time. If they say they're driving one, don't `send` to it until they hand it back.
- Never relay a child's "done" or "tests pass" that you haven't verified. Call it "reported done, not verified".
- Don't close panes, tabs or worktrees you didn't create.
- A child that supervises uses this same skill. Its children live in its own folder's `sessions/`, so `status` on any folder shows the subtree.
