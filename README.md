# skills_v3

A small set of composable agent skills, built up from an atomic core.

```
kernel/
  KERNEL.md          # rules every skill obeys: layers, skill contract, state, evidence, principles
  ground/SKILL.md    # what's true        → work/<slug>/findings.md   (F1…)
  ask/SKILL.md       # what the human wants → work/<slug>/decisions.md (D1…, O1…)
  distill/SKILL.md   # write it down for a named reader
  prove/SKILL.md     # does it work       → work/<slug>/proofs.md     (P1…)
  judge/SKILL.md     # is it good / which one → work/<slug>/verdicts.md (J1…)
evals/
  run.sh             # eval runner
  lib.sh             # helpers for check.sh files
  cases/<name>/      # one directory per case
```

## Rules

1. **The kernel must be in context wherever these skills are used.** Include `kernel/KERNEL.md` in the `AGENTS.md` that the session reads, either your global one or a repo's own:

   ```markdown
   @~/code/skills_v3/kernel/KERNEL.md
   ```

   Claude Code reads `AGENTS.md` and expands `@` imports in it. An import of a file *outside* the project needs a one-time approval in an interactive session. In a headless run (`claude -p`) it silently doesn't load. For a tool that doesn't support imports, paste the contents of `KERNEL.md` instead and re-paste it when the file changes.

   Check that it loaded by asking, without file reads: *"Quote principle 2 from the kernel in your context, or say NONE."*

2. **Install skills by linking, not copying,** so edits here take effect everywhere:

   ```sh
   for s in ~/code/skills_v3/kernel/*/; do ln -s "$s" ~/.claude/skills/"$(basename "$s")"; done
   ```

3. **Every skill follows the contract in `KERNEL.md`.** New skills are compositions of the kernel verbs. Add a new verb only when the kernel's admission rule allows it.

## Evals

Evals check that each verb keeps the one habit it exists for. They run the real skills in headless Claude Code sessions, so **they spend real tokens.** Read [Spending](#spending) before running them.

### Running

```sh
evals/run.sh                              # every case, once
evals/run.sh prove-red-first              # named cases only
evals/run.sh --repeat 3 --jobs 6 ask-cache-grill
```

Each run goes to `evals/.runs/<timestamp>/<case>/r<n>/` (gitignored), which holds:
- `repo/`: the fixture after the run
- `transcript.jsonl` and `digest.txt`: what the agent did
- `grade.md`: the grader's verdict

The script prints PASS or FAIL for each run, then a summary of passes per case. It exits non-zero if any run failed.

Each case runs in a fresh git repo, set up the way a real repo would be:
- `KERNEL.md` is imported from `AGENTS.md` with a relative path. Imports from outside the project don't load in headless runs.
- Each verb is linked into `.claude/skills/`.
- Only project settings load (`--setting-sources project`), so your user-level skills and memory don't interfere.

### Writing a case

```
evals/cases/<name>/
  prompt.md    # the user prompt (required)
  fixture/     # files copied into the run repo (optional)
  setup.sh     # runs in the repo after the first commit, e.g. to add a commit to review (optional)
  check.sh     # deterministic checks (optional)
  rubric.md    # criteria for a grader model (optional)
```

`check.sh` runs first, with these helpers from `lib.sh` available:
- `invoked <skill>`
- `invoked_none`
- `has <glob> <regex>`
- `fail <msg>`

The grader runs only if the checks pass. It's a separate Claude session that reads the rubric, a digest of the transcript and the files under `work/`. It must end with `VERDICT: PASS` or `VERDICT: FAIL`.

Put everything you can into `check.sh`. It's free and it doesn't vary between runs. Use the rubric only for things a script can't judge.

A good case targets the specific failure a verb is prone to, and **must be able to fail.** A case that every reasonable version of the skill passes only guards against regressions.

### Current cases

These are a seed set: one case per verb's key habit, plus one negative trigger. Results so far are from 4 runs per case (one initial run, then 3 repeats), all before the `ask` fix:

| Case | Tests that the verb… | Passes |
|---|---|---|
| `ground-config-override` | finds a production override (5 retries, not the default 3), cites it, and doesn't ask the user | 4/4 |
| `ask-cache-grill` | reads the repo first, then asks exactly one question with a recommendation | 2/4 (follow-up questions; fixed in the skill, not yet re-run) |
| `distill-open-gap` | keeps an open question open and invents no decisions | 3/4 |
| `prove-red-first` | shows the check failing before the fix and passing after | 4/4 |
| `judge-clean-diff` | reports no defects on a correct change and states criteria first | 3/4 |
| `trigger-negative-rename` | doesn't fire any kernel verb for a plain rename | 4/4 |

The single failures for `distill` and `judge` could be noise from the skill or from the grader. Leave them unchanged unless they come back.

### What these cases are and aren't good for

The seed cases are **unit tests for one habit each**: cheap, easy to read, and good at catching regressions. Their limits:

- **They're easy.** A toy repo with three files and one planted trap makes the right move obvious.
- **They're single-turn.** Every case is one prompt in a fresh session. Many real failures happen later: the kernel drifting out of attention in a long session, after compaction, when verbs are chained, or when other installed skills compete for the same prompt.

New effort should go into **cases mined from real sessions:**
- a snapshot of a real repo at a commit,
- the prompt that actually went wrong,
- and checks for the failure you saw.

A failure you've seen is worth more than one you imagine.

**Language** matters little for these verbs, because they test habits rather than coding skill. The fixtures use Python because `python3` ships with macOS and a bare-`assert` test needs no dependencies. Language starts to matter for realistic cases:
- `prove` choosing the strongest check: typed builds, real test runners, flaky tests.
- `ground` finding its way through real conventions: config layering, monorepos, generated code.

So write realistic cases in the stacks you actually work in.

### What a gold-standard eval would add

| | Seed cases | Gold standard |
|---|---|---|
| Source | Invented | Mined from real sessions, plus adversarial cases built to provoke a known failure |
| Fixtures | 3–5 file toy repos | Real repos snapshotted at a commit, at realistic size with distractions |
| Turns | One prompt, one turn | Multi-turn. A simulated user with hidden preferences answers `ask`. Sessions resume or compact partway through |
| Comparison | Absolute pass/fail | Paired and blind: the new skill against the old one (or no skill) on the same cases. Report the improvement |
| Repeats | 1–4 | Enough for a pass rate with a confidence range |
| Grader | One unchecked Claude run | Deterministic where possible. The grader is checked against human-labelled good and bad transcripts so its accuracy is known |
| Coverage | One habit per verb, one negative trigger | Should and shouldn't trigger for each verb, competition with other installed skills, chained verbs in workflows, cost and turns tracked |
| Overfitting guard | None | A held-out set the skill text is never tuned against |

The two that matter most are **paired comparison** and **checking the grader against human labels**. Without them, you can't tell whether a change helped or whether the grader is just noisy.

### Spending

The agent under test costs about $0.10–0.20 per run. `distill` costs more because its cold-read step starts a subagent. Each graded run adds a grader session on top. The first 24 runs cost roughly $7.

Run an eval only when its result would change what you do:

1. **Tier the runs.**
   - After editing a skill: run that skill's cases once.
   - Before committing a kernel change: run the full suite once.
   - Repeat only the cases whose result is ambiguous.
2. **Spend repeats where you're uncertain.** A case that has passed 4 out of 4 needs one run as a regression check. A case at 2 out of 4 is where repeats teach you something.
3. **Stop when the answer is clear.** If the first 3 runs fail the same way, you don't need 5.
4. **Use the cheapest check first.** Deterministic checks gate the grader, so push criteria into `check.sh` where you can.
5. **Use a cheaper grader, but the real model under test.** The grader can run on a smaller model once it has been checked against human labels. The agent under test should be the model you actually use, or the results won't carry over.

Not built yet:
- a per-case cost column in the results (the transcripts already record cost),
- a mode that runs only the cases for skills that have changed.
