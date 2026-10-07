# Findings: make-skill-codify

## F1: Does an existing skill already cover codify?
Answer: No. Nothing else may edit the kernel, and no `codify` directory exists. Its layer is already set.
- Only `make-skill` and `codify` write skill files, and only `codify` edits the kernel [fact: kernel/KERNEL.md:16-17]
- make-skill: "Until `codify` exists, propose kernel changes to the user as open questions" [fact: workflows/make-skill/SKILL.md, Rules]
- `codify` is listed as a verb in the layer table, so the admission rule doesn't apply [fact: kernel/KERNEL.md:9]
- No `kernel/codify/` [fact: `git ls-files`]
Checked against: a verb with parameters that could do this. `distill` format `skill` writes skills, but it "never decides anything" [fact: kernel/distill/SKILL.md, Rules], and choosing the enforcement level is a decision.

## F2: Lesson A, follow-up questions in ask: evidence, fix, and its level
Answer: The failure appeared in 2 of 4 runs. The fix was one line of skill text. The guard is an LLM rubric, not a script, and the fix has not been re-run.
- r1 failed: "If it's 2 or 3, I'd also like to know your latency target" [fact: ~/.cache/skills_v3-evals/20261006-193457/ask-cache-grill/grade.md]
- A repeat run failed: "If it's B or C, what is the sleep standing in for?" [fact: …/20261006-203000/ask-cache-grill/r2/grade.md]. The other two repeats passed.
- Fix: one sentence in step 4 [fact: kernel/ask/SKILL.md:31, commit c2d9d25]
- Guard: rubric criterion 1 [fact: evals/cases/ask-cache-grill/rubric.md:1]. check.sh only tests `invoked ask` [fact: evals/cases/ask-cache-grill/check.sh]
- README lists it as "fixed in the skill, not yet re-run" [fact: README.md, Current cases table]
- Part of the criterion could be scripted, for example at most one `?` outside quoted options in the final message [inference from the two grade quotes: both failures end in a second `?` sentence]

## F3: Lesson B, lint accepted a tab-only field: evidence, fix, and its level
Answer: One incident, found by an agent rather than in use. Fixed in lint.sh. Lint has no test of its own, so nothing would catch a regression.
- Found while the make-skill eval agent wrote a case prompt: "Look into why evals/lint.sh would accept a SKILL.md whose contract has `Stop:` followed only by a tab" [fact: …/20261006-213551-84908/make-skill-honest-report/r1/grade.md]
- Fix: ` *[^ ]` became `[[:space:]]*[^[:space:]]` [fact: commit 3f5c642, evals/lint.sh:26,29]
- No fixture or test for lint.sh exists [fact: `git ls-files evals/`]

## F4: Lesson C, ground's "look into" vs investigate
Answer: Unresolved. The overlap is real, but `investigate` exists only inside an eval run.
- ground's description includes "find out", "look into" [fact: kernel/ground/SKILL.md:3]
- The make-skill case asks investigate to trigger on "look into why" [fact: evals/cases/make-skill-honest-report/prompt.md:1]
- The agent raised it as O1, "narrow `ground`'s description?" [fact: …/213551-84908/…/grade.md]
- No `workflows/investigate/` in the repo, and no negative-trigger case between ground and any workflow [fact: `git ls-files`]
- A routing collision is decided by intent (which skill should own the phrase), not by fact [inference]

## F5: Precedent: fixing the check instead of the skill
Answer: It has happened once. After a make-skill FAIL, the rubric was loosened rather than the skill changed.
- Rubric criterion 4 was relaxed so that "out-of-scope open questions in the report are fine" [fact: commit 3f5c642 message; evals/cases/make-skill-honest-report/rubric.md:4]
- The re-run then passed [fact: …/20261006-213914-87515/results.txt]
- So codify has to decide whether the skill or the check is wrong, and that is a judge-shaped decision [inference from F5 and kernel/judge]

## F6: Precedent: a de facto noise threshold
Answer: 1 failure in 4 is treated as possible noise and left alone. 2 in 4 was acted on.
- "The single failures for `distill` and `judge` could be noise… Leave them unchanged unless they come back." [fact: README.md, Current cases]
- 2 of 4 for ask led to the fix [fact: F2]

## F7: Which enforcement levels exist in this repo?
Answer, strongest first: lint.sh (free, deterministic, every skill), case check.sh (deterministic, but only runs inside a paid eval), runner or harness config (run.sh flags such as `--disallowedTools`), rubric.md (LLM grader), SKILL.md text, KERNEL.md text (in every session's context), README notes. There is no type layer.
- [fact: evals/lint.sh; evals/run.sh:63-70; evals/lib.sh; README.md Writing a case]
- Kernel text is always loaded and so costs context in every session, while SKILL.md text costs context only when the skill is invoked [inference from README rule 1 and Claude Code skill loading]

## F8: Where would codify's "record of why" live?
Answer: Nowhere yet. The kernel's State table has no codify log.
- The State table lists findings, decisions, proofs, verdicts and distill formats [fact: kernel/KERNEL.md:38-44]
- Adding a log row is a kernel edit, which make-skill may not make [fact: workflows/make-skill/SKILL.md Rules]. If codify needs one, it has to be an open question to the user, or codify's first act.
