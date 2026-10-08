---
name: ground
description: Answer a factual question from primary sources (code, git history, docs, runtime, transcripts, web), tagging every claim as fact-with-source or inference. Use before asking the human anything that could be looked up, before changing unfamiliar code, and for "how does X work", "why is X like this", "what breaks if I change X", "find out", "look into".
---

# Ground

Find out what is true. Not what is likely, and not what was true last time.

## Contract

```
Layer:     verb
Input:     question; optional source scope (code | history | docs | runtime | transcripts | web); slug
Output:    entry F<n> appended to $WORK/<slug>/findings.md
Stop:      every load-bearing part of the answer has at least one fact, or the question is declared unanswerable with what was checked
Returns:   the answer in 1–3 lines, plus the F id
```

## Procedure

1. **Sharpen the question.** Rewrite it so it has an observable answer. If it bundles several questions, split them and ground each one.
2. **Pick sources, primary first.** Running it beats reading the code, the code beats the docs, a commit beats a comment, and the original beats a summary. Note which sources you skipped and why.
3. **Search wide in isolation.** If answering means sweeping many files or a long history, do it in a subagent that returns only citations and short excerpts.
4. **Read the actual source** for each claim you will rely on. Record it with a tag as you go.
5. **Try to break the leading answer.** Look for one counterexample: another call site, a later commit, a config override, an environment difference. Record what you checked even if nothing turned up.
6. **Write the entry.**

## Entry format

```markdown
## F3: <question as sharpened>
Answer: <1–3 lines>
- <claim> [fact: src/auth/session.ts:42]
- <claim> [fact: `git log -S refreshToken` → a1b2c3d]
- <claim> [inference from the two above]
Checked against: <counterexamples looked for>
Unknowns: <what remains open and what would close it>
```

## Rules

- Don't answer from memory when the answer can be checked.
- Don't widen the question. Note adjacent discoveries under Unknowns and leave them.
- Don't fix anything. Ground reports; other skills act.
- If the answer depends on intent rather than fact, say so and hand that part to `ask`. If it depends on behaviour you can't confirm by reading, hand it to `prove`.
