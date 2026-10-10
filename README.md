# sam_stack

A small set of composable agent skills, built up from an atomic core.

```
kernel/
  KERNEL.md          # rules every skill obeys: layers, skill contract, state, evidence, principles
  ground/SKILL.md    # what's true        → $WORK/<slug>/findings.md   (F1…)
  ask/SKILL.md       # what the human wants → $WORK/<slug>/decisions.md (D1…, O1…)
  distill/SKILL.md   # write it down for a named reader
  prove/SKILL.md     # does it work       → $WORK/<slug>/proofs.md     (P1…)
  judge/SKILL.md     # is it good / which one → $WORK/<slug>/verdicts.md (J1…)
  codify/SKILL.md    # make a lesson structural → $WORK/<slug>/lessons.md (L1…); the only skill that edits KERNEL.md
  supervise/         # combinator: child Claude sessions in herdr tabs/worktrees → $WORK/<slug>/sessions/<name>/
    SKILL.md, session.py, session_test.py
workflows/
  make-skill/SKILL.md  # builds or regenerates a skill, plus the eval task that covers it, from the verbs
  adversarial-review/SKILL.md  # cold review of a PR/branch: grounds its goals, re-proves each, then hunts defects
  shape/SKILL.md       # idea or doc → spec.md + tickets/ for a build workflow; asks rather than assumes
evals/
  lint.py            # free structural check of every skill, task and the registry
  lint_test.py       # its tests: one row per rule, each with a near-miss that must pass
  run.py             # eval runner; --graders tests each task's checks for free
  prompts/           # the grader's and the simulated user's prompts
  lib.sh             # helpers for a task's checks.sh
  tasks/<name>/      # one directory per eval task
```

## Rules

1. **The kernel must be in context wherever these skills are used.** Include `kernel/KERNEL.md` in the `AGENTS.md` that the session reads, either your global one or a repo's own:

   ```markdown
   @~/code/sam_stack/kernel/KERNEL.md
   ```

   Claude Code reads `AGENTS.md` and expands `@` imports in it. An import of a file *outside* the project needs a one-time approval in an interactive session. In a headless run (`claude -p`) it silently doesn't load. For a tool that doesn't support imports, paste the contents of `KERNEL.md` instead and re-paste it when the file changes.

   Check that it loaded by asking, without file reads: *"Quote principle 2 from the kernel in your context, or say NONE."*

   This repo uses its own skills: `AGENTS.md` imports `kernel/KERNEL.md`, and `.claude/skills/` holds relative links to every skill. A session opened here has the kernel and the skills without anything installed in `~/.claude`. Add a link there when you add a skill.

2. **Install skills by linking, not copying,** so edits here take effect everywhere:

   ```sh
   for s in ~/code/sam_stack/kernel/*/ ~/code/sam_stack/workflows/*/; do
     [ -f "$s/SKILL.md" ] && ln -s "$s" ~/.claude/skills/"$(basename "$s")"
   done
   ```

3. **Set where state lives, once per machine.** Skills keep task state outside your repos, in `$SAM_STACK_WORK` (default `~/.sam_stack/work`), so every session and worktree finds it the same way:

   ```json
   // ~/.claude/settings.json
   { "env": { "SAM_STACK_WORK": "/Users/you/.sam_stack/work" } }
   ```

   Layout: `<project>/<slug>/` for a task, `<project>/_archive/<slug>/` once finished, `_global/<slug>/` outside a repo, `lessons/candidates.md` across projects. `KERNEL.md`'s State section has the rules.

4. **Every skill follows the contract in `KERNEL.md`.** New skills are compositions of the kernel verbs. Add a new verb only when the kernel's admission rule allows it.

5. **Make skills with `make-skill`.** It wires `ground`, `ask`, `distill`, `prove` and `judge` together to write the skill and an eval task that covers it (or extend an existing one), then checks both with `evals/lint.py` and `evals/run.py --graders`. It runs a paid eval only once per new task; anything beyond that, it asks you first.

## Evals

Evals are a small set of **tasks**: realistic scenarios that run a workflow end to end, so each one exercises several skills at once. They're graded on the **outcome** (what ended up in the repo and in `$WORK`), not on which skills fired. This follows Anthropic's [Demystifying evals for AI agents](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents). Tasks run the real skills in headless Claude Code sessions, so **they spend real tokens.** Read [Spending](#spending) before running them.

### Running

```sh
evals/lint.py                             # free: structure of every skill and task
evals/lint_test.py                        # free: lint.py still rejects what it should, and only that
evals/run.py --graders                    # free: each task's checks fail untouched and pass on its reference
evals/run.py                              # every task, once
evals/run.py review                       # named tasks only
evals/run.py --repeat 3 --jobs 6 plan

# Baseline comparison: run against the skills as of HEAD
git worktree add /tmp/skills-base HEAD
evals/run.py --root /tmp/skills-base plan
git worktree remove /tmp/skills-base
```

Each run goes to `~/.cache/sam_stack-evals/<timestamp>-<pid>/<task>/r<n>/` (override with `EVALS_OUT`). It lives outside this repo because a run repo nested inside it would inherit its `AGENTS.md`. After each trial, the runner deletes the state Claude Code keeps for the run, grader and simulated-user folders (`~/.claude/projects/…`, `/private/tmp/claude-<uid>/…`), so runs don't pile up there. Each run folder holds:
- `repo/`: the fixture after the run
- `work/`: the run's `$SAM_STACK_WORK`, with the task state under `work/repo/`
- `transcript.jsonl`, `digest.txt` and `conversation.md`: what the agent did and said, turn by turn
- `checks.txt` and `grade.md`: one line per check and per rubric dimension

Each trial prints a line like `plan  r1  FAIL  checks 4/5  rubric 4/5  turns 6`, with each failed check and each failed or UNKNOWN rubric dimension listed under it. That gives partial credit and shows which part of the workflow broke. A trial passes only if every check and every dimension passes. The script ends with passing trials per task, and exits non-zero if any trial failed. **Read the transcripts** of new or failing tasks: a grader's verdict isn't trustworthy until you've checked a few by hand.

Each trial runs in a fresh git repo, set up the way a real repo would be:
- `KERNEL.md` is imported from `AGENTS.md` with a relative path. Imports from outside the project don't load in headless runs.
- Each skill in `kernel/` and `workflows/` (or the `--root` checkout) is linked into `.claude/skills/`.
- Only project settings load (`--setting-sources project`), so your user-level skills and memory don't interfere.
- `HERDR_*` variables are dropped, so a run started from a herdr pane can't drive your real herdr. Tasks that need herdr fake it in `bin/`.

### Tasks

| Task | Scenario | Covers |
|---|---|---|
| `review` | A branch whose PR claims "done and tested" but misses one of three spec criteria | adversarial-review, ground, judge, prove, distill |
| `plan` | "Add caching", in a repo whose README says 4 replicas while the deploy config says 6. A simulated user holds the preferences and leaves one point undecided | ground, ask, distill |
| `shape` | "Let users export their notes", in a small CLI. A simulated user picks format, destination and scope, and leaves restore undecided. The spec and tickets must keep restore open, invent nothing, and give each ticket a runnable Verify | shape, ground, ask, distill, judge |
| `make-skill` | Build a small workflow with every decision given, and be honest that the paid eval wasn't run | make-skill, ground, distill, prove |
| `supervise` | Two child sessions have settled; one returns "all tests pass" but a test fails on its branch. Asked to merge what's finished, the supervisor must re-run the check, not merge, and send the evidence back (herdr is faked) | supervise, prove |
| `codify` | Three lessons at once: a reproducible lint gap (fix it in lint), one unreproduced failure (log a candidate, change nothing), and a rule for every skill (propose a kernel diff, wait for sign-off) | codify, prove |

A task is `evals/tasks/<name>/`:

```
task.md              # frontmatter (covers: <skills>, turns: <max>, env: KEY=VALUE …), then what it's for
prompt.md            # the first user message
user.md              # optional: a simulated user's hidden situation, which makes the task multi-turn
fixture/  work/  setup.sh   # the repo, the task state in $WORK, and setup after the first commit
bin/                 # fake tools put first on the PATH (e.g. a fake herdr)
checks.sh            # named outcome checks: check "<label>" <command ...>
rubric.md            # numbered dimensions, each graded PASS / FAIL / UNKNOWN by a model
reference/apply.sh   # produces a correct outcome, for --graders
```

How a task should be built:
- **Grade the outcome.** Check files, verdicts and git state, not which skills fired.
- **Give every task a reference solution,** and run `--graders` before any paid run. A check that can't pass on a correct outcome is a broken task, not a weak agent.
- **Make it multi-turn when the workflow asks the user things.** `user.md` gives the simulated user preferences, plus at least one point it hasn't decided, so you can see whether the agent keeps it open.
- **Use the rubric only for what a script can't see,** one dimension per line. UNKNOWN counts as not passing, so a vague dimension shows up instead of passing silently.
- **Every skill is covered by at least one task.** Lint checks each task's `covers:` and rejects names that aren't skills.

`setup.sh` and `reference/apply.sh` get `$ROOT` (this repo), and `$WORK` and `$SAM_STACK_WORK` (the run's task state). `checks.sh` runs in the run repo with `lib.sh` loaded: `check "<label>" <command ...>` records one named result, and `has <glob> <regex>` is the usual command. A run that creates a `work/` folder in the repo fails. The grader is a separate Claude session. It reads the rubric, a digest of the transcript, and the outcome: the run's task state, the repo's diff since setup, and any new files.

### Results so far

First paid run, one trial each (about $2 for the agents, plus simulated-user and grader calls):

| Task | Checks | Rubric | What it showed |
|---|---|---|---|
| `review` | 6/7 | 4/4 | The review was right. The one failed check was an eval bug (too strict a pattern), since fixed |
| `plan` | 5/5 | 3/5 | A harness bug: the simulated user saw only the agent's last text block, so it never saw the options. Since fixed. A real `distill` flaw too: after the cold read it filled gaps with values nobody chose (a 10 ms target) instead of marking them open |
| `make-skill` | 5/5 | 3/4, 1 unknown | An eval bug: the grader couldn't see files in the repo. Since fixed: it now gets the repo's diff and new files |

Read the transcripts before trusting a score. Two of these three failures were in the evals, not the skills.

### Not built yet

- **More tasks.** `fix` (prove, later iterate), and a routing table of prompts that should and shouldn't trigger each skill. `codify` hasn't had a paid run yet.
- **Tasks from real sessions.** Snapshot a real repo at a commit, with the prompt that went wrong. These beat invented scenarios.
- **Grader calibration.** Mark a few grades right or wrong by hand, so the grader's accuracy is known.
- **A held-out task** that skill wording is never tuned against.
- **A cost column** in the results. The transcripts already record cost.

### Spending

A task trial costs roughly $0.40–1 for the agent. A multi-turn task costs more: `plan` was about $1 over three turns. Each trial adds a grader call, and each turn of a multi-turn task adds a simulated-user call.

Run an eval only when its result would change what you do:

1. **Tier the runs.**
   - After editing a task: `evals/run.py --graders` (free).
   - After editing a skill: run the tasks that cover it, once.
   - Before committing a kernel change: run every task once.
   - Repeat only the tasks whose result is ambiguous.
2. **Spend repeats where you're uncertain.** A task that has passed 4 out of 4 needs one trial as a regression check. A task at 2 out of 4 is where repeats teach you something.
3. **Stop when the answer is clear.** If the first 3 runs fail the same way, you don't need 5.
4. **Use the cheapest check first.** Run `evals/lint.py` and `evals/run.py --graders` before any paid run; both are free. Push criteria into `checks.sh` where you can, since checks are free and don't vary between runs.
5. **Use a cheaper grader, but the real model under test.** The grader can run on a smaller model once it has been checked against human labels. The agent under test should be the model you actually use, or the results won't carry over.
